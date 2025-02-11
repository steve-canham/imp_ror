pub mod setup;
mod import;
mod process;
mod summarise;
mod export;
pub mod err;

use std::sync::OnceLock;
use err::AppError;
use setup::log_helper;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub static LOG_RUNNING: OnceLock<bool> = OnceLock::new();

pub async fn run(args: Vec<OsString>) -> Result<(), AppError> {
    
    let config_file = PathBuf::from("./app_config.toml");
    let config_string: String = fs::read_to_string(&config_file)
                    .map_err(|e| AppError::IoReadErrorWithPath(e, config_file))?;
    
    let params = setup::get_params(args, config_string)?;
    let flags = params.flags;
    let test_run = flags.test_run;

    if !test_run {
       log_helper::setup_log(&params.log_folder, &params.source_file_name)?;
       LOG_RUNNING.set(true).unwrap();   // no other thread - therefore should always work
       log_helper::log_startup_params(&params);
    }
            
    let pool = setup::get_db_pool().await?;

    // Processing of the remaining stages depends on the 
    // presence of the relevant CLI flag(s).

    // The first two routines normally run only as an initial 
    // 'setup' of the program's DB, but can be repeated later if required.
    // If combined with data import / processing they will obviously 
    // occur before that import / processing.

    if flags.create_config
    {  
        setup::edit_config().await?;
    }

    if flags.create_lookups
    {  
        setup::create_lup_tables(&pool).await?;
    }

    if flags.create_summary
    {  
        summarise::create_smm_tables(&pool).await?;
    }
    
    // In each of the following stages, the initial step is to recreate 
    // the relevant DB tables, before doing the processing and summarising.
    // These steps are not considered if both create_context and create_summary 
    // are true (as in initial database installation).

    if !(flags.create_lookups && flags.create_summary) {

        if flags.import_ror    // import ror from json file and store in ror schema tables
        {
            import::create_ror_tables(&pool).await?;
            import::import_data(&params.data_folder, &params.source_file_name, 
                                &params.data_version, &params.data_date, &pool).await?;
            if !test_run {
                import::summarise_import(&pool).await?;
            }
        }
    
        if flags.process_data  // transfer data to src tables, and summarise in smm tables
        {
            process::create_src_tables(&pool).await?;
            process::process_data(&params.data_version, &pool).await?;
            summarise::summarise_data(&pool).await?;
        }

        if flags.export_text  // write out summary data from data in smm tables
        { 
            export::export_as_text(&params.output_folder, &params.data_version, &pool).await?;
        }

        if flags.export_csv  // write out summary data from data in smm tables
        { 
            export::export_as_csv(&params.output_folder, &params.data_version, &pool).await?;
        }

        if flags.export_full_csv  // write out summary data for all versions from data in smm tables
        {       
                export::export_all_as_csv(&params.output_folder, &pool).await?;
        }

        if test_run {
            summarise::smm_helper::delete_any_existing_data(&"v99".to_string(), &pool).await?; // Clear any test data from the smm tables.
        }
    }

    Ok(())  
}

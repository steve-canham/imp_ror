/**********************************************************
 * A small library to import ror data from the ror data 
 * dump, maintained on Zenodo, process that data (lightly),
 * and summarise key features of it. The summaries for each 
 * version are retained, whilst the base data is replaced 
 * each time the program is run. The system can also output 
 * a report summarising the key features of the data set. 
 * See the read.me file for further details.
 **********************************************************/

pub mod setup;
mod import;
mod process;
mod summarise;
mod export;
pub mod error_defs;

use error_defs::AppError;
use setup::log_helper;
use std::ffi::OsString;

pub async fn run(args: Vec<OsString>) -> Result<(), AppError> {
    
    // Important that there are no errors in the intial three steps.
    // If one does occur the program exits.
    // 1) Collect initial parameters such as file names and CLI flags. 
    // CLI arguments are collected explicitly to facilitate unit testing. 
    // of 'get_params'. Relevant environmental variables are also read.
    // 2) Establish a log file, in the specified data folder.
    // The initial parameters are recorded as the initial part of the log.
    // 3) The database connection pool is established for the database "ror".

    let params = setup::get_params(args).await?;
    let flags = params.flags;
    let test_run = flags.test_run;

    if !test_run {
       log_helper::setup_log(&params.log_folder, &params.source_file_name)?;
       log_helper::log_startup_params(&params);
    }
            
    let pool = setup::get_db_pool().await?;

    // Processing of the remaining stages depends on the 
    // presence of the relevant CLI flag(s).

    // The first two routines normally run only as an initial 
    // 'setup' of the program's DB, but can be repeated later if required.
    // If combined with data import / processing they will obviously 
    // occur before that import / processing.

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
            export::export_as_text(&params.output_folder, &params.output_file_name, 
                    &params.data_version, &pool).await?;
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

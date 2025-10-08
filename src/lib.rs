pub mod setup;
pub mod err;
mod import;
mod process;
mod summarise;
mod export;


use setup::cli_reader;
use err::AppError;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub async fn run(args: Vec<OsString>) -> Result<(), AppError> {
    
    // If no config file the command line arguments are forced into
    // the equivalent of a user's initialisation request. Otherwise
    // they are read using the CLAP based CLI reader.

    let cli_pars: cli_reader::CliPars;
    if !cli_reader::config_file_exists() {
        cli_pars = cli_reader::get_initalising_cli_pars();  // force flags to equal initialisation request
    }
    else {
        cli_pars = cli_reader::fetch_valid_arguments(args)?;
    }
    let flags = cli_pars.flags;

    // The create config file flag may nave been set explicitly by the user
    // or generated automatically by the absence of a config file. The config
    // file must be generated / edited before the rest of the program proceeds.

    if flags.create_config {
        if cli_reader::config_file_exists() {
            setup::edit_config()?; 
        }
        else {
            setup::create_config()?; 
        }
    }

    let config_file = PathBuf::from("./app_config.toml");
    let config_string: String = fs::read_to_string(&config_file)
                    .map_err(|e| AppError::IoReadErrorWithPath(e, config_file))?;
    
    let params = setup::get_params(cli_pars, &config_string)?;

    setup::establish_log(&params, &config_string)?;
    let pool = setup::get_db_pool().await?;
    let test_run = flags.test_run;

    // The first two routines below normally run only as an initial 
    // 'setup' of the program's config file and DB, but can be repeated later if required.

    if flags.create_lookups
    {  
        setup::create_lup_tables(&pool).await?;
    }

    if flags.create_summary
    {  
        summarise::create_smm_tables(&pool).await?;
    }
    
    // The routines below run as part of the 'normal' functioning of the program.
    // Exactluy which is dependent on the flags provided in the CLI

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

    if test_run {  // Clear any test data from the smm tables.
        summarise::smm_helper::delete_any_existing_data(&"v99".to_string(), &pool).await?; 
    }

    Ok(())  
}

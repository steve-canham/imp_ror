pub mod setup;
pub mod config;
pub mod err;
mod import;
mod process;
mod summarise;
mod export;
mod sql;

use err::AppError;
use std::ffi::OsString;

pub async fn run(args: Vec<OsString>) -> Result<(), AppError> {

    // Combine parameters from CLI and config file,
    // establish a log file and obtain a pool of database connections.

    let cli = setup::get_command_line_args(args)?;
    let config = setup::get_config_file_args(cli.flags)?;
    let params = setup::combine_args(cli, config)?;
    
    setup::establish_log(&params)?;
    let pool = setup::db_pars::get_db_pool().await?;
    
    // The first two routines below normally run only as part of an initial
    // 'setup' of the program, after setting up the config file and DB, but can be repeated later if required.

    let flags = params.flags;
    if flags.create_lookups
    {
        setup::create_lup_tables(&pool).await?;
    }
 
    if flags.create_summary
    {
        summarise::create_smm_tables(&pool).await?;
    }

    // The routines below run as part of the 'normal' functioning of the program.
    // Exactly which is dependent on the flags provided in the CLI

    if flags.import_ror   {     // import ror from json file and store in src schema tables
        import::import_data(&params, &pool).await?;
        
        if !flags.test_run {
            import::summarise_import(&pool).await?;
        }

        process::process_data(&params, &pool).await?;
        summarise::store_summary_data(&params, &pool).await?;
        export::export_as_text(&params, &pool).await?;
    }


    if flags.export_csv  // write out summary data from data in smm tables
    {
        export::export_as_csv(&params, &pool).await?;
    }

    if flags.export_all_csv  // write out summary data for all versions from data in smm tables
    {
        export::export_all_as_csv(&params, &pool).await?;
    }

    //if flags.test_run {  // Clear any test data from the smm tables.
    //    summarise::smm_helper::delete_any_existing_data(&"v99".to_string(), false, &pool).await?;
    //}

    Ok(())
}

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

    let (cli_pars, config_string) = setup::obtain_parameters(args)?;
    let params = setup::combine_params(cli_pars, &config_string)?;

    setup::establish_log(&params, &config_string)?;
    let pool = setup::db_pars::get_db_pool().await?;
    let flags = params.flags;
    let test_run = flags.test_run;
    
    // The first two routines below normally run only as part of an initial
    // 'setup' of the program, after setting up the config file and DB, but can be repeated later if required.

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

    if flags.import_ror   { // import ror from json file and store in src schema tables
        import::import_data(&params.data_folder, &params.source_file_name,
                            &params.data_version, &params.data_date, &pool).await?;
        if !test_run {
            import::summarise_import(&pool).await?;
        }
    }

    if flags.process_data  { // transfer data to ppr tables, and summarise in smm tables
        process::process_data(&params.data_version, &pool).await?;
        summarise::store_summary_data(&pool).await?;
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

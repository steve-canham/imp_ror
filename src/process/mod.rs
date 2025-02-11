mod src_data_importer;
mod src_data_processor;
mod src_create_tables;
mod src_rmv_dup_names;


use log::{info, error};
use sqlx::{Pool, Postgres};
use crate::AppError;


pub async fn create_src_tables(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    match src_create_tables::create_tables(pool).await {
        Ok(()) => info!("Tables created for src schema"),
        Err(e) => {
            error!("An error occured while creating the src schema tables: {}", e);
            return Err(e)
            },
    };
    match src_create_tables::create_admin_data_table(pool).await {
        Ok(()) => info!("Admin data table created in src schema"),
        Err(e) => {
            error!("An error occured while creating the src admin data table: {}", e);
            return Err(e)
            },
    };
    Ok(())

}

pub async fn process_data(data_version: &String, pool : &Pool<Postgres>) -> Result<(), AppError>
{

    // Import the data from ror schema to src schema.

    match src_data_importer::import_data(data_version, pool).await
    {
        Ok(()) => {
            info!("Data imported from ror to src tables"); 
        },
        Err(e) => {
            error!("An error occured while transferring to the src tables: {}", e);
            return Err(e)
            },
    }

    // Calculate number of attributes for each org, and populate the admin data table with results.

    match src_data_processor::store_org_attribute_numbers(pool).await
    {
        Ok(()) => {
            info!("All org attributes counted and results added to admin table"); 
        },
        Err(e) => {
            error!("An error occured while processing the imported data: {}", e);
            return Err(e)
            },
    }

    // Add the script codes to the names.

    match src_data_processor::add_script_codes(pool).await
    {
        Ok(()) => {
            info!("Script codes added to organisation names"); 
        },
        Err(e) => {
            error!("An error occured while adding the script codes: {}", e);
            return Err(e)
            },
    }

    Ok(())
}
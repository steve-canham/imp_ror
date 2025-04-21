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

    // if data version = "" obtain it from the ror tables
    let mut dv = data_version.to_string();
    if data_version == "" {
        dv = get_current_data_version(pool).await?;
    }

    match src_data_importer::import_data(dv, pool).await
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
    
    // Improve data cleanliness in the names table

    src_data_processor::tidy_names(pool).await?;
    src_data_processor::prepare_names_for_script_codes(pool).await?;
    src_data_processor::add_script_codes(pool).await?;
    src_data_processor::add_langs_for_nonlatin_codes(pool).await?;
    src_data_processor::clean_japanese_script_codes(pool).await?;
    src_data_processor::clean_script_codes_with_numbers(pool).await?;

    Ok(())

}


pub async fn get_current_data_version(pool: &Pool<Postgres>)-> Result<String, AppError> {
    
    let sql = "select version from ror.version_details";
    sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

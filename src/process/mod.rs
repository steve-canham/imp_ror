mod ppr_data_importer;
mod ppr_data_processor;
mod ppr_create_tables;
mod ppr_rmv_dup_names;
mod ppr_script_coder;

use log::{info, error};
use sqlx::{Pool, Postgres};
use crate::AppError;


pub async fn create_ppr_tables(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    match ppr_create_tables::create_tables(pool).await {
        Ok(()) => info!("Tables created for ppr schema"),
        Err(e) => {
            error!("An error occured while creating the ppr schema tables: {}", e);
            return Err(e)
            },
    };
    match ppr_create_tables::create_admin_data_table(pool).await {
        Ok(()) => {
            info!("Admin data table created in ppr schema");
            info!(""); 
        },
        Err(e) => {
            error!("An error occured while creating the ppr admin data table: {}", e);
            return Err(e)
            },
    };
    Ok(())

}

pub async fn process_data(data_version: &String, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Import the data from ror schema to ppr schema.

    // if data version = "" obtain it from the ror tables

    let mut dv = data_version.to_string();
    if data_version == "" {
        dv = get_current_data_version(pool).await?;
    }

    match ppr_data_importer::import_data(dv, pool).await
    {
        Ok(()) => {
            info!("Data imported from ror to ppr tables"); 
            info!(""); 
        },
        Err(e) => {
            error!("An error occured while transferring to the ppr tables: {}", e);
            return Err(e)
            },
    }

    // Calculate number of attributes for each org, and populate the admin data table with results.

    match ppr_data_processor::store_org_attribute_numbers(pool).await
    {
        Ok(()) => {
            info!("All org attributes counted and results added to admin table"); 
            info!(""); 
        },
        Err(e) => {
            error!("An error occured while processing the imported data: {}", e);
            return Err(e)
            },
    }
    
    // Generate script codes

    ppr_script_coder::prepare_names_for_script_codes(pool).await?;
    ppr_script_coder::add_script_codes(pool).await?;
    ppr_script_coder::clean_japanese_script_codes(pool).await?;
    ppr_script_coder::clean_double_script_codes(pool).await?;
    ppr_script_coder::apply_script_codes_to_names(pool).await?;

    // Update lang codes from scripts where possible, record lang code source type

    //ppr_script_coder::update_lang_code_source("ror", pool).await?;
    //ppr_script_coder::add_langs_for_nonlatin_codes(pool).await?;
    //ppr_script_coder::update_lang_code_source("script_auto", pool).await?;

    Ok(())
}

pub async fn get_current_data_version(pool: &Pool<Postgres>)-> Result<String, AppError> {
    
    let sql = "select version from src.version_details";
    sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

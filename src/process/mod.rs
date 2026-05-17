mod data_importer;
mod data_processor;
mod rmv_dup_names;
mod script_coder;

use log::{info, error};
use sqlx::{Pool, Postgres};
use crate::AppError;


pub async fn process_data(data_version: &String, pool : &Pool<Postgres>) -> Result<(), AppError>
{

    // First recreate the ppr schema tables - sqlscript in file (path is relative
    // and Linux specific - Windows would need a similar string but with backslashes)

    let sql = include_str!("../../sql/create_ppr_tables.sql");
    sqlx::raw_sql(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    // Import the data from ror schema to ppr schema.

    // if data version = "" obtain it from the ror tables

    let mut dv = data_version.to_string();
    if data_version == "" {
        dv = get_current_data_version(pool).await?;
    }

    match data_importer::import_data(dv, pool).await
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

    match data_processor::store_org_attribute_numbers(pool).await
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

    script_coder::prepare_names_for_script_codes(pool).await?;
    script_coder::add_script_codes(pool).await?;
    script_coder::clean_japanese_script_codes(pool).await?;
    script_coder::clean_double_script_codes(pool).await?;
    script_coder::apply_script_codes_to_names(pool).await?;

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

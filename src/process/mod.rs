mod data_transferrer;
mod data_processor;
mod rmv_dup_names;
mod script_coder;

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
    
    // Import the data from ror schema to ppr schema. First get data version - 
    // if data version = "" (i.e. is implicit) obtain it from the ror tables.

    let mut dv = data_version.to_string();
    if data_version == "" {
        dv = get_current_data_version(pool).await?;
    }

    data_transferrer::transfer_data(&dv, pool).await?;
    
    // Calculate number of attributes for each org, and populate the admin data table with results.

    data_processor::store_org_attribute_numbers(pool).await?;
        
    // Generate script codes

    script_coder::apply_script_codes(pool).await?;

    Ok(())
}

pub async fn get_current_data_version(pool: &Pool<Postgres>)-> Result<String, AppError> {
    
    let sql = "select version from src.version_details";
    sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

mod smm_storer;
mod smm_structs;
pub mod smm_helper;
mod smm_create_tables;

use log::{info, error};
use sqlx::{Pool, Postgres};
use crate::AppError;

pub async fn create_smm_tables(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    match smm_create_tables::create_tables(pool).await {
        Ok(()) => info!("Tables created for smm schema"),
        Err(e) => {
            error!("An error occured while creating the smm schema tables: {}", e);
            return Err(e)
            },
    };
    Ok(())
}

pub async fn summarise_data(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Store data into smm tables.

    match smm_storer::store_summary_data(pool).await
    {
        Ok(()) => {
            info!("All summary data transferred to smm tables"); 
            return Ok(())
        },
        Err(e) => {
            error!("An error occured while constructing  summary data: {}", e);
            return Err(e)
            },
    }
}

mod export_text;
mod export_csv;
mod export_structs;

use log::{info, error};
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use crate::err::AppError;

pub async fn export_as_text(output_folder : &PathBuf, data_version: &String, 
                            pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Write out summary data for this dataset into the designated file

    check_data_version_present_in_summ_data(data_version, pool).await?;

    let r = export_text::generate_text(output_folder, data_version, pool).await;
    match r {
        Ok(()) => {
            info!("Data summary generated as text file"); 
            return Ok(())
        },
        Err(e) => {
            error!("An error occured while writing out the text file: {}", e);
            return Err(e)
            },
    }
}


pub async fn export_as_csv(output_folder : &PathBuf, data_version: &String, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Write out summary data for this as a set of csv files into the designated folder

    check_data_version_present_in_summ_data(data_version, pool).await?;

    let r = export_csv::generate_csv(output_folder, data_version, pool).await;
    match r {
        Ok(()) => {
            info!("Data summary generated as csv files"); 
            return Ok(())
        },
        Err(e) => {
            error!("An error occured while writing out the csv files: {}", e);
            return Err(e)
        },
    }
}


pub async fn export_all_as_csv(output_folder : &PathBuf, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Write out summary data for all versions as a set of csv files into the designated folder

    let r = export_csv::generate_all_versions_csv(output_folder, pool).await;
    match r {
        Ok(()) => {
            info!("Data summary generated as csv files"); 
            return Ok(())
        },
        Err(e) => {
            error!("An error occured while writing out the csv files: {}", e);
            return Err(e)
        },
    }
}


async fn check_data_version_present_in_summ_data(data_version: &String, pool: &Pool<Postgres>)-> Result<(), AppError> {
    
    let sql = r#"SELECT EXISTS(select vcode from smm.version_summaries where vcode = '"#.to_string() + data_version + r#"')"#;
    let check_result: bool  = sqlx::query_scalar(&sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    if !check_result
    {
        return Result::Err(AppError::MissingVersion(data_version.to_string()));
    }
    else {
        Ok(())
    }
}

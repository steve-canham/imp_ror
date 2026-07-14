mod export_text;
mod export_csv;
mod export_structs;
mod export_helpers;

use log::{info, error};
use sqlx::{Pool, Postgres};
use crate::{err::AppError, setup::InitParams};

pub async fn export_as_text(params: &InitParams, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Write out summary data for this dataset into the designated file
    // The data version is the 'current' ppr version.
             
    let dv = get_current_data_version(pool).await?;    
    check_data_version_present_in_summary_data(&dv, params.flags.inc_withdrawn, pool).await?;  // ensure version summary is present

    export_text::generate_text(&params.output_folder, &dv, params.flags.inc_withdrawn, pool).await?;
    info!("Data summary generated as text file"); 
    Ok(())
}


pub async fn export_as_csv(params: &InitParams, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Write out summary data for designated version into the preset CSV files
    
    let mut dv = params.data_version.to_string();
    if params.data_version == "" {     // no data version given explicitly
        dv = get_current_data_version(pool).await?;
    }

    // if current or required version has -w = true, remember to add -w to the -x command
    // Ottherwise if there is a current version with -w  = false that will be the one exported
    
    check_data_version_present_in_summary_data(&dv, params.flags.inc_withdrawn, pool).await?;
    
    // Write out summary data for this as a set of csv files into the designated folder

    export_csv::generate_csv(&params.output_folder, &dv, params.flags.inc_withdrawn, pool).await?;
    info!("Data summary generated as csv files"); 
    Ok(())
   
}

pub async fn export_all_as_csv(params: &InitParams, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Write out summary data for all versions as a set of csv files into the designated folder

    let r = export_csv::generate_all_versions_csv(&params.output_folder, params.flags.inc_withdrawn, pool).await;
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

async fn check_data_version_present_in_summary_data(data_version: &String, inc_wd: bool, pool: &Pool<Postgres>)-> Result<(), AppError> {
    
    let sql = format!(r#"SELECT EXISTS(select vcode from smm.version_summaries 
                         where vcode = '{data_version}'
                         and inc_wd = {inc_wd})"#, );
    let check_result: bool  = sqlx::query_scalar(&sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    match check_result {
        true => Ok(()),
        _ => Result::Err(AppError::MissingVersion(data_version.to_string()))
    }
}


async fn get_current_data_version(pool: &Pool<Postgres>) -> Result<String, AppError> {
    
    let sql = "select version from ppr.version_details";
    sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))

}

mod smm_structs;
pub mod smm_helper;

use crate::{setup::InitParams, sql::create_smm_tables};
use smm_structs::FileParams;
use sqlx::{Pool, Postgres};
use chrono::NaiveDate;
use crate::AppError;
use log::info;

pub async fn create_smm_tables(pool: &Pool<Postgres>) -> Result<(), AppError>
{
    let sql = create_smm_tables::get_sql();
    sqlx::raw_sql(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(())
}

pub async fn store_summary_data (params: &InitParams, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Obtain the data version and date (as previously stored in table during import process)
    // and derive standard first item of many sql statements below.

    let sql = r#"SELECT version as vcode, data_date as vdate_as_string, 
               data_days as vdays, inc_wd from ppr.version_details;"#;
    let fp: FileParams = sqlx::query_as(sql).fetch_one(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let vcode = fp.vcode;
    let vdate = NaiveDate::parse_from_str(&fp.vdate_as_string, "%Y-%m-%d").unwrap();
    let vdays = fp.vdays;
    let inc_wd = fp.inc_wd;
    let sdv = format!("select '{}' as vcode, ",  vcode);   // common beginning of sql statements
    
    // Delete existing data in smm. tables for this version and construct the 
    // initial version summary table by obtaining record counts of all ppr tables.                  

    smm_helper::delete_any_existing_data(&vcode, inc_wd, pool).await?;

    // Get status breakdown.

    let num_active = smm_helper::get_count("select count(*) from ppr.core_data where status = 1", pool).await?;
    let num_inactive = smm_helper::get_count("select count(*) from ppr.core_data where status = 2", pool).await?;
    let num_withdrawn =  if params.flags.inc_withdrawn {
        smm_helper::get_count("select count(*) from ppr.core_data where status = 3", pool).await?
    } 
    else {
        smm_helper::get_count("select count(*) from rec.withdrawn", pool).await?
    };
    let num_recs = num_active + num_inactive + num_withdrawn;
    let num_denom = if params.flags.inc_withdrawn {num_recs} else {num_active + num_inactive};
    
    // Get table count numbers
    
    let num_names = smm_helper::get_count("select count(*) from ppr.names", pool).await?;
    let num_types = smm_helper::get_count("select count(*) from ppr.type", pool).await?;
    let num_links = smm_helper::get_count("select count(*) from ppr.links", pool).await?;
    let num_ext_ids = smm_helper::get_count("select count(*) from ppr.external_ids", pool).await?;
    let num_rels = smm_helper::get_count("select count(*) from ppr.relationships", pool).await?;
    let num_locations = smm_helper::get_count("select count(*) from ppr.locations", pool).await?;
    let num_domains = smm_helper::get_count("select count(*) from ppr.domains", pool).await?;
    
    let sql = r#"INSERT into smm.version_summaries (vcode, inc_wd, vdate, vdays, num_recs, 
                      num_active, num_inactive, num_withdrawn, num_denom, num_names,
                      num_types, num_links, num_ext_ids, num_rels, num_locations, num_domains)
                      values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"#;
    sqlx::query(sql).bind(&vcode).bind(inc_wd).bind(vdate).bind(vdays)
        .bind(num_recs).bind(num_active).bind(num_inactive).bind(num_withdrawn).bind(num_denom)
        .bind(num_names).bind(num_types).bind(num_links)
        .bind(num_ext_ids).bind(num_rels).bind(num_locations).bind(num_domains)
        .execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("Version summary record created");

    // Summarise the data in the groups represented by the functions below.

    smm_helper::create_name_attributes(&sdv, &vcode, inc_wd, num_denom, num_names, pool).await?;
    smm_helper::create_other_attributes(&sdv, inc_wd, num_denom, num_types, num_ext_ids, 
                            num_links, num_rels, pool).await?;
    info!("Attribute summaries created");

    smm_helper::create_count_distributions(&sdv, inc_wd, num_denom, pool).await?;     
    info!("Count distributions created");

    smm_helper::create_ranked_count_distributions(&vcode, &sdv, inc_wd, num_names, num_locations, pool).await?;   
    info!("Ranked count distributions created");

    smm_helper::create_type_linked_tables(&sdv, inc_wd, pool).await?;
    smm_helper::store_singletons(&vcode, inc_wd, num_denom, num_names, pool).await?;
    info!("All summary data transferred to smm tables"); 
    
    Ok(())
}



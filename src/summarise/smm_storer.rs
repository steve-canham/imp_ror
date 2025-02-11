use super::smm_helper;
use super::smm_structs::FileParams;
use sqlx::{Pool, Postgres};
use chrono::NaiveDate;
use crate::AppError;
use log::info;

pub async fn store_summary_data (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Obtain the data version and date (as previously stored in table during import process)
    // and derive standard first two items of many sql statements below.

    let sql = r#"SELECT version as vcode, data_date as vdate_as_string, 
               data_days as vdays from src.version_details;"#;
    let fp: FileParams = sqlx::query_as(sql).fetch_one(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let vcode = fp.vcode;
    let vdate = NaiveDate::parse_from_str(&fp.vdate_as_string, "%Y-%m-%d").unwrap();
    let vdays = fp.vdays;
    let sdv = "select \'".to_string() + &vcode + "\' as vcode, ";

    // Delete existing data in smm. tables and construct the initial version
    // summary table by obtaining record counts of all src tables.                  

    smm_helper::delete_any_existing_data(&vcode, pool).await?;

    let num_orgs = smm_helper::get_count("select count(*) from src.core_data", pool).await?;
    let num_names = smm_helper::get_count("select count(*) from src.names", pool).await?;
    let num_types= smm_helper::get_count("select count(*) from src.type", pool).await?;
    let num_links= smm_helper::get_count("select count(*) from src.links", pool).await?;
    let num_ext_ids= smm_helper::get_count("select count(*) from src.external_ids", pool).await?;
    let num_rels= smm_helper::get_count("select count(*) from src.relationships", pool).await?;
    let num_locations= smm_helper::get_count("select count(*) from src.locations", pool).await?;
    let num_domains= smm_helper::get_count("select count(*) from src.domains", pool).await?;
    
    let sql = r#"INSERT into smm.version_summaries (vcode, vdate, vdays, num_orgs, num_names,
                      num_types, num_links, num_ext_ids, num_rels, num_locations , num_domains)
                      values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#;
    sqlx::query(sql).bind(&vcode).bind(vdate).bind(vdays)
        .bind(num_orgs).bind(num_names).bind(num_types).bind(num_links)
        .bind(num_ext_ids).bind(num_rels).bind(num_locations).bind(num_domains)
        .execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("Version summary record created");

    // Summarise the data in the groups represented by the functions below.

    let num_orgs_str = num_orgs.to_string();

    smm_helper::create_name_attributes(&sdv, &vcode, &num_orgs_str, &num_names.to_string(), pool).await?;

    smm_helper::create_other_attributes(&sdv, &num_orgs_str, &num_types.to_string(), &num_ext_ids.to_string(), 
                            &num_links.to_string(), &num_rels.to_string(), pool).await?;

    info!("Attribute summaries created");

    smm_helper::create_count_distributions(&sdv, &num_orgs_str, pool).await?;     

    info!("Count distributions created");

    //let num_ne = smm_helper::get_count("select count(*) from src.names where lang_code <> 'en'", pool).await?;
    //let num_nltn = smm_helper::get_count("select count(*) from src.names where script_code <> 'Latn'", pool).await?;
    //let num_nus = smm_helper::get_count("select count(*) from src.locations where country_code <> 'US'", pool).await?;

    smm_helper::create_ranked_count_distributions(&vcode, &sdv, num_names, num_locations, pool).await?;   

    info!("Ranked count distributions created");

    smm_helper::create_type_linked_tables(&sdv, pool).await?;

    smm_helper::store_singletons(&vcode, num_orgs, num_names, pool).await?;

    Ok(())
}

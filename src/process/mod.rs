
mod rmv_dup_names;
mod script_coder;

use crate::sql::create_ppr_tables;
use crate::sql::transfer_to_ppr::*;
use crate::sql::process_num_data::*;
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::err::AppError;


pub async fn process_data(data_version: &String, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // First recreate the ppr schema tables

    let sql = create_ppr_tables::get_sql();
    sqlx::raw_sql(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    // Import the data from ror schema to ppr schema. First get data version - 
    // if data version = "" (i.e. is implicit) it will be obtained from the ror tables.
    // If data version explicitly given check it is the correct one!

    if data_version != "" {
        check_data_version_matches_ror_schema_data(data_version, pool).await?;
    }
    execute_sql(get_version_details_sql(), pool).await?;
    
    rmv_dup_names::remove_dups(pool).await?;  // done here to prevent PK errors in core_data
    
    execute_sql(get_core_data_sql(), pool).await?;
    execute_sql(get_admin_data_sql(), pool).await?;
    info!("Core organisation data transferred to ppr table");

    execute_sql(get_import_names_sql(), pool).await?;
    info!("Name data transferred to ppr table");
    
    execute_sql(get_links_sql(), pool).await?;
    execute_sql(get_external_ids_sql(), pool).await?;
    execute_sql(get_types_sql(), pool).await?;
    info!("External Ids, links and types data transferred to ppr table");
    
    execute_sql(get_locations_sql(), pool).await?;
    execute_sql(get_relationships_sql(), pool).await?;
    execute_sql(get_domains_sql(), pool).await?;
    info!("Location, relationship and domain data transferred to ppr table");
    info!("Data imported from ror to ppr tables"); 
    info!(""); 
    
    // Calculate number of attributes for each org, and populate the admin data table with results.
    
    execute_sql(get_name_nums_sql(), pool).await?;
    execute_sql(get_label_nums_sql(), pool).await?;
    execute_sql(get_alias_nums_sql(), pool).await?;
    execute_sql(get_acronym_nums_sql(), pool).await?;

    info!("Basic name data summarised in admin data table");

    execute_sql(get_nacro_nums_sql(), pool).await?;
    execute_sql(get_names_wolc_nums_sql(), pool).await?;
    execute_sql(get_nacro_wolc_nums_sql(), pool).await?;

    info!("Name language code data summarised in admin data table");

    execute_sql(get_companies_nums_sql(), pool).await?;
    execute_sql(get_types_nums_sql(), pool).await?;

    info!("Types data summarised in admin data table");
    
    execute_sql(get_isni_nums_sql(), pool).await?;
    execute_sql(get_grid_nums_sql(), pool).await?;
    execute_sql(get_fundref_nums_sql(), pool).await?;
    execute_sql(get_wikidata_nums_sql(), pool).await?;
    execute_sql(get_ext_ids_nums_sql(), pool).await?;

    info!("External ID summarised in admin data table");

    execute_sql(get_wikipedia_nums_sql(), pool).await?;
    execute_sql(get_website_nums_sql(), pool).await?;
    execute_sql(get_links_nums_sql(), pool).await?;

    info!("Links data summarised in admin data table");
    
    execute_sql(get_parrels_nums_sql(), pool).await?;
    execute_sql(get_chrels_nums_sql(), pool).await?;
    execute_sql(get_relrels_nums_sql(), pool).await?;
    execute_sql(get_predrels_nums_sql(), pool).await?;
    execute_sql(get_succrels_nums_sql(), pool).await?;
    execute_sql(get_domains_nums_sql(), pool).await?;

    info!("Relationship, domain data summarised in admin data table");

    execute_sql(get_locations_nums_sql(), pool).await?;
    execute_sql(get_subdivs_nums_sql(), pool).await?;
    execute_sql(get_countries_nums_sql(), pool).await?;

    info!("Location data summarised in admin data table");

    execute_sql(update_core_data_sql1(), pool).await?;
    execute_sql(update_core_data_sql2(), pool).await?;
    execute_sql(update_core_data_sql3(), pool).await?;
    execute_sql(update_core_data_sql4(), pool).await?;
 
    info!("Location data added to core data table");
    info!("All org attributes counted and results added to admin table"); 
    info!(""); 
        
    // Generate script codes

    script_coder::apply_script_codes(pool).await?;

    Ok(())
}


async fn check_data_version_matches_ror_schema_data(data_version: &String, pool: &Pool<Postgres>)-> Result<(), AppError> {

    // Part of a double check. Would only fail if an explict version parameter 
    // had been provided with -p that did not match the current version in the src tables
    
    let sql = "select version from src.version_details";
    let stored_version: String  = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    if stored_version != data_version.to_string()
    {
        Err(AppError::IncompatibleVersions(data_version.to_string(), stored_version))
    }
    else {
        Ok(())
    }
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}
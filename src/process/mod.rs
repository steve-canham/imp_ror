mod process_names;
mod rmv_dup_names;
mod script_coder;

use crate::setup::InitParams;
use crate::sql::create_ppr_tables;
use crate::sql::transfer_to_ppr::*;
use crate::sql::process_num_data::*;
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::err::AppError;

pub async fn process_data(params: &InitParams, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Import the data from src schema to ppr schema. 
    // First recreate the ppr schema tables
    // Then get data version from the src tables, but also include the 'include withdrawn' boolean parameter.
    // Then remove duplicate names in the system before transferring the src data across to the ppr tables.
    // If withdrawn organisations are removed (the default) this is domne at the end.
     
    let sql = create_ppr_tables::get_sql();
    sqlx::raw_sql(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    let sql = format!(r#"insert into ppr.version_details (version, data_date, data_days, inc_wd)
        select version, data_date, data_days, {} from src.version_details;"#, params.flags.inc_withdrawn);
    execute_sql(&sql, pool).await?;

    process_names::clean_names1(pool).await?;  // before che3cking for duplicates so some basic tidying of names
       
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

    if !params.flags.inc_withdrawn {
        
        // Normally, remove the withdawn records from the tables and store them separately
        // Then remove the corresponding records from all other tables
        
        execute_sql(get_withdrawn_sql(), pool).await?;
        let sql = "select count(*) from rec.withdrawn";
        let wd: i64 = sqlx::query_scalar(sql)
            .fetch_one(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        info!("{wd} Withdrawn records table created, within rec.withdrawn table");
        
        delete_withdrawn("admin_data", pool).await?;
        delete_withdrawn("names", pool).await?;
        delete_withdrawn("locations", pool).await?;
        delete_withdrawn("type", pool).await?;
        delete_withdrawn("links", pool).await?;
        delete_withdrawn("external_ids", pool).await?;
        delete_withdrawn("relationships", pool).await?;
        delete_withdrawn("domains", pool).await?;
        delete_withdrawn("admin_data", pool).await?;
        delete_withdrawn("core_data", pool).await?;
        info!("Withdrawn record details removed from attribute tables");
        info!("");
        
    }
    else {    
        
        // Withdrawn orgs are included. Delete any rec.withdrawn table

        let sql = "drop table if exists rec.withdrawn;";
        sqlx::raw_sql(sql).execute(pool)
            .await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        info!("Withdrawn records retained within main dataset");
        info!("");
    }
    
    // Generate script codes for names
   
    script_coder::apply_script_codes(pool).await?;

    Ok(())
}

async fn delete_withdrawn(table_name: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {

    let sql = format!(r#"delete from ppr.{table_name} a
        using rec.withdrawn w
        where a.id = w.ror_id"#);
    execute_sql(&sql, pool).await
}


async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    sqlx::query(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::err::AppError;
use super::ppr_rmv_dup_names;

pub async fn import_data (data_version: String, pool: &Pool<Postgres>) -> Result<(), AppError> {

    check_data_version_matches_ror_schema_data(data_version, pool).await?;
    execute_sql(get_version_details_sql(), pool).await?;
    
    ppr_rmv_dup_names::remove_dups(pool).await?;  // done here to prevent PK errors in core_data
    
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

    Ok(())
}


async fn check_data_version_matches_ror_schema_data(data_version: String, pool: &Pool<Postgres>)-> Result<(), AppError> {
    
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

fn get_version_details_sql <'a>() -> &'a str {
       r#"insert into ppr.version_details (version, data_date, data_days)
       select version, data_date, data_days from src.version_details;"#
}

fn get_core_data_sql <'a>() -> &'a str {
   
        r#"insert into ppr.core_data (id, ror_full_id, 
        ror_name, status, established)
        select c.id, c.ror_full_id, m.value, 
        case 
            when c.status = 'active' then 1
            when c.status = 'inactive' then 2
            when c.status = 'withdrawn' then 3
        end, 
        c.established 
        from src.core_data c
        inner join
            (select id, value from src.names where is_ror_name = true) m
        on c.id = m.id;"#
}


fn get_admin_data_sql <'a>() -> &'a str {
        r#"insert into ppr.admin_data(id, ror_name, created, cr_schema, 
        last_modified, lm_schema)
        select a.id, c.ror_name, a.created, a.cr_schema, a.last_modified, a.lm_schema 
        from src.admin_data a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

fn get_import_names_sql <'a>() -> &'a str {
        
        r#"insert into ppr.names(id, value, name_type, 
        is_ror_name, lang_code)
        select id, value, 
        case 
            when name_type = 'alias' then 7
            when name_type = 'acronym' then 10
            when name_type = 'label' then 5
            else 0
        end,
        case
            when is_ror_name = true then true
            else false
        end, 
        lang
        from src.names"#
}


fn get_links_sql <'a>() -> &'a str  {
        r#"insert into ppr.links(id, ror_name, link_type, link)
        select a.id, c.ror_name, 
        case 
            when a.link_type = 'wikipedia' then 21
            when a.link_type = 'website' then 22
            else 0
        end, 
        value
        from src.links a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

fn get_external_ids_sql <'a>() -> &'a str {
        r#"insert into ppr.external_ids(id, ror_name, id_type, id_value, is_preferred)
        select a.id, c.ror_name,
        case 
            when id_type = 'isni' then 11
            when id_type = 'wikidata' then 12
            when id_type = 'grid' then 13
            when id_type = 'fundref' then 14
            else 0
        end,
        a.id_value, 
        case
            when a.is_preferred = true then true
            else false
        end
        from src.external_ids a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

fn get_types_sql <'a>() -> &'a str {
        r#"insert into ppr.type(id, ror_name, org_type)
        select a.id, c.ror_name, 
        case 
            when org_type = 'government' then 100
            when org_type = 'education' then 200
            when org_type = 'healthcare' then 300
            when org_type = 'company' then 400
            when org_type = 'nonprofit' then 500
            when org_type = 'funder' then 600
            when org_type = 'facility' then 700
            when org_type = 'archive' then 800
            when org_type = 'other' then 900
            else 0
        end
        from src.type a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

fn get_locations_sql <'a>() -> &'a str  {
        r#"insert into ppr.locations(id, ror_name, geonames_id, 
        location, lat, lng, cont_code, cont_name, 
        country_code, country_name, csubdiv_code, csubdiv_name)
        select a.id, c.ror_name, a.geonames_id, a.name,
                a.lat, a.lng, a.continent_code, a.continent_name, 
                a.country_code, a.country_name, 
                a.country_subdivision_code, a.country_subdivision_name
        from src.locations a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

fn get_relationships_sql <'a>() -> &'a str {
        r#"insert into ppr.relationships(id, ror_name, rel_type, related_id, related_name)
        select a.id, c.ror_name, 
        case 
            when a.rel_type = 'parent' then 1
            when a.rel_type = 'child' then 2
            when a.rel_type = 'related' then 3
            when a.rel_type = 'predecessor' then 4
            when a.rel_type = 'successor' then 5
            else 0
        end, 
        a.related_id, a.related_label
        from src.relationships a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

fn get_domains_sql <'a>() -> &'a str {
        r#"insert into ppr.domains(id, ror_name, domain)
        select a.id, c.ror_name, a.value
        from src.domains a
        inner join ppr.core_data c
        on a.id = c.id;"#
}

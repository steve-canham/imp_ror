use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::AppError;
use log::info;

pub async fn store_org_attribute_numbers (pool: &Pool<Postgres>) -> Result<(), AppError> {
        
    execute_sql(get_name_data_sql(), pool).await?;
    execute_sql(get_label_data_sql(), pool).await?;
    execute_sql(get_alias_data_sql(), pool).await?;
    execute_sql(get_acronym_data_sql(), pool).await?;

    info!("Basic name data summarised in admin data table");

    execute_sql(get_nacro_data_sql(), pool).await?;
    execute_sql(get_names_wolc_sql(), pool).await?;
    execute_sql(get_nacro_wolc_sql(), pool).await?;

    info!("Name language code data summarised in admin data table");

    execute_sql(get_companies_sql(), pool).await?;
    execute_sql(get_types_data_sql(), pool).await?;

    info!("Types data summarised in admin data table");
    
    execute_sql(get_isni_data_sql(), pool).await?;
    execute_sql(get_grid_data_sql(), pool).await?;
    execute_sql(get_fundref_data_sql(), pool).await?;
    execute_sql(get_wikidata_data_sql(), pool).await?;
    execute_sql(get_ext_ids_data_sql(), pool).await?;

    info!("External ID summarised in admin data table");

    execute_sql(get_wikipedia_data_sql(), pool).await?;
    execute_sql(get_website_data_sql(), pool).await?;
    execute_sql(get_links_data_sql(), pool).await?;

    info!("Links data summarised in admin data table");
    
    execute_sql(get_parrels_data_sql(), pool).await?;
    execute_sql(get_chrels_data_sql(), pool).await?;
    execute_sql(get_relrels_data_sql(), pool).await?;
    execute_sql(get_predrels_data_sql(), pool).await?;
    execute_sql(get_succrels_data_sql(), pool).await?;
    execute_sql(get_domains_data_sql(), pool).await?;

    info!("Relationship, domain data summarised in admin data table");

    execute_sql(get_locations_data_sql(), pool).await?;
    execute_sql(get_subdivs_data_sql(), pool).await?;
    execute_sql(get_countries_data_sql(), pool).await?;

    info!("Location data summarised in admin data table");

    execute_sql(update_core_data_sql1(), pool).await?;
    execute_sql(update_core_data_sql2(), pool).await?;
    execute_sql(update_core_data_sql3(), pool).await?;
    execute_sql(update_core_data_sql4(), pool).await?;
 
    info!("Location data added to core data table");

    Ok(())
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

fn get_name_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_names = n
    from (
        select id, count(id) as n
        from src.names 
        group by id) c
    where ad.id = c.id;"#
}

fn get_label_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_labels = n
    from (
        select id, count(id) as n
        from src.names where name_type = 5
        group by id) c
    where ad.id = c.id;"#
}

fn get_alias_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_aliases = n
    from (
        select id, count(id) as n
        from src.names where name_type = 7
        group by id) c
    where ad.id = c.id;"#
}

fn get_acronym_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_acronyms = n
    from (
        select id, count(id) as n
        from src.names where name_type = 10
        group by id) c
    where ad.id = c.id;"#
}

fn get_nacro_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_nacro = n_names - n_acronyms;"#
}

fn get_names_wolc_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_names_wolc = n
    from (
        select id, count(id) as n
        from src.names 
        where lang_code is null
        group by id) c
    where ad.id = c.id;"#
}

fn get_nacro_wolc_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_nacro_wolc = n
    from (
        select id, count(id) as n
        from src.names 
        where lang_code is null and name_type <> 10
        group by id) c
    where ad.id = c.id;"#
}

fn get_companies_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set is_company = true
    from src.type t
    where ad.id = t.id
    and t.org_type = 400;"#
}

fn get_types_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_types = n
    from (
        select id, count(id) as n
        from src.type 
        group by id) c
    where ad.id = c.id;"#
}

fn get_isni_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_isni = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 11
        group by id) c
    where ad.id = c.id;"#
}

fn get_grid_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_grid = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 13
        group by id) c
    where ad.id = c.id;"#
}

fn get_fundref_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_fundref = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 14
        group by id) c
    where ad.id = c.id;"#
}

fn get_wikidata_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_wikidata = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 12
        group by id) c
    where ad.id = c.id;"#
}

fn get_ext_ids_data_sql <'a>() -> &'a str {
    r#"update src.admin_data
    set n_ext_ids = n_isni + n_grid + n_fundref + n_wikidata;"#
}

fn get_wikipedia_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_wikipedia = n
    from (
        select id, count(id) as n
        from src.links 
        where link_type = 21
        group by id) c
    where ad.id = c.id;"#
}

fn get_website_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_website = n
    from (
        select id, count(id) as n
        from src.links 
        where link_type = 22
        group by id) c
        where ad.id = c.id;"#
}

fn get_links_data_sql <'a>() -> &'a str {
    r#"update src.admin_data
    set n_links = n_wikipedia + n_website"#
}


fn get_locations_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_locs = n
    from (
        select id, count(id) as n
        from src.locations 
        group by id) c
    where ad.id = c.id;"#
}

fn get_subdivs_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_subdivs = n
    from (
        select id, count(country_subdivision_code) as n
        from
            (select distinct id, country_subdivision_code
            from ror.locations t) t
        group by t.id) c
    where ad.id = c.id;"#
}

fn get_countries_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_countries = n
    from (
        select id, count(country_code) as n
        from
            (select distinct id, country_code
            from ror.locations t) t
        group by t.id) c
    where ad.id = c.id;"#
}

fn get_parrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_parrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 1
        group by id) c
    where ad.id = c.id;"#
}

fn get_chrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_chrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 2
        group by id) c
    where ad.id = c.id;"#
}

fn get_relrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_relrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 3
        group by id) c
    where ad.id = c.id;"#
}

fn get_predrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_predrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 4
        group by id) c
    where ad.id = c.id;"#
}

fn get_succrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_sucrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 5
        group by id) c
    where ad.id = c.id;"#
}

fn get_domains_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_doms = n
    from (
        select id, count(id) as n
        from src.domains 
    group by id) c
    where ad.id = c.id;"#
}

// update core table with location data - for single location orgs

fn update_core_data_sql1 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = t.name,
    csubdiv_code = t.country_subdivision_code,
    country_code = t.country_code
    from  
       (select a.id, name, country_subdivision_code, country_code
        from ror.locations loc
        inner join src.admin_data a
        on loc.id = a.id
        where a.n_locs = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, single subdiv

fn update_core_data_sql2 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = '**multi**',
    csubdiv_code = t.country_subdivision_code,
    country_code = t.country_code
    from 
        (select distinct a.id, country_subdivision_code, country_code
        from ror.locations loc
        inner join src.admin_data a
        on loc.id = a.id
        where a.n_locs > 1
        and a.n_subdivs = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, multi subdivs, single country

fn update_core_data_sql3 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = '**multi**',
    csubdiv_code = '**',
    country_code = t.country_code
    from 
        (select distinct a.id, country_code
        from ror.locations loc
        inner join src.admin_data a
        on loc.id = a.id
        where a.n_subdivs > 1
        and a.n_countries = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, multi countries

fn update_core_data_sql4 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = '**multi**',
    csubdiv_code = '**',
    country_code = '**'
    from src.admin_data a
    where c.id = a.id 
    and a.n_countries > 1;"#
}

use sqlx::{Pool, Postgres};
use crate::AppError;
use log::info;


async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let res = sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(res.rows_affected())
}


pub async fn create_ext_schema(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    execute_sql(r#"SET client_min_messages TO WARNING; 
    create schema if not exists ext;"#, pool).await
}


pub async fn load_orgs(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"drop table if exists ext.orgs;
            create table ext.orgs
    (
          id                varchar     not null primary key
        , ror_full_id       varchar     not null
        , ror_name          varchar     not null	
        , status            int         not null default 1
        , established       int         null
        , location          varchar     null
        , csubdiv_code      varchar     null
        , country_code      varchar     null
    );"#;

    execute_sql(sql, pool).await?;
    
    let sql = r#"insert into ext.orgs (id, ror_full_id, ror_name, 
            status, established, location, csubdiv_code, country_code)
            select id, ror_full_id, ror_name, 
            status, established, location, csubdiv_code, country_code
            from src.core_data;"#;
        
    let res = execute_sql(sql, pool).await?;
    info!("{} organisation records transferred to ext schema", res);

    Ok(())
        
}


pub async fn load_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"drop table if exists ext.names;
            create table ext.names
    (
          id                varchar     not null
        , name              varchar     not null  
        , name_to_match     varchar     null  
        , name_type         int         null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
        , lang_source       varchar     null
        , script_code       varchar     null
    );
    create index names_idx on ext.names(id);"#;

    execute_sql(sql, pool).await?;

    let sql = r#"insert into ext.names (id, name, name_to_match, name_type, 
            is_ror_name, lang_code, script_code)
            select id, value, lower(value), name_type, 
            is_ror_name, lang_code, script_code
            from src.names;"#;
        
    let res = execute_sql(sql, pool).await?;
    info!("{} organisation names transferred to ext schema", res);

    Ok(())
}


pub async fn load_rels(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"drop table if exists ext.relationships;
            create table ext.relationships
            (
                  id                varchar     not null
                , ror_name          varchar     not null
                , rel_type          int         not null
                , related_id        varchar     not null
                , related_name      varchar     not null
            );  
            create index relationships_idx on ext.relationships(id);"#;

    execute_sql(sql, pool).await?;
    
    let sql = r#"insert into ext.relationships (id, ror_name, rel_type, 
            related_id, related_name)
            select id, ror_name, rel_type, 
            related_id, related_name
            from src.relationships;"#;
        
    let res = execute_sql(sql, pool).await?;
    info!("{} relationship records transferred to ext schema", res);
        
    Ok(())
}


pub async fn load_types(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"drop table if exists ext.type;
            create table ext.type
            (
                  id                varchar     not null
                , ror_name          varchar     not null
                , org_type          int         not null
            );  
            create index type_idx on ext.type(id);"#;

    execute_sql(sql, pool).await?;
    
    
            let sql = r#"insert into ext.type(id, ror_name, org_type)
            select id, ror_name, org_type
            from src.type;"#;
        
    let res = execute_sql(sql, pool).await?;
    info!("{} type records transferred to ext schema", res);
        
    Ok(())
}


pub async fn load_locs(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"drop table if exists ext.locations;
            create table ext.locations
            (
                  id                varchar     not null
                , ror_name          varchar     not null
                , geonames_id       int         null
                , location          varchar     null	
                , lat               real        null
                , lng               real        null
                , cont_code         varchar     null
                , cont_name         varchar     null
                , country_code      varchar     null
                , country_name      varchar     null
                , csubdiv_code      varchar     null  
                , csubdiv_name      varchar     null	
            );
            create index locations_idx on ext.locations(id);"#;

    execute_sql(sql, pool).await?;
        
    let sql = r#"insert into ext.locations(id, ror_name, 
                geonames_id, location, lat, lng, cont_code, 
                cont_name, country_code, country_name, 
                csubdiv_code, csubdiv_name)
            select id, ror_name, 
                geonames_id, location, lat, lng, cont_code, 
                cont_name, country_code, country_name, 
                csubdiv_code, csubdiv_name
            from src.locations;"#;
        
    let res = execute_sql(sql, pool).await?;
    info!("{} location records transferred to ext schema", res);
    
    let sql = r#"drop table if exists ext.org_countries;
            create table ext.org_countries
            (
                  id                varchar     not null
                , country_code      varchar     null
            );
            create index countries_idx on ext.org_countries(id);"#;

    sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        
            let sql = r#"insert into ext.org_countries(id, country_code)
            select distinct id, country_code
            from ext.locations;"#;
        
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} country records created", res.rows_affected());

    Ok(())
}

pub async fn reset_postgres_messaging(pool: &Pool<Postgres>) -> Result<u64, AppError> {
    execute_sql(r#"SET client_min_messages TO NOTICE;"#, pool).await
}


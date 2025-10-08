use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::AppError;

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {

    execute_sql(get_schema_sql(), pool).await?;
    execute_sql(get_version_details_sql(), pool).await?;
    execute_sql(get_core_data_sql(), pool).await?;
    execute_sql(get_admin_data_sql(), pool).await?;
    execute_sql(get_names_sql(), pool).await?;
    execute_sql(get_locations_sql(), pool).await?;
    execute_sql(get_external_ids_sql(), pool).await?;
    execute_sql(get_links_sql(), pool).await?;
    execute_sql(get_org_types_sql(), pool).await?;
    execute_sql(get_relationships_sql(), pool).await?;
    execute_sql(get_domains_sql(), pool).await?;
    execute_sql(get_bare_ror_names_sql(), pool).await?;
    execute_sql(get_dup_names_sql(), pool).await?;
    execute_sql(get_message_sql(), pool).await?;

    Ok(())
}
    
async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

fn get_schema_sql <'a>() -> &'a str {
    r#"SET client_min_messages TO WARNING; 
    create schema if not exists ror;"#
}

fn get_version_details_sql <'a>() -> &'a str {
    r#"drop table if exists ror.version_details;
    create table ror.version_details
    (
          version           varchar     not null
        , data_date         varchar     not null
        , data_days         int         not null
        , import_datetime   timestamp   not null  default current_timestamp
    );"#
}

fn get_core_data_sql <'a>() -> &'a str {
    r#"drop table if exists ror.core_data;
    create table ror.core_data
    (
          id                varchar     not null primary key 
        , ror_full_id       varchar     not null  
        , status            varchar     not null
        , established       int         null
    );"#
}

fn get_admin_data_sql <'a>() -> &'a str {
    r#"drop table if exists ror.admin_data;
    create table ror.admin_data
    (
          id                varchar     not null primary key
        , created           date        not null
        , cr_schema         varchar     not null
        , last_modified     date        not null
        , lm_schema         varchar     not null  
    );"#
}

// ror names has an identity column to help resolve ambiguities 
// The column does not appear in the derived src.names table

fn get_names_sql <'a>() -> &'a str {
    r#"drop table if exists ror.names;
    create table ror.names
    (  
          ident             int         GENERATED ALWAYS AS IDENTITY
        , id                varchar     not null
        , value             varchar     not null  
        , name_type         varchar     not null
        , is_ror_name       bool        null
        , lang              varchar     null
    );
    create index src_names_idx on ror.names(id);"#
}

fn get_locations_sql <'a>() -> &'a str {
    r#"drop table if exists ror.locations;
    create table ror.locations
    (  
          id                varchar     not null
        , geonames_id       int         null
        , name              varchar     null	
        , lat               real        null
        , lng               real        null
        , continent_code    varchar     null
        , continent_name    varchar     null	    
        , country_code      varchar     null
        , country_name      varchar     null	
        , country_subdivision_code      varchar     null
        , country_subdivision_name      varchar     null	
    );
    create index src_locations_idx on ror.locations(id);"#
}

fn get_external_ids_sql <'a>() -> &'a str {
    r#"drop table if exists ror.external_ids;
    create table ror.external_ids
    (
          id                varchar     not null
        , id_type           varchar     not null
        , id_value          varchar     not null
        , is_preferred      bool        null
    );
    create index src_external_ids_idx on ror.external_ids(id);"#
}

fn get_links_sql <'a>() -> &'a str {
    r#"drop table if exists ror.links;
    create table ror.links
    (
          id                varchar	    not null
        , link_type         varchar     not null
        , value             varchar     not null
    );
    create index src_links_idx on ror.links(id);"#
}

fn get_org_types_sql <'a>() -> &'a str {
    r#"drop table if exists ror.type;
    create table ror.type
    (  
          id                varchar	    not null
        , org_type          varchar     not null
    ); 
    create index src_type_idx on ror.type(id);"#
}

fn get_relationships_sql <'a>() -> &'a str {
    r#"drop table if exists ror.relationships;
    create table ror.relationships
    (
          id                varchar     not null
        , rel_type          varchar     not null
        , related_id        varchar     not null
        , related_label     varchar     not null
    ); 
    create index src_relationships_idx on ror.relationships(id);"#
}

fn get_domains_sql <'a>() -> &'a str {
    r#"drop table if exists ror.domains;
    create table ror.domains
    (
          id                varchar     not null
        , value             varchar     not null
    );
    create index src_domains_idx on ror.domains(id);"#
}


fn get_bare_ror_names_sql <'a>() -> &'a str {
    r#"drop table if exists ror.bare_ror_names;
    create table ror.bare_ror_names
    (
          id                varchar     not null
        , value             varchar     not null
    );
    create index src_bare_ror_names_idx on ror.bare_ror_names(id);"#
}


fn get_dup_names_sql <'a>() -> &'a str {
    r#"drop table if exists ror.dup_names;
    create table ror.dup_names
    (
          ident             int         not null
        , id                varchar     not null
        , value             varchar     not null  
        , name_type         varchar     null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
        , fate              varchar     null
    );
    create index dup_names_idx on ror.dup_names(id);"#
}

fn get_message_sql <'a>() -> &'a str {
    r#"SET client_min_messages TO NOTICE;"#
}





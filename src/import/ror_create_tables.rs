use sqlx::{Pool, Postgres};
use crate::AppError;

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"SET client_min_messages TO WARNING; 
    create schema if not exists ror;

    drop table if exists ror.version_details;
    create table ror.version_details
    (
          version           varchar     not null
        , data_date         varchar     not null
        , data_days         int         not null
        , import_datetime   timestamp   not null  default current_timestamp
    );

    drop table if exists ror.core_data;
    create table ror.core_data
    (
          id                varchar     not null primary key 
        , ror_full_id       varchar     not null  
        , status            varchar     not null
        , established       int         null
    );

    drop table if exists ror.admin_data;
    create table ror.admin_data
    (
          id                varchar     not null primary key
        , created           date        not null
        , cr_schema         varchar     not null
        , last_modified     date        not null
        , lm_schema         varchar     not null  
    );

    drop table if exists ror.names;
    create table ror.names
    (  
          id                varchar     not null
        , value             varchar     not null  
        , name_type         varchar     not null
        , is_ror_name       bool        null
        , lang              varchar     null
    );
    create index src_names_idx on ror.names(id);


    drop table if exists ror.locations;
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
    create index src_locations_idx on ror.locations(id);

    drop table if exists ror.external_ids;
    create table ror.external_ids
    (
          id                varchar     not null
        , id_type           varchar     not null
        , id_value          varchar     not null
        , is_preferred      bool        null
    );
    create index src_external_ids_idx on ror.external_ids(id);

    drop table if exists ror.links;
    create table ror.links
    (
          id                varchar	    not null
        , link_type         varchar     not null
        , value             varchar     not null
    );
    create index src_links_idx on ror.links(id);

    drop table if exists ror.type;
    create table ror.type
    (  
          id                varchar	    not null
        , org_type          varchar     not null
    ); 
    create index src_type_idx on ror.type(id);

    drop table if exists ror.relationships;
    create table ror.relationships
    (
          id                varchar     not null
        , rel_type          varchar     not null
        , related_id        varchar     not null
        , related_label     varchar     not null
    ); 
    create index src_relationships_idx on ror.relationships(id);

    drop table if exists ror.domains;
    create table ror.domains
    (
          id                varchar     not null
        , value             varchar     not null
    );
    create index src_domains_idx on ror.domains(id);
    
    SET client_min_messages TO NOTICE;"#;

    sqlx::raw_sql(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(())
    
}



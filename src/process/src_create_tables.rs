use sqlx::{Pool, Postgres};
use crate::AppError;

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"
    SET client_min_messages TO WARNING; 
    create schema if not exists src;

    drop table if exists src.version_details;
    create table src.version_details
    (
          version           varchar     not null
        , data_date         varchar     not null
        , data_days         int         not null
        , process_datetime  timestamp   not null  default current_timestamp
    );

    drop table if exists src.core_data;
    create table src.core_data
    (
          id                varchar     not null primary key
        , ror_full_id       varchar     not null
        , ror_name          varchar     not null	
        , status            int         not null default 1
        , established       int         null
        , location          varchar     null
        , csubdiv_code      varchar     null
        , country_code      varchar     null
    );

    drop table if exists src.names;
    create table src.names
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         not null 
        , is_ror_name       bool        not null default false
        , lang_code         varchar     null
        , script_code       varchar     null
    );
    create index names_idx on src.names(id);
        
    drop table if exists src.dup_names;
    create table src.dup_names
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , dup_type          varchar     not null
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_idx on src.dup_names(id);

    drop table if exists src.dup_names_deleted;
    create table src.dup_names_deleted
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , dup_type          varchar     not null
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_deleted_idx on src.dup_names(id);

    drop table if exists src.locations;
    create table src.locations
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
    create index locations_idx on src.locations(id);

    drop table if exists src.external_ids;
    create table src.external_ids
    (
          id                varchar     not null
        , ror_name          varchar     not null	
        , id_type           int         not null
        , id_value          varchar     not null
        , is_preferred      bool        not null default false
    );
    create index external_ids_idx on src.external_ids(id);

    drop table if exists src.links;
    create table src.links
    (
          id                varchar     not null
        , ror_name          varchar     not null  	  
        , link_type         int         not null
        , link              varchar     not null
    );
    create index links_idx on src.links(id);

    drop table if exists src.type;
    create table src.type
    (
          id                varchar     not null
        , ror_name          varchar     not null
        , org_type          int         not null
    );  
    create index type_idx on src.type(id);

    drop table if exists src.relationships;
    create table src.relationships
    (
          id                varchar     not null
        , ror_name          varchar     not null
        , rel_type          int         not null
        , related_id        varchar     not null
        , related_name      varchar     not null
    );  
    create index relationships_idx on src.relationships(id);

    drop table if exists src.domains;
    create table src.domains
    (
          id                varchar     not null
        , ror_name          varchar     not null
        , domain            varchar     not null
    );
    create index domains_idx on src.domains(id);

    SET client_min_messages TO NOTICE;"#;

    sqlx::raw_sql(sql).execute(pool).await?;
    Ok(())
    
}


pub async fn create_admin_data_table(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"
    drop table if exists src.admin_data;
    create table src.admin_data
    (
          id                varchar     not null primary key
        , ror_name          varchar     not null	              
        , n_locs            int         not null default 0
        , n_labels          int         not null default 0
        , n_aliases         int         not null default 0
        , n_acronyms        int         not null default 0
        , n_names           int         not null default 0
        , n_names_wolc      int         not null default 0
        , n_nacro           int         not null default 0
        , n_nacro_wolc      int         not null default 0
        , is_company        bool        not null default false
        , n_types           int         not null default 0
        , n_isni            int         not null default 0
        , n_grid            int         not null default 0
        , n_fundref         int         not null default 0
        , n_wikidata        int         not null default 0
        , n_ext_ids         int         not null default 0
        , n_wikipedia       int         not null default 0
        , n_website         int         not null default 0
        , n_links           int         not null default 0
        , n_relrels         int         not null default 0
        , n_parrels         int         not null default 0
        , n_chrels          int         not null default 0
        , n_sucrels         int         not null default 0
        , n_predrels        int         not null default 0
        , n_doms            int         not null default 0
        , created           date        not null
        , cr_schema         varchar     not null
        , last_modified     date        not null
        , lm_schema         varchar     not null  
    );"#;

    sqlx::raw_sql(sql).execute(pool).await?;
    Ok(())
    
}


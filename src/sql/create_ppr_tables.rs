
pub fn get_sql<'a>() -> &'a str {

    r#"SET client_min_messages TO WARNING; 
    create schema if not exists ppr;
    
    drop table if exists ppr.version_details;
    create table ppr.version_details
    (
        version           varchar       not null
      , data_date         varchar       not null
      , data_days         int           not null
      , inc_wd            bool          not null
      , process_datetime  timestamptz   not null  default current_timestamp
    );
    
    drop table if exists ppr.core_data;
    create table ppr.core_data
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
    
    drop table if exists ppr.names;
    create table ppr.names
    (
        id                varchar     not null
      , value             varchar     not null  
      , name_type         int         not null 
      , is_ror_name       bool        not null default false
      , lang_code         varchar     null
      , script_code       varchar     null
    );
    create index names_idx on ppr.names(id);
    
    
    drop table if exists ppr.names_pad;
    create table ppr.names_pad
    (
        id                varchar     not null
      , original_name     varchar     not null    
      , name              varchar     null  
      , lang_code         varchar     null
      , script_code       varchar     null
      , latin             varchar     null
      , nonlatin          varchar     null
    );
    create index names_pad_idx on ppr.names_pad(id);
    
    
    drop table if exists ppr.locations;
    create table ppr.locations
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
    create index locations_idx on ppr.locations(id);
    
    drop table if exists ppr.external_ids;
    create table ppr.external_ids
    (
        id                varchar     not null
      , ror_name          varchar     not null	
      , id_type           int         not null
      , id_value          varchar     not null
      , is_preferred      bool        not null default false
    );
    create index external_ids_idx on ppr.external_ids(id);
    
    drop table if exists ppr.links;
    create table ppr.links
    (
        id                varchar     not null
      , ror_name          varchar     not null  	  
      , link_type         int         not null
      , link              varchar     not null
    );
    create index links_idx on ppr.links(id);
    
    drop table if exists ppr.type;
    create table ppr.type
    (
        id                varchar     not null
      , ror_name          varchar     not null
      , org_type          int         not null
    );  
    create index type_idx on ppr.type(id);
    
    drop table if exists ppr.relationships;
    create table ppr.relationships
    (
        id                varchar     not null
      , ror_name          varchar     not null
      , rel_type          int         not null
      , related_id        varchar     not null
      , related_name      varchar     not null
    );  
    create index relationships_idx on ppr.relationships(id);
    
    drop table if exists ppr.domains;
    create table ppr.domains
    (
        id                varchar     not null
      , ror_name          varchar     not null
      , domain            varchar     not null
    );
    create index domains_idx on ppr.domains(id);
    
    
    drop table if exists ppr.admin_data;
    create table ppr.admin_data
    (
        id                varchar     not null primary key
      , ror_name          varchar     not null	              
      , n_labels          int         not null default 0
      , n_aliases         int         not null default 0
      , n_acronyms        int         not null default 0
      , n_names           int         not null default 0
      , n_names_wolc      int         not null default 0
      , n_nacro           int         not null default 0
      , n_nacro_wolc      int         not null default 0
      , is_company        bool        not null default false
      , n_locs            int         not null default 0
      , n_subdivs         int         not null default 0
      , n_countries       int         not null default 0
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
    );

    drop table if exists ppr.withdrawn;
    create table ppr.withdrawn
    (
        id                int         Generated always as identity (START WITH 10001 INCREMENT BY 1) Primary Key
      , ror_id            varchar     not null
      , ror_name          varchar     not null	
      , established       int         null
      , location          varchar     null
      , csubdiv_code      varchar     null
      , country_code      varchar     null
      , successor_id      varchar     null
      , succ_name         varchar     null	
      , succ_status       int         null
      , succ_established  int         null
      , succ_location     varchar     null
      , succ_csubdiv_code varchar     null
      , succ_country_code varchar     null
    );
        
    SET client_min_messages TO NOTICE"#
}
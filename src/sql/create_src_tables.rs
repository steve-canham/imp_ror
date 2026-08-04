
pub fn get_sql<'a>() -> &'a str {

    r#"SET client_min_messages TO WARNING; 
    create schema if not exists src;
    
    drop table if exists src.version_details;
    create table src.version_details
    (
        version           varchar       not null
      , data_date         varchar       not null
      , data_days         int           not null
      , import_datetime   timestamptz   not null  default current_timestamp
    );
    
    drop table if exists src.core_data;
    create table src.core_data
    (
        id                varchar     not null primary key 
      , ror_full_id       varchar     not null  
      , status            varchar     not null
      , established       int         null
    );
    
    drop table if exists src.admin_data;
    create table src.admin_data
    (
        id                varchar     not null primary key
      , created           date        not null
      , cr_schema         varchar     not null
      , last_modified     date        not null
      , lm_schema         varchar     not null  
    );
    
    -- ror names has an identity column to help resolve ambiguities 
    -- The column does not appear in the derived ppr.names table
    -- rec.names also has an orig_value and value columns, as some 
    -- original values are changed (to correct errors, make them more consistent) 
    -- before further processing. The change_hist and change_type
    -- columns are used to record the nature of any changes

    drop table if exists src.names;
    create table src.names
    (  
        ident             int         GENERATED ALWAYS AS IDENTITY
      , id                varchar     not null
      , value             varchar     not null  
      , name_type         varchar     not null
      , is_ror_name       bool        null
      , lang              varchar     null
    );
    create index src_names_idx on src.names(id);
    
    drop table if exists rec.names;
    create table rec.names
    (  
        ident             int         not null
      , id                varchar     not null
      , orig_value        varchar     not null       -- as from src.names
      , display_value     varchar     not null       -- after basic repairs (if applied, after apost processing too and after company name processing)
      , lang_value        varchar     null           -- lower case and without punctuation (but still with spaces, ampersands)
      , match_value       varchar     null           -- lower case, no punctuation or ampersands, some stop words removed, hyphens standardised
      , script_value      varchar     null           -- spaces and hyphens removed - chars only
      , name_type         int         not null
      , is_ror_name       bool        not null
      , lang              varchar     null
      , der_lang          varchar     null
      , der_script        varchar     null
      , num_countries     int         null
      , country_code      varchar     null
      , changed           bool        not null  default false
      , change_type_id    varchar     null
      , change_type       varchar     null
    );
    create index rec_names_idx on src.names(id);

    -- if company name processing applied - redo match values
    
    drop table if exists src.locations;
    create table src.locations
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
    create index src_locations_idx on src.locations(id);
    
    drop table if exists src.countries;
    create table src.countries
    (
          id                varchar     not null
        , country_code      varchar     null
    );
    create index src_countries_idx on src.countries(id);

    
    drop table if exists src.external_ids;
    create table src.external_ids
    (
        id                varchar     not null
      , id_type           varchar     not null
      , id_value          varchar     not null
      , is_preferred      bool        null
    );
    create index src_external_ids_idx on src.external_ids(id);
    
    drop table if exists src.links;
    create table src.links
    (
        id                varchar	    not null
      , link_type         varchar     not null
      , value             varchar     not null
    );
    create index src_links_idx on src.links(id);
    
    drop table if exists src.type;
    create table src.type
    (  
        id                varchar	    not null
      , org_type          varchar       not null
    ); 
    create index src_type_idx on src.type(id);
    
    drop table if exists src.relationships;
    create table src.relationships
    (
        id                varchar     not null
      , rel_type          varchar     not null
      , related_id        varchar     not null
      , related_label     varchar     not null
    ); 
    create index src_relationships_idx on src.relationships(id);
    
    drop table if exists src.domains;
    create table src.domains
    (
        id                varchar     not null
      , value             varchar     not null
    );
    create index src_domains_idx on src.domains(id);
    
    drop table if exists rec.bare_ror_names;
    create table rec.bare_ror_names
    (
        id                varchar     not null
      , value             varchar     not null
    );
    create index rec_bare_ror_names_idx on rec.bare_ror_names(id);
    
    SET client_min_messages TO NOTICE;"#
}

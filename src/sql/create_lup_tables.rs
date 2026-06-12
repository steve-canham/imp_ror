
pub fn get_sql<'a>() -> &'a str {

    r#"SET client_min_messages TO WARNING; 
    create schema if not exists lup;
    
    drop table if exists lup.ror_status_types;
    create table lup.ror_status_types (
        id              int         not null primary key 
      , name            varchar
    );
    
    insert into lup.ror_status_types(id, name) 
    values (1, 'active'), (2, 'inactive'), (3, 'withdrawn');
    
    
    drop table if exists lup.ror_org_types;
    create table lup.ror_org_types (
        id              int         not null primary key 
      , name            varchar
    );
    
    insert into lup.ror_org_types(id, name) 
    values (100, 'government'), (200, 'education'), (300, 'healthcare'), 
    (400, 'company'), (500, 'nonprofit'), (600, 'funder'),
    (700, 'facility'), (800, 'archive'),  (900, 'other');

    
    drop table if exists lup.ror_name_types;
    create table lup.ror_name_types (
        id              int         not null primary key
      , name            varchar
    );
    
    insert into lup.ror_name_types(id, name) 
    values (5, 'label'), (7, 'alias'), (10, 'acronym');
    
    drop table if exists lup.ror_id_types;
    create table lup.ror_id_types (
        id              int         not null primary key
      , name            varchar
    );
    
    insert into lup.ror_id_types(id, name) 
    values (11, 'isni'), (12, 'wikidata'),
    (13, 'grid'), (14, 'fundref');

    
    drop table if exists lup.ror_link_types;
    create table lup.ror_link_types (
        id              int         not null primary key
      , name            varchar
    );
    
    insert into lup.ror_link_types(id, name) 
    values (21, 'wikipedia'), (22, 'website');

    
    drop table if exists lup.ror_rel_types;
    create table lup.ror_rel_types (
        id              int         not null primary key
      , name            varchar
    );
    
    insert into lup.ror_rel_types(id, name) 
    values (1, 'has parent'), (2, 'has child'), (3, 'is related to'),
        (4, 'has predecessor'), (5, 'has successor');
    
    SET client_min_messages TO NOTICE;"#
}
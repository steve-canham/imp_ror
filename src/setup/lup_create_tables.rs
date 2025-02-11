use sqlx::{Pool, Postgres};
use crate::AppError;

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"SET client_min_messages TO WARNING; 
    create schema if not exists lup;

    drop table if exists lup.ror_status_types;
    create table lup.ror_status_types (
        id                int         not null primary key 
        , name            varchar
    );

    drop table if exists lup.ror_org_types;
    create table lup.ror_org_types (
        id                int         not null primary key 
        , name            varchar
    );
    
    drop table if exists lup.ror_name_types;
    create table lup.ror_name_types (
        id                int         not null primary key
        , name            varchar
    );

    drop table if exists lup.ror_id_types;
    create table lup.ror_id_types (
        id                int         not null primary key
        , name            varchar
    );

    drop table if exists lup.ror_link_types;
    create table lup.ror_link_types (
        id                int         not null primary key
        , name            varchar
    );

    drop table if exists lup.ror_org_rels;
    create table lup.ror_org_rels (
        id                int         not null primary key
        , name            varchar
    );

    drop table if exists lup.countries;
    create table lup.countries (
        code              varchar     not null primary key
        , name            varchar
    );

    drop table if exists lup.lang_codes;
    create table lup.lang_codes (
        code              varchar     not null primary key
        , marc_code       varchar
        , name            varchar
        , source          varchar
    );

    drop table if exists lup.lang_scripts;
    create table lup.lang_scripts (
        code              varchar     not null primary key
        , unicode_name    varchar
        , iso_name        varchar
        , dir             varchar
        , chars           int
        , notes           varchar
        , hex_start       varchar
        , hex_end         varchar
        , ascii_start	  int
        , ascii_end       int
        , source          varchar
    );

    SET client_min_messages TO NOTICE;"#;

    sqlx::raw_sql(sql).execute(pool).await 
         .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(())
    
}


use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::AppError;

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {

    execute_sql(get_schema_sql(), pool).await?;
    execute_sql(get_version_summaries_sql(), pool).await?;
    execute_sql(get_count_distributions_sql(), pool).await?;
    execute_sql(get_ranked_distributions_sql(), pool).await?;
    execute_sql(get_attributes_summary_sql(), pool).await?;
    execute_sql(get_singletons_sql(), pool).await?;
    execute_sql(get_org_type_and_relationships_sql(), pool).await?;
    execute_sql(get_org_type_and_lang_code_sql(), pool).await?;
    execute_sql(get_message_sql(), pool).await?;

    Ok(())
}
 
async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

fn get_schema_sql <'a>() -> &'a str {
    r#"SET client_min_messages TO WARNING; 
    create schema if not exists smm;"#
}
    
fn get_version_summaries_sql <'a>() -> &'a str {
    r#"drop table if exists smm.version_summaries;
    create table smm.version_summaries
    (    
          vcode             varchar     not null primary key
        , vdate             date        not null
        , vdays             int         not null
        , num_orgs          int         null	
        , num_names         int         null	
        , num_types         int         null
        , num_links         int         null
        , num_ext_ids       int         null
        , num_rels          int         null
        , num_locations     int         null
        , num_domains       int         null
    );"#
}

fn get_count_distributions_sql <'a>() -> &'a str {
    r#"drop table if exists smm.count_distributions;
    create table smm.count_distributions
    (    
          vcode             varchar     not null
        , count_type        varchar     not null
        , count             int         null
        , num_of_orgs       int         null
        , pc_of_orgs        real        null
    );"#
}

fn get_ranked_distributions_sql <'a>() -> &'a str {
    r#"drop table if exists smm.ranked_distributions;
    create table smm.ranked_distributions
    (    
          vcode             varchar     not null
        , dist_type         int         not null 
        , rank              int         not null 
        , entity            varchar     null
        , number            int         null
        , pc_of_entities    real        null
        , pc_of_base_set    real        null
    );"#
}

fn get_attributes_summary_sql <'a>() -> &'a str {
    r#"drop table if exists smm.attributes_summary;
    create table smm.attributes_summary
    (    
          vcode             varchar     not null
        , att_type          int         null
        , att_name          varchar     null
        , id                int         null
        , name              varchar     null
        , number_atts       int         null
        , pc_of_atts        real        null
        , number_orgs       int         null
        , pc_of_orgs        real        null        
    );"#
}

fn get_singletons_sql <'a>() -> &'a str {
    r#"drop table if exists smm.singletons;
    create table smm.singletons
    (    
          vcode             varchar     not null
        , id                varchar     not null
        , description       varchar     null
        , number            int         null
        , pc                real        null
    );"#
}

fn get_org_type_and_relationships_sql <'a>() -> &'a str {
    r#"drop table if exists smm.org_type_and_relationships;
    create table smm.org_type_and_relationships
    (    
          vcode             varchar     not null
        , org_type          varchar     null
        , rel_type          varchar     null
        , num_links         int         null
        , num_orgs          int         null
        , num_orgs_total    int         null
        , num_orgs_pc       real        null
    );"#
}


fn get_org_type_and_lang_code_sql <'a>() -> &'a str {
    r#"drop table if exists smm.org_type_and_lang_code;
    create table smm.org_type_and_lang_code
    (    
          vcode             varchar     not null
        , org_type          varchar     null
        , name_type         varchar     null
        , names_num         int         null
        , names_wlc         int         null
        , names_wolc        int         null
        , names_wlc_pc      real        null
        , names_wolc_pc     real        null
    );"#
}

fn get_message_sql <'a>() -> &'a str {
    r#"SET client_min_messages TO NOTICE;"#
}









use sqlx::{Postgres, Pool};

use super::src_record_structs::{SrcCoreData, SrcRelationship, SrcExternalId, 
                            SrcName, SrcLocation, SrcLink, SrcType, SrcAdminData};

pub async fn fetch_src_record_num (table_name: &str, pool: &Pool<Postgres>) -> i64 {
    let sql = "SELECT COUNT(*) FROM src.".to_owned() + table_name;
    sqlx::query_scalar(&sql).fetch_one(pool).await.unwrap()
}

pub async fn fetch_src_first_record_id (pool: &Pool<Postgres>) -> String {
    let sql = "SELECT id FROM src.core_data order by id LIMIT 1;".to_owned();
    sqlx::query_scalar(&sql).fetch_one(pool).await.unwrap()
}

pub async fn fetch_src_last_record_id (pool: &Pool<Postgres>) -> String {
    let sql = "SELECT id FROM src.core_data order by id desc LIMIT 1;".to_owned();
    sqlx::query_scalar(&sql).fetch_one(pool).await.unwrap()
}

pub async fn fetch_src_core_data_record (id: &str, pool: &Pool<Postgres>) -> SrcCoreData {
    let sql: &str  = "select * from src.core_data where id = $1";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_one(pool).await.unwrap()
}

pub async fn fetch_src_admin_data_record (id: &str, pool: &Pool<Postgres>) -> SrcAdminData {
    let sql: &str  = "select * from src.admin_data where id = $1";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_one(pool).await.unwrap()
}

pub async fn fetch_src_relationship_records (id: &str, pool: &Pool<Postgres>) -> Vec<SrcRelationship> {
    let sql: &str  = "select * from src.relationships where id = $1 order by related_id";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_all(pool).await.unwrap()
}

pub async fn fetch_src_external_id_records (id: &str, pool: &Pool<Postgres>) -> Vec<SrcExternalId> {
    let sql: &str  = "select * from src.external_ids where id = $1 order by id_value";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_all(pool).await.unwrap()
}

pub async fn fetch_src_location_records (id: &str, pool: &Pool<Postgres>) -> Vec<SrcLocation> {
    let sql: &str  = "select * from src.locations where id = $1 order by geonames_id";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_all(pool).await.unwrap()
}

pub async fn fetch_src_link_records (id: &str, pool: &Pool<Postgres>) -> Vec<SrcLink> {
    let sql: &str  = "select * from src.links where id = $1 order by value";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_all(pool).await.unwrap()
}

pub async fn fetch_src_type_records (id: &str, pool: &Pool<Postgres>) -> Vec<SrcType> {
    let sql: &str  = "select * from src.type where id = $1 order by type";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_all(pool).await.unwrap()
}

pub async fn fetch_src_name_records (id: &str, pool: &Pool<Postgres>) -> Vec<SrcName> {
    let sql: &str  = "select * from src.names where id = $1 order by value";
    sqlx::query_as(sql)
        .bind(id)
        .fetch_all(pool).await.unwrap()
}

use sqlx::{Postgres, Pool};

use super::ppr_record_structs::{PprCoreData, PprRelationship, PprExternalId, 
    PprName, PprLocation, PprLink, PprType, PprAdminData};

pub async fn fetch_ppr_record_num (table_name: &str, pool: &Pool<Postgres>) -> i64 {
let sql = "SELECT COUNT(*) FROM ppr.".to_owned() + table_name;
sqlx::query_scalar(&sql).fetch_one(pool).await.unwrap()
}

pub async fn fetch_ppr_first_record_id (pool: &Pool<Postgres>) -> String {
let sql = "SELECT id FROM ppr.core_data order by id LIMIT 1;".to_owned();
sqlx::query_scalar(&sql).fetch_one(pool).await.unwrap()
}

pub async fn fetch_ppr_last_record_id (pool: &Pool<Postgres>) -> String {
let sql = "SELECT id FROM ppr.core_data order by id desc LIMIT 1;".to_owned();
sqlx::query_scalar(&sql).fetch_one(pool).await.unwrap()
}

pub async fn fetch_ppr_core_data_record (id: &str, pool: &Pool<Postgres>) -> PprCoreData {
let sql: &str  = "select * from ppr.core_data where id = $1";
sqlx::query_as(sql)
.bind(id)
.fetch_one(pool).await.unwrap()
}

pub async fn fetch_ppr_admin_data_record (id: &str, pool: &Pool<Postgres>) -> PprAdminData {
let sql: &str  = "select * from ppr.admin_data where id = $1";
sqlx::query_as(sql)
.bind(id)
.fetch_one(pool).await.unwrap()
}

pub async fn fetch_ppr_relationship_records (id: &str, pool: &Pool<Postgres>) -> Vec<PprRelationship> {
let sql: &str  = "select * from ppr.relationships where id = $1 order by related_id";
sqlx::query_as(sql)
.bind(id)
.fetch_all(pool).await.unwrap()
}

pub async fn fetch_ppr_external_id_records (id: &str, pool: &Pool<Postgres>) -> Vec<PprExternalId> {
let sql: &str  = "select * from ppr.external_ids where id = $1 order by id_value";
sqlx::query_as(sql)
.bind(id)
.fetch_all(pool).await.unwrap()
}

pub async fn fetch_ppr_location_records (id: &str, pool: &Pool<Postgres>) -> Vec<PprLocation> {
let sql: &str  = "select * from ppr.locations where id = $1 order by geonames_id";
sqlx::query_as(sql)
.bind(id)
.fetch_all(pool).await.unwrap()
}

pub async fn fetch_ppr_link_records (id: &str, pool: &Pool<Postgres>) -> Vec<PprLink> {
let sql: &str  = "select * from ppr.links where id = $1 order by link";
sqlx::query_as(sql)
.bind(id)
.fetch_all(pool).await.unwrap()
}

pub async fn fetch_ppr_type_records (id: &str, pool: &Pool<Postgres>) -> Vec<PprType> {
let sql: &str  = "select * from ppr.type where id = $1 order by type";
sqlx::query_as(sql)
.bind(id)
.fetch_all(pool).await.unwrap()
}

pub async fn fetch_ppr_name_records (id: &str, pool: &Pool<Postgres>) -> Vec<PprName> {
let sql: &str  = "select * from ppr.names where id = $1 order by value";
sqlx::query_as(sql)
.bind(id)
.fetch_all(pool).await.unwrap()
}

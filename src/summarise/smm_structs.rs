use sqlx::{Postgres, Pool, postgres::PgQueryResult};
use crate::AppError;

#[derive(sqlx::FromRow)]
pub struct FileParams {
    pub vcode: String,
    pub vdate_as_string: String,
    pub vdays: i32,
    pub inc_wd: bool,
}

#[derive(sqlx::FromRow)]
pub struct TypeRow {
    pub vcode: String,
    pub cat_id: i32,
    pub cat_name: String,
    pub number_cat: i64,
    pub pc_of_atts: f64,
    pub number_orgs: i64,
    pub pc_of_orgs: f64,
}

#[derive(sqlx::FromRow)]
pub struct DistribRow {
  pub vcode: String,
  pub count: i32,
  pub num_of_orgs: i64,
  pub pc_of_orgs: f64,
}

#[derive(sqlx::FromRow)]
pub struct RankedRow {
  pub vcode: String,
  pub entity: String,
  pub number: i64,
  pub pc_of_entities: f64,
  pub pc_of_base_set: f64,
}


#[derive(sqlx::FromRow)]
pub struct OrgRow {
    pub org_type_id: i32,
    pub name: String,
    pub org_num: i64,
}

pub struct Singletons {
    pub vcodes: Vec<String>,
    pub inc_wds: Vec<bool>,
    pub ids: Vec<i32>,
    pub names: Vec<String>,
    pub descriptions: Vec<String>,
    pub numbers: Vec<i64>,
    pub pcs: Vec<Option<f64>>,
}

impl Singletons {

    pub fn new(vsize: usize) -> Self {
        Singletons {
            vcodes: Vec::with_capacity(vsize),
            inc_wds: Vec::with_capacity(vsize),
            ids: Vec::with_capacity(vsize),
            names: Vec::with_capacity(vsize),
            descriptions: Vec::with_capacity(vsize),
            numbers: Vec::with_capacity(vsize),
            pcs: Vec::with_capacity(vsize),
        }
    }

    pub fn add(&mut self, vcode: &String, inc_wd: bool, id: i32, name: &str, description: &str, number: i64, pc: Option<f64>) {
        self.vcodes.push(vcode.to_string());
        self.inc_wds.push(inc_wd);
        self.ids.push(id);
        self.names.push(name.to_string());
        self.descriptions.push(description.to_string());
        self.numbers.push(number);
        self.pcs.push(pc);
    }

    pub async fn store(&self, pool : &Pool<Postgres>)  -> Result<PgQueryResult, AppError> {

        let sql = format!(r#"INSERT INTO smm.singletons (vcode, inc_wd, id, name, description, number, pc)
            SELECT * FROM UNNEST($1::text[], $2::bool[], $3::int[], $4::text[], $5::text[], $6::int[], $7::real[])"#);
        sqlx::query(&sql)
        .bind(&self.vcodes).bind(&self.inc_wds).bind(&self.ids)
        .bind(&self.names).bind(&self.descriptions)
        .bind(&self.numbers).bind(&self.pcs)
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
    }


}

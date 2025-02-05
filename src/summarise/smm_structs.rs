#[derive(sqlx::FromRow)]
pub struct FileParams {
    pub vcode: String,
    pub vdate_as_string: String,
    pub vdays: i32
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
pub struct TypeRow {
    pub vcode: String,
    pub id: i32,
    pub name: String,
    pub number_atts: i64,
    pub pc_of_atts: f64,
    pub number_orgs: i64,
    pub pc_of_orgs: f64,
}

#[derive(sqlx::FromRow)]
pub struct OrgRow {
    pub type_id: i32, 
    pub name: String,
    pub org_num: i64, 
}


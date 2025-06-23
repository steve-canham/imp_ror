use chrono::NaiveDate;

#[derive(sqlx::FromRow)]
pub struct VSummary {
    pub vdate: NaiveDate,
    pub vdays: i32,
    pub num_orgs: i32,
    pub num_names: i32,
    pub num_types: i32,
    pub num_links: i32,
    pub num_ext_ids: i32,
    pub num_rels: i32,
    pub num_locations: i32,
    pub num_domains: i32,
}

#[derive(sqlx::FromRow)]
pub struct TypeRow {
    pub name: String,
    pub number_atts: i32,
    pub pc_of_atts: f32,
    pub number_orgs: i32,
    pub pc_of_orgs: f32,
}

#[derive(sqlx::FromRow)]
pub struct DistribRow {
  pub count: i32,
  pub num_of_orgs: i32,
  pub pc_of_orgs: f32,
}

#[derive(sqlx::FromRow)]
pub struct RankedRow {
  pub entity: String,
  pub number: i32,
  pub pc_of_entities: f32,
  pub pc_of_base_set: f32,
}

#[derive(sqlx::FromRow)]
pub struct SingletonRow {
    pub id: String,
    pub description: String,
    pub number: i32,
    pub pc: Option<f32>,
}

#[derive(sqlx::FromRow)]
pub struct Singleton {
    pub description: String,
    pub number: i32,
    pub pc: Option<f32>,
}

#[derive(sqlx::FromRow)]
pub struct OrgAndRel{
    pub org_type: String,
    pub rel_type: String,
    pub num_links: i32,
    pub num_orgs: i32,
    pub num_orgs_pc: f32,
}

#[derive(sqlx::FromRow)]
pub struct OrgAndLangCode{
    pub org_type: String,
    pub name_type: String,
    pub names_num: i32,
    pub names_wolc: i32,
    pub names_wolc_pc: f32,
}






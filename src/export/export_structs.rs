use serde::Serialize;
use chrono::NaiveDate;

#[derive(sqlx::FromRow)]
pub struct VSummary {
    pub vdate: NaiveDate,
    pub vdays: i32,
    pub num_recs: i32,
    pub num_active: i32,
    pub num_inactive: i32,
    pub num_withdrawn: i32,
    pub num_denom: i32,
    pub num_names: i32,
    pub num_types: i32,
    pub num_links: i32,
    pub num_ext_ids: i32,
    pub num_rels: i32,
    pub num_locations: i32,
    pub num_domains: i32,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct CSVSummaryRow {
    pub vcode: String,
    pub inc_wd: bool,
    pub vdate: String,
    pub vdays: i32,
    pub num_recs: i32,
    pub num_active: i32,
    pub num_inactive: i32,
    pub num_withdrawn: i32,
    pub num_denom: i32,
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
    pub cat_name: String,
    pub number_cat: i32,
    pub pc_of_atts: f32,
    pub number_orgs: i32,
    pub pc_of_orgs: f32,
}


#[derive(sqlx::FromRow, Serialize)]
pub struct CSVAttributeRow {
    pub vcode: String,
    pub inc_wd: bool,
    pub vdate: String,
    pub vdays: i32,
    pub att_id: i32,
    pub att_name: String,
    pub cat_id: i32,
    pub cat_name: String,
    pub number_cat: i32,
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


#[derive(sqlx::FromRow, Serialize)]
pub struct CSVDistribRow {
  pub vcode: String,
  pub inc_wd: bool, 
  pub vdate: String,
  pub vdays: i32,
  pub count_id: i32,
  pub count_name: String,
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

#[derive(sqlx::FromRow, Serialize)]
pub struct CSVRankedRow {
  pub vcode: String,
  pub inc_wd: bool,
  pub vdate: String,
  pub vdays: i32,
  pub dist_id: i32,
  pub dist_name: String,
  pub rank: i32,
  pub entity: String,
  pub number: i32,
  pub pc_of_entities: f32,
  pub pc_of_base_set: f32,
}

#[derive(sqlx::FromRow)]
pub struct SingletonRow {
    pub name: String,
    pub description: String,
    pub number: i32,
    pub pc: Option<f32>,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct CSVSingletonRow {
    pub vcode: String,
    pub inc_wd: bool,
    pub vdate: String,
    pub vdays: i32,
    pub id: i32,
    pub name: String,
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

#[derive(sqlx::FromRow, Serialize)]
pub struct CSVOrgAndRelRow{
    pub vcode: String,
    pub inc_wd: bool,
    pub vdate: String,
    pub vdays: i32,
    pub rel_type_id: i32,
    pub rel_type: String,
    pub org_type_id: i32,
    pub org_type: String,
    pub num_links: i32,
    pub num_orgs: i32,
    pub num_orgs_total: i32,
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

#[derive(sqlx::FromRow, Serialize)]
pub struct CSVOrgAndLangRow{
    pub vcode: String,
    pub inc_wd: bool,
    pub vdate: String,
    pub vdays: i32,
    pub name_type_id: i32,
    pub name_type: String,
    pub org_type_id: i32,
    pub org_type: String,
    pub names_num: i32,
    pub names_wlc: i32,
    pub names_wolc: i32,
    pub names_wlc_pc: f32,
    pub names_wolc_pc: f32,
}






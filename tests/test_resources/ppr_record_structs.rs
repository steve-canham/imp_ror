use sqlx::FromRow;
use chrono::NaiveDate;

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprCoreData {
    pub ror_name: String, 
    pub ror_full_id: String,
    pub status: i32,
    pub established: Option<i32>,
    pub location: Option<String>,
    pub csubdiv_code: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprAdminData {
    pub ror_name: String, 
    pub n_locs: i32,
    pub n_labels: i32,
    pub n_aliases: i32,
    pub n_acronyms: i32,
    pub n_names: i32,
    pub n_names_wolc: i32,
    pub n_nacro: i32,
    pub n_nacro_wolc: i32,
    pub is_company: bool,
    pub n_isni: i32,
    pub n_grid: i32,
    pub n_fundref: i32,
    pub n_wikidata: i32,
    pub n_ext_ids: i32,
    pub n_wikipedia: i32,
    pub n_website: i32,
    pub n_links: i32,
    pub n_types: i32,
    pub n_relrels: i32,
    pub n_parrels: i32,
    pub n_chrels: i32,
    pub n_sucrels: i32,
    pub n_predrels: i32,
    pub n_doms: i32,
    pub created: NaiveDate,
    pub cr_schema: String,
    pub last_modified: NaiveDate,
    pub lm_schema: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprRelationship {
    pub ror_name: String, 
    pub rel_type: i32,
    pub related_id: String,
    pub related_name: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprExternalId {
    pub ror_name: String, 
    pub id_type: i32,
    pub id_value: String,
    pub is_preferred: bool,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprLink {
    pub ror_name: String, 
    pub link_type: i32,
    pub link: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprType {
    pub ror_name: String, 
    pub org_type: i32, 
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprLocation {
    pub ror_name: String, 
    pub geonames_id: i32,
    pub location: String,
    pub lat: Option<f32>,
    pub lng: Option<f32>,
    pub cont_code: Option<String>,
    pub cont_name: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub csubdiv_code: Option<String>,
    pub csubdiv_name: Option<String>,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PprName {
    pub value: String,
    pub name_type: i32, 
    pub is_ror_name: bool,
    pub lang_code: Option<String>,
    pub script_code: Option<String>,
}

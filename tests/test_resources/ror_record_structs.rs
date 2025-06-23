use sqlx::FromRow;
use chrono::NaiveDate;

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorCoreData {
    pub ror_full_id: String,
    pub status: String,
    pub established: Option<i32>,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorAdminData {
    pub created: NaiveDate,
    pub cr_schema: String,
    pub last_modified: NaiveDate,
    pub lm_schema: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorRelationship {
    pub rel_type: String,
    pub related_id: String,
    pub related_label: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorExternalId {
    pub id_type: String,
    pub id_value: String,
    pub is_preferred: Option<bool>,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorLink {
    pub link_type: String,
    pub value: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorType {
    pub org_type: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorLocation {
    pub geonames_id: i32,
    pub name: String,
    pub lat: Option<f32>,
    pub lng: Option<f32>,
    pub continent_code: Option<String>,
    pub continent_name: Option<String>,
    pub country_code: String,
    pub country_name: String,
    pub country_subdivision_code: Option<String>,
    pub country_subdivision_name: Option<String>,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RorName {
    pub value: String,
    pub name_type: String,
    pub is_ror_name: Option<bool>,
    pub lang: Option<String>,
}







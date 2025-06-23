use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
    pub struct RorRecord {
        pub id: String,
        pub status: String,
        pub established: Option<i16>,
        pub names: Vec<Name>,
        pub types: Vec<String>,
        pub locations: Vec<Location>,
        pub external_ids: Option<Vec<ExternalId>>,
        pub links: Option<Vec<Link>>,
        pub relationships: Option<Vec<Relationship>>,
        pub domains: Option<Vec<String>>,
        pub admin: Admin,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Name {
        pub value: String,
        pub lang: Option<String>,
        pub types: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Location {
        pub geonames_id: i64,
        pub geonames_details: GeoDetails,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GeoDetails {
        pub continent_code : Option<String>,
        pub continent_name : Option<String>,
        pub country_code: String,
        pub country_name: String,
        pub country_subdivision_code : Option<String>,
        pub country_subdivision_name : Option<String>,
        pub lat: f64,
        pub lng: f64,
        pub name: String,
    } 

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ExternalId {
        #[serde(rename(deserialize = "type"))]
        pub id_type: String,
        pub all: Vec<String>,
        pub preferred: Option<String>,
    } 


    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Link {
        #[serde(rename(deserialize = "type"))]
        pub link_type: String,
        pub value: String,
    }


    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Relationship {
        #[serde(rename(deserialize = "type"))]
        pub rel_type: String,
        pub label: String,
        pub id: String,
    }

    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Admin {
        pub created: DateSchema,
        pub last_modified: DateSchema,
    }


    #[derive(Serialize, Deserialize, Debug)]
    pub struct DateSchema {
        pub date: String,
        pub schema_version: String,
    }


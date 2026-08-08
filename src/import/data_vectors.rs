use chrono::NaiveDate;
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::import::json_models::RorRecord;
use crate::AppError;

// vectors to hold column values, 100 at a time

pub struct CoreDataVecs {
    pub db_ids: Vec<String>,
    pub ror_ids: Vec<String>,
    pub statuses: Vec<String>,
    pub estabs: Vec<Option<i16>>,
    pub created_dates: Vec<NaiveDate>,
    pub created_vs: Vec<String>,
    pub lastmod_dates: Vec<NaiveDate>,
    pub lastmod_vs: Vec<String>,
}

impl CoreDataVecs{
    pub fn new(vsize: usize) -> Self {
        CoreDataVecs { 
            db_ids: Vec::with_capacity(vsize),
            ror_ids: Vec::with_capacity(vsize),
            statuses: Vec::with_capacity(vsize),
            estabs: Vec::with_capacity(vsize),
            created_dates: Vec::with_capacity(vsize),
            created_vs: Vec::with_capacity(vsize),
            lastmod_dates: Vec::with_capacity(vsize),
            lastmod_vs: Vec::with_capacity(vsize),
        }
    }

    pub fn add_core_data(&mut self, r: &RorRecord, db_id: &String) 
    {
        self.db_ids.push(db_id.clone());

        self.ror_ids.push(r.id.clone());
        self.statuses.push(r.status.clone());
        self.estabs.push(r.established.clone());

        let cr_date = NaiveDate::parse_from_str(&r.admin.created.date, "%Y-%m-%d").unwrap();
        let lm_date = NaiveDate::parse_from_str(&r.admin.last_modified.date, "%Y-%m-%d").unwrap();
            
        self.created_dates.push(cr_date);
        self.created_vs.push(r.admin.created.schema_version.clone());
        self.lastmod_dates.push(lm_date);
        self.lastmod_vs.push(r.admin.last_modified.schema_version.clone());
    }


    pub async fn store_data(&self, pool : &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
        // Do the core data.
        
        let sql = r#"INSERT INTO src.core_data (id, ror_full_id, status, established) 
            SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::int[])"#;
        sqlx::query(sql)
        .bind(&self.db_ids).bind(&self.ror_ids).bind(&self.statuses).bind(&self.estabs)
        .execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        
        // Do the admin data.
        
        let sql = r#"INSERT INTO src.admin_data (id, created, cr_schema, last_modified, lm_schema) 
            SELECT * FROM UNNEST($1::text[], $2::timestamp[], $3::text[], $4::timestamp[], $5::text[])"#;
        sqlx::query(sql)
        .bind(&self.db_ids).bind(&self.created_dates)
        .bind(&self.created_vs).bind(&self.lastmod_dates).bind(&self.lastmod_vs)
        .execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))
    }
}

pub struct RorName {
    pub db_id: String,
    pub name: String,
    pub name_type: String,
    pub is_ror:bool,
    pub lang: Option<String>,
}

pub struct RequiredDataVecs {
    pub name_db_ids: Vec<String>,
    pub names: Vec<String>,
    pub name_types: Vec<String>,
    pub is_rors:Vec<bool>,
    pub langs: Vec<Option<String>>,

    pub type_db_ids: Vec<String>,
    pub org_types: Vec<String>,

    pub loc_db_ids: Vec<String>,
    pub gn_ids: Vec<i64>,
    pub gn_names: Vec<String>,
    pub lats: Vec<f64>,
    pub lngs: Vec<f64>,
    pub cont_codes: Vec<Option<String>>,
    pub cont_names: Vec<Option<String>>,
    pub cy_codes: Vec<String>,
    pub cy_names: Vec<String>,
    pub cy_subdiv_codes: Vec<Option<String>>,
    pub cy_subdiv_names: Vec<Option<String>>,
}


impl RequiredDataVecs{
    pub fn new(vsize: usize) -> Self {
        RequiredDataVecs { 
            name_db_ids: Vec::with_capacity(vsize),
            names: Vec::with_capacity(vsize),
            name_types: Vec::with_capacity(vsize),
            is_rors: Vec::with_capacity(vsize),
            langs: Vec::with_capacity(vsize),

            type_db_ids:Vec::with_capacity(vsize),
            org_types: Vec::with_capacity(vsize),

            loc_db_ids:Vec::with_capacity(vsize),
            gn_ids: Vec::with_capacity(vsize),
            gn_names: Vec::with_capacity(vsize),
            lats: Vec::with_capacity(vsize),
            lngs: Vec::with_capacity(vsize),
            cont_codes: Vec::with_capacity(vsize),
            cont_names: Vec::with_capacity(vsize),
            cy_codes: Vec::with_capacity(vsize),
            cy_names: Vec::with_capacity(vsize),
            cy_subdiv_codes: Vec::with_capacity(vsize),
            cy_subdiv_names: Vec::with_capacity(vsize),
        }
    }

    pub async fn add_name_data(&mut self, r: &RorRecord, db_id: &String, pool : &Pool<Postgres>) -> Result<(), AppError> 
    {
        if r.names.len() > 0 {
            let mut org_ror_name = 0;
            for name in r.names.iter()
            {
                let mut rn = RorName {
                    db_id: db_id.clone(),
                    name: name.value.clone(),
                    name_type: "".to_string(),
                    is_ror: false,
                    lang: name.lang.clone(),
                };

                if name.types.len() > 0 {
                    if name.types.len() == 1 {
                        if name.types[0] == "ror_display" {    // rare but seems to occur in about 100 cases
                            rn.name_type = "label".to_string();
                            rn.is_ror = true;
                            store_strange_ror_record(&db_id, &name.value, 1, pool).await?;
                            org_ror_name += 1;
                        }
                        else      // much more commonly
                        {
                            rn.name_type = name.types[0].clone();
                        }
                    }
                    else if name.types.len() == 2 {   // the other usual situation

                        // One would normally be 'ror_display', the other the name type
                        // Check not both 'ror_display' or that none of them are

                        let zero_is_ror = name.types[0].as_str() == "ror_display";
                        let one_is_ror = name.types[1].as_str() == "ror_display";
                        
                        if zero_is_ror && one_is_ror {
                            store_strange_ror_record(&db_id, &name.value, 2, pool).await?;
                        }
                        else if !zero_is_ror && !one_is_ror {   // A pair of type designations 
                            
                            let t1 = name.types[0].as_str();
                            let t2 = name.types[1].as_str();
                            let (selected_type, other_type) = obtain_name_type(&name.value, t1, t2);
                            rn.name_type = selected_type.clone();
                            let info = format!("{}: {}, {}", &name.value, selected_type, other_type);
                            store_strange_ror_record(&db_id, &info, 3, pool).await?;
                        }
                        else {   // a single name type that is also a ror_name
                            
                            if zero_is_ror {
                                rn.name_type = name.types[1].clone();
                            }
                            if one_is_ror {
                                rn.name_type = name.types[0].clone();
                            }
                            rn.is_ror = true;
                            org_ror_name += 1;
                        }
                    }
                    else if name.types.len() == 3 { // almost always a ror_name indicator plus two namne types

                        let mut is_ror_name = false;
                        let mut t1 = "";
                        let mut t2 = "";
                        for t in &name.types {
                            if t == "ror_display" {
                                is_ror_name = true;
                                org_ror_name += 1;
                            }
                            else {
                                if t1 == "" {
                                    t1 = t.as_str();
                                }
                                else {
                                    t2 = t.as_str();
                                }
                            }
                        }

                        if !is_ror_name {
                            // very strange!
                            let info = format!("{}: {}, {}", &name.value, t1, t2);
                            store_strange_ror_record(&db_id, &info, 4, pool).await?;
                        }
                        else {
                            let (selected_type, other_type) = obtain_name_type(&name.value, t1, t2);
                            rn.is_ror = true;
                            rn.name_type = selected_type.clone();
                            let info = format!("{}: {}, {}", &name.value, selected_type, other_type);
                            store_strange_ror_record(&db_id, &info, 5, pool).await?;
                        }
                    }
                    else {    // 4 or more name types!
                        
                    }
                }

                self.name_db_ids.push(rn.db_id);
                self.names.push(rn.name); 
                self.name_types.push(rn.name_type);
                self.is_rors.push(rn.is_ror); 
                self.langs.push(rn.lang); 
            }

            if org_ror_name == 0 {   // store the fact that no name is identified as a ror name in the data
                store_strange_ror_record(&db_id, "no ROR name", 6, pool).await?;  
            }
            if org_ror_name > 1 {   // store the fact that multiple names are identified as a ror name in the data
                store_strange_ror_record(&db_id, "More than one ROR name", 7, pool).await?;  
            }
        }
        else {   // store the fact that no names at alll are listed
            store_strange_ror_record(&db_id, "no names at all!", 8, pool).await?;  
        }
        
        Ok(())
    }

    
    pub fn add_locs_and_types_data(&mut self, r: &RorRecord, db_id: &String) 
    {

        if r.types.len() > 0 {
            // types is a vector of strings
            for item in r.types.iter()
            {
                self.type_db_ids.push(db_id.clone());
                self.org_types.push(item.clone());
            }
        }

        if r.locations.len() > 0 {
            for loc in r.locations.iter()
            {
                self.loc_db_ids.push(db_id.clone());
                self.gn_ids.push(loc.geonames_id.clone());
                self.gn_names.push(loc.geonames_details.name.clone());
                self.lats.push(loc.geonames_details.lat.clone());
                self.lngs.push(loc.geonames_details.lng.clone());
                self.cont_codes.push(loc.geonames_details.continent_code.clone());
                self.cont_names.push(loc.geonames_details.continent_name.clone());
                self.cy_codes.push(loc.geonames_details.country_code.clone());
                self.cy_names.push(loc.geonames_details.country_name.clone());
                self.cy_subdiv_codes.push(loc.geonames_details.country_subdivision_code.clone());
                self.cy_subdiv_names.push(loc.geonames_details.country_subdivision_name.clone());
            }
        }

    }

    pub async fn store_data(&self, pool : &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
        
        // Do the name data.
        
        let sql = r#"INSERT INTO src.names (id, value, name_type, is_ror_name, lang) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::bool[], $5::text[])"#;

        sqlx::query(sql)
                .bind(&self.name_db_ids).bind(&self.names)
                .bind(&self.name_types).bind(&self.is_rors).bind(&self.langs)
                .execute(pool)
                .await.map_err(|e| AppError::SqlxError(e, "Storing src names".to_string()))?;

        // Do the type data.

        let sql = r#"INSERT INTO src.type (id, org_type) 
        SELECT * FROM UNNEST($1::text[], $2::text[])"#;

        sqlx::query(sql)
                .bind(&self.type_db_ids).bind(&self.org_types)
                .execute(pool)
                .await.map_err(|e| AppError::SqlxError(e, "Storing src types".to_string()))?;

        // Do the location data.

        let sql = r#"INSERT INTO src.locations (id, geonames_id, name, lat, lng, 
        continent_code, continent_name, country_code, country_name, country_subdivision_code, country_subdivision_name ) 
        SELECT * FROM UNNEST($1::text[], $2::int[], $3::text[], $4::real[], $5::real[], 
        $6::text[], $7::text[], $8::text[], $9::text[], $10::text[], $11::text[])"#;

        sqlx::query(sql)
                .bind(&self.loc_db_ids).bind(&self.gn_ids).bind(&self.gn_names).bind(&self.lats).bind(&self.lngs)
                .bind(&self.cont_codes).bind(&self.cont_names).bind(&self.cy_codes).bind(&self.cy_names)
                .bind(&self.cy_subdiv_codes).bind(&self.cy_subdiv_names)
                .execute(pool)
                .await.map_err(|e| AppError::SqlxError(e, "Storing src locations".to_string()))

    }
}


fn obtain_name_type(name: &str, t1: &str, t2: &str) -> (String, String) {

    let acronym_limit: usize = 5;
    let selected_type: &str;
    let other_type: &str;
    
    if name.chars().count() <=  acronym_limit {
        if t1 == "acronym" || t2 == "acronym" {
            selected_type = "acronym";
            other_type = if t1 == "acronym" {t2} else {t1};
        }
        else {
            if t1 == "label" || t2 == "label" {
                selected_type =  "label";
                other_type = if t1 == "label" {t2} else {t1};
            }
            else {  // shouldn't occur but if it does use first
                selected_type = t1;
                other_type = t2;
            }
        }
    }
    else {
        if t1 == "label" || t2 == "label" {
            selected_type = "label";
            other_type = if t1 == "label" {t2} else {t1};
        }
        else {
            if t1 == "alias" || t2 == "alias" {
                selected_type = "alias";
                other_type = if t1 == "alias" {t2} else {t1};
            }
            else {  // shouldn't occur but if it does use first
                selected_type = t1;
                other_type = t2;
            }
        }
    }

    (selected_type.to_string(), other_type.to_string())
}


pub struct NonRequiredDataVecs {
    pub link_db_ids: Vec<String>,
    pub link_types: Vec<String>,
    pub links: Vec<String>,

    pub id_db_ids: Vec<String>,
    pub id_types: Vec<String>,
    pub id_values: Vec<String>,
    pub is_prefs: Vec<Option<bool>>,

    pub rel_db_ids: Vec<String>,
    pub rel_types: Vec<String>,
    pub rel_ids: Vec<String>,
    pub rel_labels: Vec<String>,

    pub dom_db_ids: Vec<String>,
    pub doms:Vec<String>,

}

impl NonRequiredDataVecs{
    pub fn new(vsize: usize) -> Self {
        NonRequiredDataVecs { 
            link_db_ids: Vec::with_capacity(vsize),
            link_types: Vec::with_capacity(vsize),
            links: Vec::with_capacity(vsize),

            id_db_ids: Vec::with_capacity(vsize),
            id_types: Vec::with_capacity(vsize),
            id_values: Vec::with_capacity(vsize),
            is_prefs: Vec::with_capacity(vsize),

            rel_db_ids: Vec::with_capacity(vsize),
            rel_types: Vec::with_capacity(vsize),
            rel_ids: Vec::with_capacity(vsize),
            rel_labels: Vec::with_capacity(vsize),

            dom_db_ids: Vec::with_capacity(vsize),
            doms: Vec::with_capacity(vsize),

        }
    }

    pub fn add_non_required_data(&mut self, r: &RorRecord, db_id: &String) 
    {
        // Relationships.

        if let Some(rels) = r.relationships.as_ref() {
            for rel in rels.iter()
            {
                self.rel_db_ids.push(db_id.clone());
                self.rel_types.push(rel.rel_type.clone());
                self.rel_ids.push(extract_id_from(&rel.id).to_string());
                self.rel_labels.push(rel.label.clone());
            };
        }
    
        // Links.

        if let Some(lnks) = r.links.as_ref() {
            for lnk in lnks.iter()
            {
                self.link_db_ids.push(db_id.clone());
                self.link_types.push(lnk.link_type.clone());
                self.links.push(lnk.value.clone());
            }
        }
        
        // External ids.

        if let Some(eids) = r.external_ids.as_ref() {
            for eid in eids.iter()
            {
                let id_type = &eid.id_type;
                let pref = match &eid.preferred {   // Obtain (ref to) value of preferred Id
                    Some(p) => p,
                    None => "none",
                };
                
                if eid.all.len() > 0 {         // 'all' may contain one or more strings representing Ids
                    if eid.all.len()  == 1 {   // if only 1 then it is always 'preferred'
                        self.id_db_ids.push(db_id.clone());
                        self.id_types.push(id_type.to_string());
                        self.id_values.push(eid.all[0].to_string());
                        self.is_prefs.push(Some(true));  
                      }
                      else {             // Iterate across the various ids listed in '.all'.
                        for id in eid.all.iter() 
                        {
                            self.id_db_ids.push(db_id.clone());
                            self.id_types.push(id_type.to_string());
                            self.id_values.push(id.to_string());
                            if *id == *pref 
                            {
                                self.is_prefs.push(Some(true));
                            }
                            else 
                            {
                                self.is_prefs.push(None);
                            }
                        }
                    }
                }
            }
        }
            
        // Domains.

        if let Some(doms) = r.domains.as_ref() {
            for dom in doms.iter()
            {
                self.dom_db_ids.push(db_id.clone());
                self.doms.push(dom.to_string());
            }
        }
       
    }

    pub async fn store_data(&self, pool : &Pool<Postgres>) -> Result<PgQueryResult, AppError> {

        // Do the relationships data.

        let sql = r#"INSERT INTO src.relationships (id, rel_type, related_id, related_label) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])"#;
        sqlx::query(sql)
            .bind(&self.rel_db_ids).bind(&self.rel_types).bind(&self.rel_ids).bind(&self.rel_labels)
            .execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, "Storing src rels".to_string()))?;
    
        // Do the links data.

        let sql = r#"INSERT INTO src.links (id, link_type, value) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])"#;
        sqlx::query(sql)
            .bind(&self.link_db_ids).bind(&self.link_types).bind(&self.links)
            .execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, "Storing src links".to_string()))?;
        
        // Do the external ids data.

        let sql = r#"INSERT INTO src.external_ids (id, id_type, id_value, is_preferred) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::bool[])"#;
        sqlx::query(sql)
            .bind(&self.id_db_ids).bind(&self.id_types).bind(&self.id_values).bind(&self.is_prefs)
            .execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, "Storing src external_ids".to_string()))?;
    
        // Do the domain data.

        let sql = r#"INSERT INTO src.domains (id, value) 
        SELECT * FROM UNNEST($1::text[], $2::text[])"#;
        sqlx::query(sql)
            .bind(&self.dom_db_ids).bind(&self.doms)
            .execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, "Storing src domains".to_string()))
    }

}


pub fn extract_id_from(full_id: &String) -> &str {
    let b = full_id.as_bytes();
    std::str::from_utf8(&b[b.len()-9..]).unwrap()
}


pub async fn store_strange_ror_record(id: &str, name: &str, oddity_type: i32, pool : &Pool<Postgres>) -> Result<PgQueryResult, AppError> {

    let sql = r#"INSERT INTO rec.strange_ror_names (id, value, oddity_type) values ($1, $2, $3);"#;
    sqlx::query(sql)
        .bind(id).bind(name).bind(oddity_type).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))
}


// Tests

#[cfg(test)]
mod tests {
    use super::*;
    
    // Ensure the extract_id_from utility function works as expected.

    #[test]
    fn test_extracting_id() {
        let test_id = "https://src.org/123456789".to_string();
        assert_eq!(extract_id_from(&(test_id)), "123456789")
    }
}


use chrono::NaiveDate;
use sqlx::{Pool, Postgres};
use crate::import::ror_json_models::RorRecord;

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


    pub async fn store_data(&self, pool : &Pool<Postgres>) {
    
        // do the core data
        let _ = sqlx::query(r#"INSERT INTO ror.core_data (id, ror_full_id, status, established) 
            SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::int[])"#)
        .bind(&self.db_ids)
        .bind(&self.ror_ids)
        .bind(&self.statuses)
        .bind(&self.estabs)
        .execute(pool)
        .await;
        
        // do the admin data
        let _ = sqlx::query(r#"INSERT INTO ror.admin_data (id, created, cr_schema, last_modified, lm_schema) 
            SELECT * FROM UNNEST($1::text[], $2::timestamp[], $3::text[], $4::timestamp[], $5::text[])"#)
        .bind(&self.db_ids)
        .bind(&self.created_dates)
        .bind(&self.created_vs)
        .bind(&self.lastmod_dates)
        .bind(&self.lastmod_vs)
        .execute(pool)
        .await;

    }

}


pub struct RequiredDataVecs {
    pub name_db_ids: Vec<String>,
    pub names: Vec<String>,
    pub name_types: Vec<String>,
    pub is_rors:Vec<Option<bool>>,
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

    pub fn add_required_data(&mut self, r: &RorRecord, db_id: &String) 
    {
        if r.names.len() > 0 {
            for name in r.names.iter()
            {
                if name.types.len() > 0 {

                    // First option inserted for the small number of cases (~30)
                    // where only 'ror_display' is provided as the name type

                    if name.types.len() == 1 && name.types[0] == "ror_display" {
                        self.name_db_ids.push(db_id.clone());
                        self.names.push(name.value.clone());
                        self.name_types.push("label".to_string());
                        self.is_rors.push(Some(true));
                        self.langs.push(name.lang.clone()); 

                        println!("ror name found without name type for{}: {}", db_id.clone(), name.value.clone())
                    }
                    else {
                        let mut is_a_ror_name: Option<bool> = None;
                        if name.types.contains(&"ror_display".to_string())
                        {
                            is_a_ror_name = Some(true);
                        }
                        for name_type in name.types.iter()
                        {
                            if name_type != "ror_display" {
                                self.name_db_ids.push(db_id.clone());
                                self.names.push(name.value.clone());
                                self.name_types.push(name_type.clone());
                                self.is_rors.push(is_a_ror_name);
                                self.langs.push(name.lang.clone()); 
                            }
                        }
                    }
                }
            }
        }

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

    pub async fn store_data(&self, pool : &Pool<Postgres>) {
        
        // do the name data
        let _ = sqlx::query(r#"INSERT INTO ror.names (id, value, name_type, is_ror_name, lang) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::bool[], $5::text[])"#)
        .bind(&self.name_db_ids)
        .bind(&self.names)
        .bind(&self.name_types)
        .bind(&self.is_rors)
        .bind(&self.langs)
        .execute(pool)
        .await;

        // do the type data
        let _ = sqlx::query(r#"INSERT INTO ror.type (id, org_type) 
        SELECT * FROM UNNEST($1::text[], $2::text[])"#)
        .bind(&self.type_db_ids)
        .bind(&self.org_types)
        .execute(pool)
        .await;

        // do the location data
        let _ = sqlx::query(r#"INSERT INTO ror.locations (id, geonames_id, name, lat, lng, 
        continent_code, continent_name, country_code, country_name, country_subdivision_code, country_subdivision_name ) 
        SELECT * FROM UNNEST($1::text[], $2::int[], $3::text[], $4::real[], $5::real[], $6::text[], $7::text[], $8::text[], $9::text[], $10::text[], $11::text[])"#)
        .bind(&self.loc_db_ids)
        .bind(&self.gn_ids)
        .bind(&self.gn_names)
        .bind(&self.lats)
        .bind(&self.lngs)
        .bind(&self.cont_codes)
        .bind(&self.cont_names)
        .bind(&self.cy_codes)
        .bind(&self.cy_names)
        .bind(&self.cy_subdiv_codes)
        .bind(&self.cy_subdiv_names)
        .execute(pool)
        .await;

    }
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
        // relationships - may be none
        if r.relationships.is_some() {
            let rels = r.relationships.as_ref().unwrap();
            if rels.len() > 0 {
                for rel in rels.iter()
                {
                    self.rel_db_ids.push(db_id.clone());
                    self.rel_types.push(rel.rel_type.clone());
                    self.rel_ids.push(extract_id_from(&rel.id).to_string());
                    self.rel_labels.push(rel.label.clone());
                };
            }
        }
    
        // links - may be none
        if r.links.is_some(){
            let lnks = r.links.as_ref().unwrap();
            if lnks.len() > 0 {
                for lnk in lnks.iter()
                {
                    self.link_db_ids.push(db_id.clone());
                    self.link_types.push(lnk.link_type.clone());
                    self.links.push(lnk.value.clone());
                }; 
            }
        }
        
        // external ids - may be none
        if r.external_ids.is_some() {
            let eids = r.external_ids.as_ref().unwrap();
            if eids.len() > 0 {
                for eid in eids.iter()
                {
                    // these 2 constant for each Id record
                    let id_type = &eid.id_type;
                    let mut pref = "none";
                    if eid.preferred.is_some() {
                        pref = eid.preferred.as_ref().unwrap();
                    }
                    
    
                    // 'all' may contain one or more strings representing Ids
                    if eid.all.len() > 0 {
                        if eid.all.len()  == 1 {
    
                            // if only 1 then it is always preferred
    
                            self.id_db_ids.push(db_id.clone());
                            self.id_types.push(id_type.clone());
                            self.id_values.push(eid.all[0].clone());
                            self.is_prefs.push(Some(true));  
                          }
                          else {
    
                            // Iterate across the various ids listed in all.
                            // Indicate when the id = designated preferred.
    
                            for id in eid.all.iter() 
                            {
                                self.id_db_ids.push(db_id.clone());
                                self.id_types.push(id_type.clone());
                                self.id_values.push(id.clone());
                                if id == pref 
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
        }
        
    
        // domains - may be none
        if r.domains.is_some() {
            let doms = r.domains.as_ref().unwrap();
            if doms.len() > 0 {
                for dom in doms.iter()
                {
                    self.dom_db_ids.push(db_id.clone());
                    self.doms.push(dom.clone());
                }
            }
        }
    
    }

    pub async fn store_data(&self, pool : &Pool<Postgres>) {

        // do the relationships data
        let _ = sqlx::query(r#"INSERT INTO ror.relationships (id, rel_type, related_id, related_label) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])"#)
        .bind(&self.rel_db_ids)
        .bind(&self.rel_types)
        .bind(&self.rel_ids)
        .bind(&self.rel_labels)
        .execute(pool)
        .await;

    
        // do the links data
        let _ = sqlx::query(r#"INSERT INTO ror.links (id, link_type, value) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])"#)
        .bind(&self.link_db_ids)
        .bind(&self.link_types)
        .bind(&self.links)
        .execute(pool)
        .await;

    
        // do the external ids data
        let _ = sqlx::query(r#"INSERT INTO ror.external_ids (id, id_type, id_value, is_preferred) 
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::bool[])"#)
        .bind(&self.id_db_ids)
        .bind(&self.id_types)
        .bind(&self.id_values)
        .bind(&self.is_prefs)
        .execute(pool)
        .await;
    
        // do the domain data
        let _ = sqlx::query(r#"INSERT INTO ror.domains (id, value) 
        SELECT * FROM UNNEST($1::text[], $2::text[])"#)
        .bind(&self.dom_db_ids)
        .bind(&self.doms)
        .execute(pool)
        .await;

    }

}


pub fn extract_id_from(full_id: &String) -> &str {
    let b = full_id.as_bytes();
    std::str::from_utf8(&b[b.len()-9..]).unwrap()
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    
    // Ensure the extract_id_from utility function works as expected.

    #[test]
    fn test_extracting_id() {
        let test_id = "https://ror.org/123456789".to_string();
        assert_eq!(extract_id_from(&(test_id)), "123456789")
    }
}


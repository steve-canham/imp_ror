use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use crate::err::AppError;
use chrono::Local;
use super::export_structs::{CSVSummaryRow, CSVAttributeRow, CSVDistribRow, CSVRankedRow, 
                            CSVSingletonRow, CSVOrgAndLangRow, CSVOrgAndRelRow};
use serde::Serialize;
use super::export_helpers;

pub async fn generate_csv(output_folder : &PathBuf, data_version: &String, 
                 inc_withdrawn: bool, pool : &Pool<Postgres> ) -> Result<(), AppError>
{
    let datetime_string = Local::now().format("%Y-%m-%d %H%M%S").to_string();
    let where_clause = format!("where vs.vcode = '{data_version}' and vs.inc_wd = {inc_withdrawn}");
    let dv_string = if inc_withdrawn {format!("{data_version} inc WD")} else {data_version.to_string()};
    
    // 1) Version Summary 

    let output_file_name = format!("{} {} {}.csv", dv_string, "summary", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!("SELECT vcode, inc_wd, vdate::text, vdays, num_recs, 
                               num_active, num_inactive, num_withdrawn, num_denom, num_names, 
                               num_types, num_links, num_ext_ids, num_rels, num_locations, num_domains 
                               from smm.version_summaries vs
                               {where_clause}");
    let summ: CSVSummaryRow = sqlx::query_as(&sql).fetch_one(pool).await
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let summ_as_vec = vec![summ];
    generate_file(&file_path, summ_as_vec)?;

    
    // 2) Attribute Summaries
    
    let output_file_name = format!("{} {} {}.csv", dv_string, "attributes", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, att_id, att_name, 
                            cat_id, cat_name, number_cat, pc_of_atts, number_orgs, pc_of_orgs
                            from smm.attributes_summary ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by att_id, cat_id;"#);

    let att_rows: Vec<CSVAttributeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, att_rows)?;

   
    // 3) Count distributions

    let output_file_name = format!("{} {} {}.csv", dv_string, "counts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, count_id,
                            count_name, count, num_of_orgs, pc_of_orgs
                            from smm.count_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by count_id, count;"#);

    let cdist_rows: Vec<CSVDistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, cdist_rows)?;


    // 4) Ranked count distributions

    let output_file_name = format!("{} {} {}.csv", dv_string, "ranked_counts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, dist_id,
                            dist_name, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by dist_id, rank;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;


    // 5) Singletons

    let output_file_name = format!("{} {} {}.csv", dv_string, "singletons", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
    
    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            id, name, description, number, pc
                            from smm.singletons ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by id;"#);
    
    let sing_rows: Vec<CSVSingletonRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, sing_rows)?;


    // 6) Org types and WOLC

    let output_file_name = format!("{} {} {}.csv", dv_string, "orgtypes and names", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
   
    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            name_type_id, name_type, org_type_id, org_type, 
                            names_num, names_wlc, names_wolc, names_wlc_pc, names_wolc_pc
                            from smm.org_type_and_lang_code ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by name_type_id, org_type_id;"#);

    let orglang_rows: Vec<CSVOrgAndLangRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, orglang_rows)?;


    // 7) Orgs types and relationships

    let output_file_name = format!("{} {} {}.csv", dv_string, "orgtypes and relationships", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            rel_type_id, rel_type, org_type_id, org_type, 
                            ss.num_links, ss.num_orgs, num_orgs_total, num_orgs_pc 
                            from smm.org_type_and_relationships ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by rel_type_id, org_type_id;"#);
   
    let orgrel_rows: Vec<CSVOrgAndRelRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;                               
    generate_file(&file_path, orgrel_rows)?;

    Ok(())
}


pub async fn generate_all_versions_csv(output_folder : &PathBuf, inc_withdrawn: bool, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    let datetime_string = Local::now().format("%Y-%m-%d %H%M%S").to_string();
    let where_clause = format!("where vs.inc_wd = {inc_withdrawn} and vs.vcode <> 'v1.57' ");
    let dv_string = if inc_withdrawn {"All versions inc WD"} else {"All versions"};
    
    // 1) Version Summary 

    let output_file_name = format!("{} {} {}.csv", dv_string, "summary", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!("SELECT vcode, inc_wd, vdate::text, vdays, num_recs, 
                               num_active, num_inactive, num_withdrawn, num_denom, num_names, 
                               num_types, num_links, num_ext_ids, num_rels, num_locations, num_domains 
                               from smm.version_summaries vs 
                               {where_clause} 
                               order by vdays;");
    let summs: Vec<CSVSummaryRow> = sqlx::query_as(&sql).fetch_all(pool).await
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, summs)?;


    // 2) Attribute Summaries
    
    let output_file_name = format!("{} {} {}.csv", dv_string, "attributes", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
    
    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, att_id, att_name, 
                            cat_id, cat_name, number_cat, pc_of_atts, number_orgs, pc_of_orgs
                            from smm.attributes_summary ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by att_id, cat_id, vs.vdays;"#);

    let att_rows: Vec<CSVAttributeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, att_rows)?;
  

    // 3) Count distributions

    let output_file_name = format!("{} {} {}.csv", dv_string, "counts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            count_id, count_name, count, num_of_orgs, pc_of_orgs
                            from smm.count_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause} 
                            order by count_id, count, vs.vdays;"#);

    let cdist_rows: Vec<CSVDistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, cdist_rows)?;


    // 4) Ranked count distributions

    // a) languages 

    let output_file_name = format!("{} {} {}.csv", dv_string, "ranked_languages", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            dist_id, dist_name, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause} 
                            and dist_id = 1
                            order by dist_id, rank, vs.vdays;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;

    // b) scripts

    let output_file_name = format!("{} {} {}.csv", dv_string, "ranked_scripts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            dist_id, dist_name, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            and dist_id = 2
                            order by dist_id, rank, vs.vdays;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;

    // c) countries

    let output_file_name = format!("{} {} {}.csv", dv_string, "ranked_countries", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            dist_id, dist_name, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            and dist_id = 3
                            order by dist_id, rank, vs.vdays;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;

    export_helpers::set_up_country_grid(pool).await?;

    // N.B. The country grid data is most easily extracted manually, from the table (e.g. in pgAdmin).
    // Partly becaue the columns may vary for each extraction so it is difficult to construct a struct
    // for deserialisation.


    // 5) Singletons

    let output_file_name = format!("{} {} {}.csv", dv_string, "singletons", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            id, name, description, number, pc
                            from smm.singletons ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by id, vs.vdays;"#);
    
    let sing_rows: Vec<CSVSingletonRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, sing_rows)?;


    // 6) Org types and WOLC

    let output_file_name = format!("{} {} {}.csv", dv_string, "orgtypes and names", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            name_type_id, name_type, org_type_id, org_type, 
                            names_num, names_wlc, names_wolc, names_wlc_pc, names_wolc_pc
                            from smm.org_type_and_lang_code ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by name_type_id, org_type_id, vs.vdays;"#);

    let orglang_rows: Vec<CSVOrgAndLangRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, orglang_rows)?;


    // 7) Orgs types and relationships

    let output_file_name = format!("{} {} {}.csv", dv_string, "orgtypes and relationships", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

        let sql = format!(r#"SELECT vs.vcode, vs.inc_wd, vs.vdate::text, vs.vdays, 
                            rel_type_id, rel_type, org_type_id, org_type, 
                            ss.num_links, ss.num_orgs, num_orgs_total, num_orgs_pc 
                            from smm.org_type_and_relationships ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            {where_clause}
                            order by rel_type_id, org_type_id, vs.vdays;"#);
   
    let orgrel_rows: Vec<CSVOrgAndRelRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;       
    generate_file(&file_path, orgrel_rows)?;

    Ok(())
}


fn generate_file<T: Serialize>(file_path: &PathBuf, data:Vec<T>) -> Result<(), AppError> {
    
    let mut wtr = csv::Writer::from_path(file_path)
                .map_err(|e|AppError::CsvError(e))?;

    for d in data {
        wtr.serialize(d)
        .map_err(|e|AppError::CsvError(e))?;
    }
    
    wtr.flush()?;
    Ok(())
}

    

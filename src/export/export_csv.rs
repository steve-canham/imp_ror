use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use crate::err::AppError;
use chrono::Local;
use super::export_structs::{CSVSummaryRow, CSVAttributeRow, CSVDistribRow, CSVRankedRow, 
                            CSVSingletonRow, CSVOrgAndLangRow, CSVOrgAndRelRow};
use serde::Serialize;
use super::export_helpers;

pub async fn generate_csv(output_folder : &PathBuf, data_version: &String, pool : &Pool<Postgres> ) -> Result<(), AppError>
{
    let datetime_string = Local::now().format("%Y-%m-%d %H%M%S").to_string();
    
    // 1) Version Summary 

    let output_file_name = format!("{} {} {}.csv", data_version, "summary", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
           
    let sql = format!("SELECT vcode, vdate::text, vdays, num_orgs, num_names, 
                               num_types, num_links, num_ext_ids, num_rels, num_locations, num_domains 
                               from smm.version_summaries WHERE vcode = '{}';", data_version);
    let summ: CSVSummaryRow = sqlx::query_as(&sql).fetch_one(pool).await
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let summ_as_vec = vec![summ];
    generate_file(&file_path, summ_as_vec)?;
   

    // 2) Attribute Summaries
    
    let output_file_name = format!("{} {} {}.csv", data_version, "attributes", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, att_type as att_id, att_name, 
                            id as cat_id, name as cat_name, number_atts, pc_of_atts, number_orgs, pc_of_orgs
                            from smm.attributes_summary ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode = '{}'
                            order by att_name, id;"#, data_version);

    let att_rows: Vec<CSVAttributeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, att_rows)?;

   
    // 3) Count distributions

    let output_file_name = format!("{} {} {}.csv", data_version, "counts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            count_type, count, num_of_orgs, pc_of_orgs
                            from smm.count_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode = '{}'
                            order by count_type, count;"#, data_version);

    let cdist_rows: Vec<CSVDistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, cdist_rows)?;


    // 4) Ranked count distributions

    let output_file_name = format!("{} {} {}.csv", data_version, "ranked_counts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            dist_type, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode = '{}'
                            order by dist_type, rank;"#, data_version);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;


    // 5) Singletons

    let output_file_name = format!("{} {} {}.csv", data_version, "singletons", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
    
    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            id, description, number, pc
                            from smm.singletons ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode = '{}'
                            order by id;"#, data_version);
    
    let sing_rows: Vec<CSVSingletonRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, sing_rows)?;


    // 6) Org types and WOLC

    let output_file_name = format!("{} {} {}.csv", data_version, "orgtypes and names", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
   
    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            org_type, name_type, names_num,names_wlc, names_wolc, names_wlc_pc, names_wolc_pc
                            from smm.org_type_and_lang_code ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode = '{}'
                            order by org_type, name_type;"#, data_version);

    let orglang_rows: Vec<CSVOrgAndLangRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, orglang_rows)?;


    // 7) Orgs types and relationships

    let output_file_name = format!("{} {} {}.csv", data_version, "orgtypes and relationships", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            org_type, rel_type, ss.num_links, ss.num_orgs, num_orgs_total, num_orgs_pc 
                            from smm.org_type_and_relationships ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode = '{}' 
                            order by org_type, rel_type;"#, data_version);
   
    let orgrel_rows: Vec<CSVOrgAndRelRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;                               
    generate_file(&file_path, orgrel_rows)?;

    Ok(())
}



pub async fn generate_all_versions_csv(output_folder : &PathBuf, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    let datetime_string = Local::now().format("%Y-%m-%d %H%M%S").to_string();

    // 1) Version Summary 

    let output_file_name = format!("{} {} {}.csv", "All versions", "summary", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!("SELECT vcode, vdate::text, vdays, num_orgs, num_names, 
                               num_types, num_links, num_ext_ids, num_rels, num_locations, num_domains 
                               from smm.version_summaries WHERE vcode <> 'v1.57' 
                               order by vcode;");
    let summs: Vec<CSVSummaryRow> = sqlx::query_as(&sql).fetch_all(pool).await
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, summs)?;


    // 2) Attribute Summaries
    
    let output_file_name = format!("{} {} {}.csv", "All versions", "attributes", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
    
    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, att_type as att_id, att_name, 
                            id as cat_id, name as cat_name, number_atts, pc_of_atts, number_orgs, pc_of_orgs
                            from smm.attributes_summary ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            order by vcode, att_name, id;"#);

    let att_rows: Vec<CSVAttributeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, att_rows)?;
  

    // 3) Count distributions

    let output_file_name = format!("{} {} {}.csv", "All versions", "counts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            count_type, count, num_of_orgs, pc_of_orgs
                            from smm.count_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            order by vcode, count_type, count;"#);

    let cdist_rows: Vec<CSVDistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, cdist_rows)?;


    // 4) Ranked count distributions

    // a) languages 

    let output_file_name = format!("{} {} {}.csv", "All versions", "ranked_languages", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            dist_type, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            and dist_type = 1
                            order by vcode, dist_type, rank;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;

    // b) scripts

    let output_file_name = format!("{} {} {}.csv", "All versions", "ranked_scripts", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            dist_type, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            and dist_type = 2
                            order by vcode, dist_type, rank;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;

    // c) countries

    let output_file_name = format!("{} {} {}.csv", "All versions", "ranked_countries", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            dist_type, rank, entity, number, pc_of_entities, pc_of_base_set
                            from smm.ranked_distributions ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            and dist_type = 3
                            order by vcode, dist_type, rank;"#);

    let rdist_rows: Vec<CSVRankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, rdist_rows)?;

    export_helpers::set_up_country_grid(pool).await?;


    // 5) Singletons

    let output_file_name = format!("{} {} {}.csv", "All versions", "singletons", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();
    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            id, description, number, pc
                            from smm.singletons ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            order by vcode, id;"#);
    
    let sing_rows: Vec<CSVSingletonRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, sing_rows)?;


    // 6) Org types and WOLC

    let output_file_name = format!("{} {} {}.csv", "All versions", "orgtypes and names", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

    let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            org_type, name_type, names_num, names_wlc, names_wolc, names_wlc_pc, names_wolc_pc
                            from smm.org_type_and_lang_code ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            order by vcode, org_type, name_type;"#);

    let orglang_rows: Vec<CSVOrgAndLangRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    generate_file(&file_path, orglang_rows)?;


    // 7) Orgs types and relationships

    let output_file_name = format!("{} {} {}.csv", "All versions", "orgtypes and relationships", datetime_string);
    let file_path: PathBuf = [output_folder, &PathBuf::from(&output_file_name)].iter().collect();

        let sql = format!(r#"SELECT vs.vcode, vs.vdate::text, vs.vdays, 
                            org_type, rel_type, ss.num_links, ss.num_orgs, num_orgs_total, num_orgs_pc 
                            from smm.org_type_and_relationships ss
                            inner join smm.version_summaries vs 
                            on vs.vcode = ss.vcode 
                            where vs.vcode <> 'v1.57' 
                            order by vcode, org_type, rel_type;"#);
   
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

    

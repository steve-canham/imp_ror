use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use crate::err::AppError;
use chrono::Local;

pub async fn generate_csv(output_folder : &PathBuf, data_version: &String, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    let datetime_string = Local::now().format("%m-%d %H%M%S").to_string();

    // 1) Version Summary 

    let table_type = "summary".to_string();
    let select_statement = r#"select * from smm.version_summaries where vcode = '"#.to_string() + data_version + r#"'"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 2) Attribute Summaries
    
    let table_type = "attributes".to_string();
    let select_statement = r#"select * from smm.attributes_summary where vcode = '"#.to_string() 
                           + data_version + r#"' order by att_name, id"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;
    
    // 3) Count distributions

    let table_type = "counts".to_string();
    let select_statement = r#"select * from smm.count_distributions where vcode = '"#.to_string() 
                           + data_version + r#"' order by count_type, count"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 4) Ranked count distributions

    let table_type = "ranked_counts".to_string();
    let select_statement = r#"select * from smm.ranked_distributions where vcode = '"#.to_string() 
                           + data_version + r#"' order by dist_type, rank"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 5) Singletons

    let table_type = "singletons".to_string();
    let select_statement = r#"select * from smm.singletons where vcode = '"#.to_string() + data_version + r#"'"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 6) Org types and WOLC

    let table_type = "orgtypes and names wolc".to_string();
    let select_statement = r#"select * from smm.org_type_and_lang_code where vcode = '"#.to_string() 
                        + data_version + r#"' order by org_type, name_type"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 7) Orgs types and relationships

    let table_type = "orgtypes and relationships".to_string();
    let select_statement = r#"select * from smm.org_type_and_relationships where vcode = '"#.to_string() 
                        + data_version + r#"' order by org_type, rel_type"#;
    generate_file(output_folder, data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    Ok(())
}



pub async fn generate_all_versions_csv(output_folder : &PathBuf, pool : &Pool<Postgres>) -> Result<(), AppError>
{

    let datetime_string = Local::now().format("%m-%d %H%M%S").to_string();
    let data_version = "All versions".to_string();

    // 1) Version Summary 

    let table_type = "summary".to_string();
    let select_statement = r#"select * from smm.version_summaries where vcode <> 'v1.57' order by vcode"#.to_string();
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 2) Attribute Summaries
    
    let table_type = "attributes".to_string();
    let select_statement = r#"select vs.vcode, vs.vdate, vs.vdays, 
                             s.att_type, s.att_name, s.id, s.name, s.number_atts, s.pc_of_atts, s.number_orgs, s.pc_of_orgs 
                             from smm.version_summaries vs 
                             inner join smm.attributes_summary s
                             on vs.vcode = s.vcode
                             where vs.vcode <> 'v1.57' order by vcode, att_name, id"#.to_string();
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 3) Count distributions

    let table_type = "counts".to_string();
    let select_statement = r#"select vs.vcode, vs.vdate, vs.vdays, 
                             s.count_type, s.count_type, s.num_of_orgs, s.pc_of_orgs 
                             from smm.version_summaries vs 
                             inner join smm.count_distributions s
                             on vs.vcode = s.vcode
                             where vs.vcode <> 'v1.57' order by vcode, count_type, count"#.to_string();
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 4) Ranked count distributions

    let table_type = "ranked_counts".to_string();
    let select_statement = r#"select vs.vcode, vs.vdate, vs.vdays, 
                             s.dist_type, s.rank, s.entity, s.number, s.pc_of_entities, s.pc_of_base_set 
                             from smm.version_summaries vs 
                             inner join smm.ranked_distributions s
                             on vs.vcode = s.vcode
                             where vs.vcode <> 'v1.57' order by vcode, dist_type, rank"#.to_string();
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 5) Singletons

    let table_type = "singletons".to_string();
    let select_statement = r#"select vs.vcode, vs.vdate, vs.vdays, 
                             s.id, s.description, s.number, s.pc
                             from smm.version_summaries vs 
                             inner join smm.singletons s
                             on vs.vcode = s.vcode
                             where vs.vcode <> 'v1.57' order by vcode"#.to_string();
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    // 6) Org types and WOLC

    let table_type = "orgtypes and names wolc".to_string();
    let select_statement = r#"select vs.vcode, vs.vdate, vs.vdays, 
                             s.org_type, s.name_type, s.names_num, s.names_wlc, s.names_wolc, s.names_wlc_pc, s.names_wolc_pc
                             from smm.version_summaries vs 
                             inner join smm.org_type_and_lang_code s
                             on vs.vcode = s.vcode
                             where vs.vcode <> 'v1.57' order by vcode, org_type, name_type"#.to_string();
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;


    // 7) Orgs types and relationships

    let table_type = "orgtypes and relationships".to_string();
    let select_statement = r#"select vs.vcode, vs.vdate, vs.vdays, 
                             s.org_type, s.rel_type, s.num_links, s.num_orgs, s.num_orgs_total, s.num_orgs_pc
                             from smm.version_summaries vs 
                             inner join smm.org_type_and_relationships s
                             on vs.vcode = s.vcode
                             where vs.vcode <> 'v1.57' order by vcode, org_type, rel_type"#.to_string();
                             r#"select * from smm.org_type_and_relationships where vcode <> 'v1.57' order by vcode, org_type, rel_type"#;
    generate_file(output_folder, &data_version, &select_statement, &datetime_string, &table_type, pool).await?;

    Ok(())
}



async fn  generate_file(output_folder: &PathBuf, data_version: &String, select_statement: &String,
                    datetime_string: &String, table_type: &String, pool : &Pool<Postgres>) -> Result<(), AppError> {
    
    let output_file_name = format!("{} {} {}.csv", data_version, table_type, datetime_string);
    let output_file_path: PathBuf = [output_folder,  &PathBuf::from(&output_file_name)].iter().collect();
    let output_file = match output_file_path.to_str() {
        Some(s) => s.to_string(),
        None => {
            return Err(AppError::FileSystemError("Unable to construct the output file name".to_string(), 
                       format!("File name was: {},", output_file_name)))
        },
    };
    let sql = r#"copy ("#.to_string() + select_statement + r#") to '"# + &output_file + r#"' DELIMITER ',' CSV HEADER"#;
    sqlx::raw_sql(&sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(())
}
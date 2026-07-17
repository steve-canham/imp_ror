use sqlx::{Pool, Postgres};
use crate::AppError;
use std::fs::OpenOptions;
use std::io::prelude::*;
use super::export_structs::{Singleton, TypeRow, DistribRow, RankedRow, OrgAndLangCode, OrgAndRel}; 
use std::{collections::HashMap, path::PathBuf};

#[derive(sqlx::FromRow)]
struct CPars {
    cname: String,
    colname: String,
}


pub fn get_hdr_line(topic: &str) -> String {
    format!("\n\n\t{}\n\t{topic}\n\t{}", "=".repeat(88),  "=".repeat(88),)
}

pub fn get_sing_hdr() -> String {
    "\n\n\t                                                                 number           %age\n".to_string()
}

pub fn get_singleton_rows(singvals: &HashMap<String, Singleton>, topics: Vec<&str>) -> String {

    let mut sing_text = "".to_string();
    for i in 0..topics.len() {
        let sing = &singvals[topics[i]]; 
        sing_text += &get_singleton_line(&sing.description, sing.number, sing.pc);
    }
    sing_text
}

pub fn get_singleton_line(topic: &str, num: i32, pc: Option<f32>) -> String {

    let spacer1 = " ".repeat(71 - topic.chars().count() - num.to_string().len());
    if pc.is_some() {
        let pc_as_string = format!("{:.2}", pc.unwrap());
        let spacer2 = " ".repeat(15 - pc_as_string.len());
        format!("\n\t{topic}{spacer1}{num}{spacer2}{:.2}", pc.unwrap())
    }
    else {
        format!("\n\t{topic}{spacer1}{num}")
    }
}

pub fn get_data_and_pc_line(topic: &str, num: i32, total: i32) -> String {
    let spacer = " ".repeat(71 - topic.len() - num.to_string().len());
    let pc_as_string = format!("{:.2}", num*100 / total);
    let spacer2 = " ".repeat(15 - pc_as_string.len());
    format!("\n\t{topic}{spacer}{num}{spacer2}{pc_as_string}")
}

pub fn get_data_line(topic: &str, num: i32) -> String {
    let spacer = " ".repeat(71 - topic.len() - num.to_string().len());
    format!("\n\t{topic}{spacer}{num}")
}

pub fn get_attrib_line(category: &str, num_atts: i32, pc_atts: f32, num_orgs: i32, pc_orgs: f32)-> String {
    let spacer1 = " ".repeat(43 - category.chars().count() - num_atts.to_string().len());
    let pc1_as_string = format!("{:.2}", pc_atts);
    let spacer2 = " ".repeat(13 - pc1_as_string.len());
    let spacer3 = " ".repeat(15 - num_orgs.to_string().len());
    let pc2_as_string = format!("{:.2}", pc_orgs);
    let spacer4 = " ".repeat(15 - pc2_as_string.len());
    format!("\n\t{category}{spacer1}{num_atts}{spacer2}{pc1_as_string}{spacer3}{num_orgs}{spacer4}{pc2_as_string}")    
}

fn get_distrib_line(count: i32, num: i32, pc: f32)-> String {
    let spacer1 = " ".repeat(56 - count.to_string().len());
    let spacer2 = " ".repeat(15 - num.to_string().len());
    let pc_as_string = format!("{:.2}", pc);
    let spacer3 = " ".repeat(15 - pc_as_string.len());
    format!("\n\t{spacer1}{count}{spacer2}{num}{spacer3}{pc_as_string}")
}

pub fn get_table_base(stop1: usize, stop2: usize, total: i32, total_pc: f32) -> String {
    let spacer1 = " ".repeat(stop1 - total.to_string().len());
    let pc_as_string = format!("{:.2}", total_pc);
    let spacer2 = " ".repeat(stop2 - pc_as_string.len());
    format!("\n\t{}\n\tTOTAL{spacer1}{total}{spacer2}{pc_as_string}\n", "-".repeat(88))
}

pub fn get_ranked_distrib_line(topic: &str, num: i32, pc1: f32, pc2: f32)-> String {
    let spacer1 = " ".repeat(56 - topic.chars().count() - num.to_string().len());
    let pc1_as_string = format!("{:.2}", pc1);
    let spacer2 = " ".repeat(15 - pc1_as_string.len());
    let pc2_as_string = format!("{:.2}", pc2);
    let spacer3 = " ".repeat(15 - pc2_as_string.len());
    if topic == "United States" {
        format!("\n\t{topic}{spacer1}{num}{}----{spacer3}{pc2_as_string}", " ".repeat(11))
    }
    else {
        format!("\n\t{topic}{spacer1}{num}{spacer2}{pc1_as_string}{spacer3}{pc2_as_string}")
    }
}

pub fn get_orglc_line(org_type: &str, name_type: &str, names_num: i32, names_wolc: i32, names_wolc_pc: f32) -> String {
    let spacer1 = " ".repeat(25 - name_type.chars().count());
    let spacer2 = " ".repeat(31 - org_type.chars().count() - names_num.to_string().len());
    let spacer3 = " ".repeat(15 - names_wolc.to_string().len());
    let pc_as_string = format!("{:.2}", names_wolc_pc);
    let spacer4 = " ".repeat(15 - pc_as_string.to_string().len());
    format!("\n\t{name_type}{spacer1}{org_type}{spacer2}{names_num}{spacer3}{names_wolc}{spacer4}{pc_as_string}")
}

pub fn get_orgrel_line(org_type: &str, rel_type: &str, num_links: i32, num_orgs: i32, num_orgs_pc: f32) -> String {
    let spacer1 = " ".repeat(25 - rel_type.chars().count());
    let spacer2 = " ".repeat(31 - org_type.chars().count() - num_links.to_string().len());
    let spacer3 = " ".repeat(15 - num_orgs.to_string().len());
    let pc_as_string = format!("{:.2}", num_orgs_pc);
    let spacer4 = " ".repeat(15 - pc_as_string.to_string().len());
    format!("\n\t{rel_type}{spacer1}{org_type}{spacer2}{num_links}{spacer3}{num_orgs}{spacer4}{pc_as_string}")
}


pub async fn get_attrib_table(att_type: i32, header_type: &str, 
                          vcode: &String,  inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = format!(r#"select cat_name, number_cat, pc_of_atts, number_orgs, pc_of_orgs 
            from smm.attributes_summary
            where vcode = '{vcode}' and inc_wd = {inc_withdrawn} and att_type = {att_type} 
            order by cat_id; "#);
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;

    let mut rt_numbercat: i32 = 0;   // rt = running total
    let mut rt_numbercat_pc: f32 = 0.0;
    let mut tbl_text = format!("\n
    {}, categories and numbers:
    
                                         number         %age         number          %age
    Category                             in cat       all cats        orgs        total orgs\n\t{}",                                                 
    header_type, "-".repeat(88));

    for r in rows {
        tbl_text += &get_attrib_line(&r.cat_name, r.number_cat, r.pc_of_atts, r.number_orgs, r.pc_of_orgs);
        if !r.cat_name.starts_with("nacro")  
        {
            rt_numbercat += r.number_cat;
            rt_numbercat_pc += r.pc_of_atts;
        }
    }
    tbl_text += &get_table_base(38, 13, rt_numbercat, rt_numbercat_pc);
    Ok(tbl_text)
}


pub async fn get_distrib_table(count_type: &str, header_type: &str, vcode: &String, inc_withdrawn: bool, 
                                 pool: &Pool<Postgres>) -> Result<String, AppError> {
    let sql = format!(r#"select count, num_of_orgs, pc_of_orgs from smm.count_distributions
            where vcode = '{vcode}' and inc_wd = {inc_withdrawn} and count_type = '{count_type}' 
            order by count;"#);
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;

    let mut rt_numberorgs: i32 = 0;
    let mut rt_numberorgs_pc: f32 = 0.0;
    let hdr_spacer = " ".repeat(33 - header_type.len());
    let mut tbl_text = format!("\n
    Numbers of organisations with specified              count       number          %age
    count of {}{}                        orgs        total orgs\n\t{}",
    header_type, hdr_spacer, "-".repeat(88));

    for r in rows {
        tbl_text += &get_distrib_line(r.count, r.num_of_orgs, r.pc_of_orgs);
        rt_numberorgs += r.num_of_orgs;
        rt_numberorgs_pc += r.pc_of_orgs;
    }
    tbl_text += &get_table_base(66, 15, rt_numberorgs, rt_numberorgs_pc);
    Ok(tbl_text)
}


pub async fn get_ranked_distrib_table(dist_type: i32, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = format!(r#"SELECT entity, number, pc_of_entities, pc_of_base_set from smm.ranked_distributions 
            where vcode = '{vcode}' and inc_wd = {inc_withdrawn} and dist_type = {dist_type} order by rank; "#);
    let lang_rows: Vec<RankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
            .map_err(|e| AppError::SqlxError(e, sql))?;

    let mut tbl_text = "".to_string();
    let mut rt_numberents: i32 = 0;  // rt = running totals
    let mut rt_numberents_pc: f32 = 0.0;

    for r in lang_rows {
        tbl_text += &get_ranked_distrib_line(&r.entity, r.number, r.pc_of_entities, r.pc_of_base_set);
        rt_numberents += r.number;
        if r.entity != "United States".to_string() {
            rt_numberents_pc += r.pc_of_entities;
        }
    }
    tbl_text += &get_table_base(51, 15, rt_numberents, rt_numberents_pc);
    Ok(tbl_text)
}


pub async fn get_org_type_and_lang_code_table(vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = format!(r#"select org_type, name_type, names_num, names_wolc, names_wolc_pc 
                from smm.org_type_and_lang_code where vcode = '{vcode}' and inc_wd = {inc_withdrawn} 
                order by name_type_id, org_type_id;"#);
    let rows: Vec<OrgAndLangCode> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    let mut tbl_text = format!("\n\n
    Numbers of name types without language codes for different organisational types
        
                                                    number          names          %age
    name type                org type                  names         w/o LC         w/o LC\n\t{}",                                                 
    "-".repeat(88));
    let mut old_name_type = "".to_string();
    for r in rows {
        if r.name_type != old_name_type {
            old_name_type = r.name_type.clone();
            tbl_text += "\n";
        }
        tbl_text += &get_orglc_line(&r.org_type, &r.name_type, r.names_num, r.names_wolc, r.names_wolc_pc);
    }
    tbl_text += "\n";
    Ok(tbl_text)
}


pub async fn get_org_type_and_relationship_table(vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = format!(r#"select org_type, rel_type, num_links, num_orgs, num_orgs_pc 
            from smm.org_type_and_relationships 
            where vcode = '{vcode}' and inc_wd = {inc_withdrawn} 
            order by rel_type_id, org_type_id;"#);
    let rows: Vec<OrgAndRel> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;                               
    
    let mut tbl_text = format!("\n
    Numbers of relationship links for different organisational types:
    
                                                        number       number           %age
    relationship             org type                   links          orgs         org type\n\t{}",                                                 
    "-".repeat(88));
    let mut old_rel_type = "".to_string();
    for r in rows {
        if r.rel_type != old_rel_type {
            old_rel_type = r.rel_type.clone();
            tbl_text += "\n";
        }
        tbl_text += &get_orgrel_line(&r.org_type, &r.rel_type, r.num_links, r.num_orgs, r.num_orgs_pc);
    }
    tbl_text += "\n";
    Ok(tbl_text)
}


pub fn append_to_file(output_file_path: &PathBuf, contents: &str) -> Result<(), AppError> {

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(output_file_path)?;

    file.write_all(contents.as_bytes())
        .map_err(|e| AppError::IoWriteErrorWithPath(e, output_file_path.to_owned()))
    
}


pub async fn set_up_country_grid(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Create a temp table with the countries listed as being in the top 25
    
    let sql = r#"drop table if exists smm.temp_clist;
                create table smm.temp_clist (
                    id int primary key generated always as identity,
                    cname varchar,
                    colname varchar
                );

                insert into smm.temp_clist (cname, colname)
                select entity,
                lower(replace(entity, ' ', '_'))
                from smm.ranked_distributions
                where dist_type = 3
                and entity <> 'Remaining countries'
                group by entity
                order by sum(number) desc;

                insert into smm.temp_clist (cname, colname)
                values ('Remaining_countries', 'remaining_countries');"#;
    
    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Use that to create the sql that will generate the countries grid table
    // and create that table

    let sql = r#"select string_agg(' '||colname||'  float', ',') 
                    from (select colname from smm.temp_clist order by id)"#;

    let mid_string: String = sqlx::query_scalar(sql).fetch_one(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    let start_sql = r#"drop table if exists smm.countries_grid; 
                create table smm.countries_grid (vcode  varchar, vdate  date,"#;
    let sql = format!("{} {});", start_sql, mid_string);

    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql))?;

    // Add a row to the countries_grid table for each version

    let sql = r#"insert into smm.countries_grid (vcode, vdate)
                       select vcode, vdate from smm.version_summaries
                       where vcode <> 'v1.57'
                       order by vcode"#;
    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Fill in the grid values
    // Obtain the colum / country names from the temp clist table
    // Construct the sqwl for each country to update the grid
    
    let sql = r#"select cname, colname from smm.temp_clist"#;
    let cpars: Vec<CPars> = sqlx::query_as(&sql).fetch_all(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    for cp in cpars {

        let sql = format!(r#"update smm.countries_grid cg
            set {} = round(rd.pc_of_base_set::numeric, 2)
            from smm.ranked_distributions rd
            where cg.vcode = rd.vcode
            and rd.entity = '{}'"#, cp.colname, cp.cname);

        sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    }

    // Drop the temporary country grid table

    let sql = "drop table smm.temp_clist"; 
    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(())

}




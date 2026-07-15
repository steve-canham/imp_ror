use sqlx::{Pool, Postgres};
use std::{collections::HashMap, path::PathBuf};
use crate::AppError;
use std::fs::OpenOptions;
use std::io::prelude::*;
use chrono::{DateTime, Local};
use super::export_structs::{VSummary, TypeRow, DistribRow, RankedRow, 
                            SingletonRow, Singleton, OrgAndLangCode, OrgAndRel};
use log::info;


pub async fn generate_text(output_folder : &PathBuf, vcode: &String, 
                           inc_withdrawn: bool, pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Get path and set up file for writing.

    let datetime_string = Local::now().format("%Y-%m-%d %H-%M").to_string();
    let inc_wd_tag = if inc_withdrawn {" (inc. withdrawn orgs)"} else {""};
    let output_file_name = format!("{} summary{} at {}.txt", &vcode, inc_wd_tag, &datetime_string);
    let output_file_path: PathBuf = [output_folder, &PathBuf::from(output_file_name)].iter().collect();
            
    let singvals:HashMap<String, Singleton> = collect_singleton_values(vcode, inc_withdrawn, pool).await?;
    write_header_and_summary(&output_file_path, vcode, inc_withdrawn, pool).await?;
    write_name_info(&output_file_path, vcode, inc_withdrawn, pool, &singvals).await?;
    write_name_wolc_info(&output_file_path, vcode, inc_withdrawn, pool, &singvals).await?;
    write_ror_name_details(&output_file_path, &singvals).await?;
    write_ranked_name_info(&output_file_path, vcode, inc_withdrawn, pool, &singvals).await?;
    write_type_details(&output_file_path, vcode, inc_withdrawn, pool).await?;
    write_location_details(&output_file_path, vcode, inc_withdrawn, pool, &singvals).await?;
    write_links_and_extid_details(&output_file_path, vcode, inc_withdrawn, pool).await?;
    write_relationship_details(&output_file_path, vcode, inc_withdrawn, pool, &singvals).await?;
    write_domain_details(&output_file_path, vcode, inc_withdrawn, pool).await?;

    info!("Content appended successfully");
    Ok(())
}

async fn collect_singleton_values(vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<HashMap<String, Singleton>, AppError> {

    let mut sstructs = HashMap::new();
    let sql = format!(r#"SELECT id, description, number, pc from smm.singletons 
              WHERE vcode = '{vcode}' and inc_wd = {inc_withdrawn};"#);
    let srows: Vec<SingletonRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    
    for r in srows { 
        let s = Singleton {
            description: r.description,
            number: r.number,
            pc: r.pc,
        };
        sstructs.insert(r.id, s);
    }
    Ok(sstructs)
}

async fn write_header_and_summary(output_file_path: &PathBuf, vcode: &String, 
                                inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Get import date from the ror table, other summary details from the smm.version_summaries table

    let sql = "SELECT import_datetime from src.version_details;";
    let import_dt: DateTime<Local> = sqlx::query_scalar(sql).fetch_one(pool).await 
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = format!(r#"SELECT * from smm.version_summaries 
                      WHERE vcode = '{vcode}' and inc_wd = {inc_withdrawn};"#);
    let summ: VSummary = sqlx::query_as(&sql).fetch_one(pool).await
           .map_err(|e| AppError::SqlxError(e, sql))?;

    let mut header_txt = format!("{}\n\n\tVersion                  {vcode}\n\tDate                     {}",
        get_hdr_line("SUMMARY OF ROR DATASET"),summ.vdate);
    header_txt += &format!("\n\tDays since 29/04/24      {}", summ.vdays); 
    header_txt += &format!("\n\n\tTime data imported       {}\n\tReport generated         {}\n",
        import_dt.format("%Y-%m-%d %H:%M:%S"),Local::now().format("%Y-%m-%d %H:%M:%S"));
    append_to_file(output_file_path, &header_txt)?;

    let org_text = format!("\n\n\tORGANISATION NUMBERS{}number            %age\n\t{}\n\t{}{}{}{}\n\n", 
        " ".repeat(45), "-".repeat(88),
        get_data_and_pc_line("Active", summ.num_active,summ.num_recs),
        get_data_and_pc_line("Inactive", summ.num_inactive, summ.num_recs), 
        get_data_and_pc_line("Withdrawn", summ.num_withdrawn, summ.num_recs), 
        get_data_and_pc_line("Total", summ.num_recs, summ.num_recs));
    append_to_file(output_file_path, &org_text)?;

    let withdrawn_text = if inc_withdrawn {
        format!(r#"
        N.B. Withdrawn organisations have been RETAINED in the dataset 
        and the figures below reflect this. Thay apply to all ROR organisations, 
        whatever their status.
        The denominator for % organisations is therefore {}
        "#, summ.num_denom)
    } 
    else {
        format!(r#"
        N.B. Withdrawn organisations have been REMOVED from the main dataset
        The figures below reflect both active and inactive ROR organisations, but not
        those classed as 'withdrawn', i.e. those added to ROR in error or duplicated.  
        The denominator for % organisations is therefore {}.
        Data on withdrawn organisations, including successor
        organisations where relevant, can be found in the 'ppr.withdrawn' table.
        "#, summ.num_denom)
    };
    append_to_file(output_file_path, &withdrawn_text)?;
    
    let entity_txt = format!("\n\n\tENTITY NUMBERS{}number\n\t{}{}{}{}{}{}{}{}{}\n", 
                     " ".repeat(51), "-".repeat(88),
                     get_data_line("Organisations", summ.num_denom),
                     get_data_line("Names", summ.num_names),
                     get_data_line("Types", summ.num_types),
                     get_data_line("Links", summ.num_links),
                     get_data_line("External Ids", summ.num_ext_ids), 
                     get_data_line("Relationships", summ.num_rels), 
                     get_data_line("Locations", summ.num_locations), 
                     get_data_line("Domains", summ.num_domains));
    append_to_file(output_file_path, &entity_txt)?;

    let expl_text = format!("\n\n\t\tN.B. The totals under some columns in the tables are generated from
        the table data, and were included to provide a visual check on that data.
        
        The occasional very small deviations from 100% for percentage totals are because
        the source data are stored in the database to only 2 decimal places of accuracy.\n");
    append_to_file(output_file_path, &expl_text)?;
        
    Ok(())
}

fn get_data_and_pc_line(topic: &str, num: i32, total: i32) -> String {
    let spacer = " ".repeat(71 - topic.len() - num.to_string().len());
    let pc_as_string = format!("{:.2}", num*100 / total);
    let spacer2 = " ".repeat(15 - pc_as_string.len());
    format!("\n\t{topic}{spacer}{num}{spacer2}{pc_as_string}")
}

fn get_data_line(topic: &str, num: i32) -> String {
    let spacer = " ".repeat(71 - topic.len() - num.to_string().len());
    format!("\n\t{topic}{spacer}{num}")
}

async fn write_name_info(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, 
                         singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    append_to_file(output_file_path, &get_hdr_line("NAMES"))?;

    let s1 = &singvals["added_labels"]; 
    let s2 = &singvals["dup_names"]; 

    let name_change_text = format!("\n{}{}{}", get_sing_hdr(), 
        get_singleton_line(&s1.description, s1.number, s1.pc),
        get_singleton_line(&s2.description, s2.number, s2.pc));
    append_to_file(output_file_path, &name_change_text)?;

    // Name attribute summary - att_type 1

    let table_text = get_attrib_table(1, "Names", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Count distributions - count_type: names, labels, aliases, acronyms

    let table_text = get_distrib_table("names", "names", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("labels", "labels", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("aliases", "aliases", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("acronyms", "acronyms", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

async fn write_name_wolc_info(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>
                                                   , singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
    let s1 = &singvals["total_wolc"]; 
    let s2 = &singvals["nacro_wolc"];
    let s3 = &singvals["nacncmp_wolc"];

    let wolc_text1 = format!("\n{}{}{}", get_hdr_line("NAMES WITHOUT LANGUAGE CODE (w/o LCs)"), 
        get_sing_hdr(), get_singleton_line(&s1.description, s1.number, s1.pc));
    append_to_file(output_file_path, &wolc_text1)?;

    // Write name wolc attribute summary - att_type 11.

    let table_text = get_attrib_table(11, "Names without language codes (w/o LCs)", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let wolc_text2 = format!("\n{}{}{}", get_sing_hdr(),
        get_singleton_line(&s2.description, s2.number, s2.pc),
        get_singleton_line(&s3.description, s3.number, s3.pc));
    append_to_file(output_file_path, &wolc_text2)?;
    
    // org type and lang code data 

    let sql = format!(r#"select org_type, name_type, names_num, names_wolc, names_wolc_pc 
                 from smm.org_type_and_lang_code where vcode = '{vcode}' and inc_wd = {inc_withdrawn} order by 
                 org_type, name_type desc;"#);
    let rows: Vec<OrgAndLangCode> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    let mut tbl_text = format!("\n\n
    Numbers of name types without language codes for different organisational types
        
                                                       number         names          %age
    org type                     name type             names          w/o LC        w/o LC\n\t{}",                                                 
    "-".repeat(88));
    
    for r in rows {
        tbl_text += &get_orglc_line(&r.org_type, &r.name_type, r.names_num, r.names_wolc, r.names_wolc_pc);
    }
    tbl_text += "\n";
    append_to_file(output_file_path, &tbl_text)
}

async fn write_ranked_name_info(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, 
                                        singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("LANGUAGE AND SCRIPT USAGE"))?;

    // Singletons - Names that are not English

    let s1 = &singvals["names_ne"]; 
    let s2 = &singvals["acro_ne"];
    let s3 = &singvals["nacro_ne"];

    let s_text = format!("\n{}{}{}{}", get_sing_hdr(), 
        get_singleton_line(&s1.description, s1.number, s1.pc),
        get_singleton_line(&s2.description, s2.number, s2.pc),
        get_singleton_line(&s3.description, s3.number, s3.pc));
    append_to_file(output_file_path, &s_text)?;

    // Ranked languages other than English in use - Ranked Distributions, dist_type = 1.

    let mut tbl_text = format!("\n
                                                       number       %age non-en     %age non-
    By Language (non acronym names only)              of names        non-acr       acr names\n\t{}",                                                 
    "-".repeat(88));
    tbl_text += &get_ranked_distrib_table(1, vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &tbl_text)?;

    // Singleton - Names that are not in Latin script

    let s4 = &singvals["names_nl"]; 
    let s5 = &singvals["acro_nl"];
    let s6 = &singvals["nacro_nl"];

    let s_text = format!("\n{}{}{}{}", get_sing_hdr(), 
        get_singleton_line(&s4.description, s4.number, s4.pc),
        get_singleton_line(&s5.description, s5.number, s5.pc),
        get_singleton_line(&s6.description, s6.number, s6.pc));
    append_to_file(output_file_path, &s_text)?;

    // Ranked scripts other than Latin in use - Ranked Distributions, dist_type = 2.

    let mut tbl_text = format!("\n
                                                       number          %age           %age
    By Script                                         of names     non-ltn names  total names\n\t{}",                                                 
    "-".repeat(88));
    tbl_text += &get_ranked_distrib_table(2, vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &tbl_text)?;
    
    Ok(())
}


async fn write_ror_name_details(output_file_path: &PathBuf, singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    let s1 = &singvals["label_ror"]; 
    let s3 = &singvals["nlabel_ror"];
    let s4 = &singvals["ror_en"]; 
    let s5 = &singvals["ror_nen"];
    let s6 = &singvals["ror_wolc"];
    let s7 = &singvals["ror_wolc_ncmp"];

    let sng_text = format!("{}\n{}{}{}\n{}{}{}\n{}", get_hdr_line("ROR NAMES"), get_sing_hdr(), 
        get_singleton_line(&s1.description, s1.number, s1.pc),
        get_singleton_line(&s3.description, s3.number, s3.pc),
        get_singleton_line(&s4.description, s4.number, s4.pc),
        get_singleton_line(&s5.description, s5.number, s5.pc),
        get_singleton_line(&s6.description, s6.number, s6.pc),
        get_singleton_line(&s7.description, s7.number, s7.pc));
    append_to_file(output_file_path, &sng_text)
}


async fn write_type_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("ORGANISATION TYPES"))?;

    // Write type attribute summary - att_type 2

    let table_text = get_attrib_table(2, "Organisation types", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // write count distributions - count_type: org_types

    let table_text = get_distrib_table("org_types", "organisation types", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}


async fn write_location_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    let s1 = &singvals["poly_locs"];        // Poly locations, sub-divisions, countries
    let s2 = &singvals["poly_subdivs"];
    let s3 = &singvals["poly_countries"]; 

    let sng_text = format!("{}\n{}{}{}{}", get_hdr_line("LOCATIONS"), get_sing_hdr(), 
        get_singleton_line(&s1.description, s1.number, s1.pc),
        get_singleton_line(&s2.description, s2.number, s2.pc),
        get_singleton_line(&s3.description, s3.number, s3.pc));
    append_to_file(output_file_path, &sng_text)?;

    // Count distribution.

    let table_text = get_distrib_table("locs", "locations", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // Ranked countries - Ranked Distributions, dist_type = 3
    
    let mut tbl_text = format!("\n
    Number of locations by country:
    
                                                      number           %age           %age
    Country                                           of locs       NonUS locs     total locs\n\t{}",                                                 
    "-".repeat(88));
    tbl_text += &get_ranked_distrib_table(3, vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &tbl_text)?;

    Ok(())
}


async fn write_links_and_extid_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("EXTERNAL IDS AND LINKS"))?;

    // Write link attribute summary - att_type 4

    let table_text = get_attrib_table(4, "Links",vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write count distributions.

    let table_text = get_distrib_table("links", "links", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;
        
    // Write ext id attribute summary - att_type 3
    let table_text = get_attrib_table(3, "External Ids", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // Write count distribution.
    
    let table_text = get_distrib_table("ext_ids", "external ids", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

async fn write_relationship_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, 
                                     singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
   
    append_to_file(output_file_path, &get_hdr_line("RELATIONSHIPS"))?;

    // Write relationship attribute summary - att_type 5
    
    let table_text = get_attrib_table(5, "Relationships", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write count distributions.

    let table_text = get_distrib_table("parent orgs", "'has parent' relationships", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("child orgs", "'has child' relationships", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("related orgs", "'is related to' relationships", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;
    
    let table_text = get_distrib_table("predecessor orgs", "'has predecessor' relationships", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("successor orgs", "'has successor' relationshipss", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write out singleton values

    let s1 = &singvals["nrecip_pc"]; 
    let s2 = &singvals["nrecip_rr"];
    let s3 = &singvals["nrecip_ps"];

    let s_text = format!("\n{}{}{}{}", get_sing_hdr(), 
        get_singleton_line(&s1.description, s1.number, s1.pc),
        get_singleton_line(&s2.description, s2.number, s2.pc),
        get_singleton_line(&s3.description, s3.number, s3.pc));
    append_to_file(output_file_path, &s_text)?;
    
    // Org type and relationship data.

    let sql = format!(r#"select org_type, rel_type, num_links, num_orgs, num_orgs_pc 
            from smm.org_type_and_relationships 
            where vcode = '{vcode}' and inc_wd = {inc_withdrawn} order by org_type, rel_type;"#);
    let rows: Vec<OrgAndRel> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;                               
    
    let mut tbl_text = format!("\n
    Numbers of relationship links for different organisational types:
    
                                                        number        number          %age
    org type                      relationship          links          orgs         org type\n\t{}",                                                 
    "-".repeat(88));
    for r in rows {
        tbl_text += &get_orgrel_line(&r.org_type, &r.rel_type, r.num_links, r.num_orgs, r.num_orgs_pc);
    }
    tbl_text += "\n";
    append_to_file(output_file_path, &tbl_text)?;

    Ok(())
}

async fn write_domain_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<(), AppError> {
   
    append_to_file(output_file_path,  &get_hdr_line("DOMAINS"))?;
    
    let table_text = get_distrib_table("domains", "domains", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

fn get_hdr_line(topic: &str) -> String {
    format!("\n\n\t{}\n\t{topic}\n\t{}", "=".repeat(88),  "=".repeat(88),)
}

async fn get_attrib_table(att_type: i32, header_type: &str, 
                          vcode: &String,  inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = format!(r#"select name, number_atts, pc_of_atts, number_orgs, pc_of_orgs 
            from smm.attributes_summary
            where vcode = '{vcode}' and inc_wd = {inc_withdrawn} and att_type = {att_type} 
            order by id; "#);
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
        tbl_text += &get_attrib_line(&r.name, r.number_atts, r.pc_of_atts, r.number_orgs, r.pc_of_orgs);
        if !r.name.starts_with("nacro")  
        {
            rt_numbercat += r.number_atts;
            rt_numbercat_pc += r.pc_of_atts;
        }
    }
    tbl_text += &get_table_base(38, 13, rt_numbercat, rt_numbercat_pc);
    Ok(tbl_text)
}


fn get_attrib_line(category: &str, num_atts: i32, pc_atts: f32, num_orgs: i32, pc_orgs: f32)-> String {
    let spacer1 = " ".repeat(43 - category.chars().count() - num_atts.to_string().len());
    let pc1_as_string = format!("{:.2}", pc_atts);
    let spacer2 = " ".repeat(13 - pc1_as_string.len());
    let spacer3 = " ".repeat(15 - num_orgs.to_string().len());
    let pc2_as_string = format!("{:.2}", pc_orgs);
    let spacer4 = " ".repeat(15 - pc2_as_string.len());
    format!("\n\t{category}{spacer1}{num_atts}{spacer2}{pc1_as_string}{spacer3}{num_orgs}{spacer4}{pc2_as_string}")    
}

async fn get_distrib_table(count_type: &str, header_type: &str, vcode: &String, inc_withdrawn: bool, 
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
    Numbers of organisations with specified              count        number         %age
    count of {}{}                         orgs       total orgs\n\t{}",
    header_type, hdr_spacer, "-".repeat(88));

    for r in rows {
        tbl_text += &get_distrib_line(r.count, r.num_of_orgs, r.pc_of_orgs);
        rt_numberorgs += r.num_of_orgs;
        rt_numberorgs_pc += r.pc_of_orgs;
    }
    tbl_text += &get_table_base(66, 15, rt_numberorgs, rt_numberorgs_pc);
    Ok(tbl_text)
}

fn get_distrib_line(count: i32, num: i32, pc: f32)-> String {
    let spacer1 = " ".repeat(56 - count.to_string().len());
    let spacer2 = " ".repeat(15 - num.to_string().len());
    let pc_as_string = format!("{:.2}", pc);
    let spacer3 = " ".repeat(15 - pc_as_string.len());
    format!("\n\t{spacer1}{count}{spacer2}{num}{spacer3}{pc_as_string}")
}

async fn get_ranked_distrib_table(dist_type: i32, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<String, AppError> {

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

fn get_table_base(stop1: usize, stop2: usize, total: i32, total_pc: f32) -> String {
    let spacer1 = " ".repeat(stop1 - total.to_string().len());
    let pc_as_string = format!("{:.2}", total_pc);
    let spacer2 = " ".repeat(stop2 - pc_as_string.len());
    format!("\n\t{}\n\tTOTAL{spacer1}{total}{spacer2}{pc_as_string}\n", "-".repeat(88))
}


fn get_ranked_distrib_line(topic: &str, num: i32, pc1: f32, pc2: f32)-> String {
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

fn get_orglc_line(org_type: &str, name_type: &str, names_num: i32, names_wolc: i32, names_wolc_pc: f32) -> String {
    let spacer1 = " ".repeat(30 - org_type.chars().count());
    let spacer2 = " ".repeat(26 - name_type.chars().count()- names_num.to_string().len());
    let spacer3 = " ".repeat(15 - names_wolc.to_string().len());
    let pc_as_string = format!("{:.2}", names_wolc_pc);
    let spacer4 = " ".repeat(15 - pc_as_string.to_string().len());
    format!("\n\t{org_type}{spacer1}{name_type}{spacer2}{names_num}{spacer3}{names_wolc}{spacer4}{pc_as_string}")
}

fn get_orgrel_line(org_type: &str, rel_type: &str, num_links: i32, num_orgs: i32, num_orgs_pc: f32) -> String {
    let spacer1 = " ".repeat(30 - org_type.chars().count());
    let spacer2 = " ".repeat(26 - rel_type.chars().count() - num_links.to_string().len());
    let spacer3 = " ".repeat(15 - num_orgs.to_string().len());
    let pc_as_string = format!("{:.2}", num_orgs_pc);
    let spacer4 = " ".repeat(15 - pc_as_string.to_string().len());
    format!("\n\t{org_type}{spacer1}{rel_type}{spacer2}{num_links}{spacer3}{num_orgs}{spacer4}{pc_as_string}")
}

fn get_sing_hdr() -> String {
    "\n\n\t                                                                 number           %age\n".to_string()
}

fn get_singleton_line(topic: &str, num: i32, pc: Option<f32>) -> String {

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

fn append_to_file(output_file_path: &PathBuf, contents: &str) -> Result<(), AppError> {

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(output_file_path)?;

    file.write_all(contents.as_bytes())
        .map_err(|e| AppError::IoWriteErrorWithPath(e, output_file_path.to_owned()))
    
}

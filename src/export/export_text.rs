use sqlx::{Pool, Postgres};
use std::{collections::HashMap, path::PathBuf};
use crate::AppError;
use chrono::{DateTime, Local};
use super::export_structs::{VSummary, SingletonRow, Singleton};
use super::export_helpers::*;
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
    let summ = write_header(&output_file_path, vcode, inc_withdrawn, pool).await?;
    write_summary(&output_file_path, &summ, inc_withdrawn).await?;
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

async fn write_header(output_file_path: &PathBuf, vcode: &String, 
                      inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<VSummary, AppError> {
    
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
    Ok(summ)
}


async fn write_summary(output_file_path: &PathBuf, summ: &VSummary, inc_withdrawn: bool, ) -> Result<(), AppError> {

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
        N.B. Withdrawn organisations have been REMOVED from the main dataset.
        The figures below reflect both active and inactive ROR organisations, but not
        those classed as 'withdrawn', i.e. those added to ROR in error or duplicated.  
        The denominator for % organisations is therefore {}.
        Data on withdrawn organisations, including successor
        organisations where relevant, can be found in the 'ppr.withdrawn' table.
        "#, summ.num_denom)
    };
    append_to_file(output_file_path, &withdrawn_text)?;
    
    let entity_txt = format!("\n\n\tENTITY NUMBERS{}number\n\t{}\n{}{}{}{}{}{}{}{}\n", 
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


async fn write_name_info(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, 
                         singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    let name_change_text = format!("{}\n{}{}", 
        get_hdr_line("NAMES"), get_sing_hdr(), 
        get_singleton_rows(singvals, vec!["added_labels", "dup_names"]));  
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

    let wolc_text1 = format!("\n{}\n{}", get_hdr_line("NAMES WITHOUT LANGUAGE CODE (w/o LCs)"), 
        get_singleton_rows(singvals, vec!["total_wolc"]));  
    append_to_file(output_file_path, &wolc_text1)?;

    // Write name wolc attribute summary - att_type 11.

    let table_text = get_attrib_table(11, "Names without language codes (w/o LCs)", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let wolc_text2 = format!("{}{}", get_sing_hdr(),
        get_singleton_rows(singvals, vec!["nacro_wolc", "nacncmp_wolc"]));  
    append_to_file(output_file_path, &wolc_text2)?;
    
    // org type and lang code data 
    
    let table_text = get_org_type_and_lang_code_table(vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)
}



async fn write_ror_name_details(output_file_path: &PathBuf, singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    let sng_text = format!("{}\n{}{}\n{}\n{}\n", get_hdr_line("ROR NAMES"), get_sing_hdr(), 
        get_singleton_rows(singvals, vec!["label_ror", "nlabel_ror"]),
        get_singleton_rows(singvals, vec!["ror_en", "ror_nen", "ror_wolc"]),
        get_singleton_rows(singvals, vec!["ror_wolc_ncmp"]));
    append_to_file(output_file_path, &sng_text)
}

async fn write_ranked_name_info(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, 
                                        singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
    
    // Names that are not English.

    let s_text = format!("{}\n{}{}", get_hdr_line("LANGUAGE AND SCRIPT USAGE"), get_sing_hdr(), 
        get_singleton_rows(singvals, vec!["names_ne", "acro_ne", "nacro_ne"]));
    append_to_file(output_file_path, &s_text)?;

    // Ranked languages other than English in use - Ranked Distributions, dist_type = 1.

    let mut tbl_text = format!("\n
                                                       number       %age non-en     %age non-
    By Language (non acronym names only)              of names        non-acr       acr names\n\t{}",                                                 
    "-".repeat(88));
    tbl_text += &get_ranked_distrib_table(1, vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &tbl_text)?;

    // Names that are not in Latin script.

    let s_text = format!("\n{}{}", get_sing_hdr(), 
        get_singleton_rows(singvals, vec!["names_nl", "acro_nl", "nacro_nl"]));
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


async fn write_type_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("ORGANISATION TYPES"))?;

    // Write type attribute summary - att_type 2

    let table_text = get_attrib_table(2, "Organisation Types", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // Write count distributions - count_type: org_types.

    let table_text = get_distrib_table("org_types", "Organisation types", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write out funder types - atribute summary - type 7

    let funder_cotype_text1 = format!("\n\n\t\tIn most cases a 'funder' organisation has another type designation,
        refecting its primary role. The various primary roles for funder organisations 
        are tabulated below.\n");
    append_to_file(output_file_path, &funder_cotype_text1)?;
    
    let table_text = get_attrib_table(7, "Funder Co-types", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let funder_cotype_text2 = format!("\n\n\t\tN.B. A small number of funders have two additional type
        designations. Hence the total of co-types is slightly greater than the 
        total funder number, and the percentage is slightly greater than 100.\n");
    append_to_file(output_file_path, &funder_cotype_text2)?;

    Ok(())
}


async fn write_location_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>, singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    let sng_text = format!("{}\n{}{}", get_hdr_line("LOCATIONS"), get_sing_hdr(), 
        get_singleton_rows(singvals, vec!["poly_locs", "poly_subdivs", "poly_countries"]));
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

    let s_text = format!("\n{}{}", get_sing_hdr(), 
        get_singleton_rows(singvals, vec!["nrecip_pc", "nrecip_rr", "nrecip_ps"]));
    append_to_file(output_file_path, &s_text)?;
    
    // Org type and relationship data.

    let table_text = get_org_type_and_relationship_table(vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}


async fn write_domain_details(output_file_path: &PathBuf, vcode: &String, inc_withdrawn: bool, pool: &Pool<Postgres>) -> Result<(), AppError> {
   
    append_to_file(output_file_path,  &get_hdr_line("DOMAINS"))?;
    
    let table_text = get_distrib_table("domains", "domains", vcode, inc_withdrawn, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

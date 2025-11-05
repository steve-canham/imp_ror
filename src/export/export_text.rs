use sqlx::{Pool, Postgres};
use std::{collections::HashMap, path::PathBuf};
use crate::AppError;
use std::fs::OpenOptions;
use std::io::prelude::*;
use chrono::{NaiveDateTime, Local};
use super::export_structs::{VSummary, TypeRow, DistribRow, RankedRow, 
                            SingletonRow, Singleton, OrgAndLangCode, OrgAndRel};
use log::info;


pub async fn generate_text(output_folder : &PathBuf, data_version: &String, 
                            pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // If data version and date not given explicitly derive them from the data version table
    // as being the version, date of the currently stored version

    let mut vcode: String = data_version.clone();
    if vcode == "" {
        let sql = "SELECT version as vcode from ppr.version_details;";
        vcode = sqlx::query_scalar(sql).fetch_one(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    }

    // Get path and set up file for writing.

    let datetime_string = Local::now().format("%m-%d %H%M%S").to_string();
    let output_file_name = format!("{} summary at {}.txt", &vcode, &datetime_string);
    let output_file_path: PathBuf = [output_folder, &PathBuf::from(output_file_name)].iter().collect();
            
    let singvals:HashMap<String, Singleton> = collect_singleton_values(&vcode, pool).await?;
    write_header_and_summary(&output_file_path, &vcode, pool).await?;
    write_explanation(&output_file_path).await?;
    write_name_info(&output_file_path, &vcode, pool, &singvals).await?;
    write_name_wolc_info(&output_file_path, &vcode, pool, &singvals).await?;
    write_ror_name_details(&output_file_path, &singvals).await?;
    write_ranked_name_info(&output_file_path, &vcode, pool, &singvals).await?;
    write_type_details(&output_file_path, &vcode, pool).await?;
    write_location_details(&output_file_path, &vcode, pool, &singvals).await?;
    write_links_and_extid_details(&output_file_path, &vcode, pool).await?;
    write_relationship_details(&output_file_path, &vcode, pool, &singvals).await?;
    write_domain_details(&output_file_path, &vcode, pool).await?;

    info!("Content appended successfully");
    Ok(())
}

async fn collect_singleton_values(vcode: &String, pool: &Pool<Postgres>) -> Result<HashMap<String, Singleton>, AppError> {

    let mut sstructs = HashMap::new();
    let sql = r#"SELECT id, description, number, pc from smm.singletons WHERE vcode = '"#.to_string() + vcode  + r#"';"#;
    let srows: Vec<SingletonRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
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

async fn write_header_and_summary(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Get import date from the ror table, other summary detrails from the smm.version_summaries table

    let sql = "SELECT import_datetime from src.version_details;";
    let import_dt: NaiveDateTime = sqlx::query_scalar(sql).fetch_one(pool).await 
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = "SELECT * from smm.version_summaries WHERE vcode = \'".to_string() + &vcode  + "\' ;";
    let summ: VSummary = sqlx::query_as(&sql).fetch_one(pool).await
           .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let header_txt = get_hdr_line("SUMMARY OF ROR DATASET")
                   + "\n\n\tVersion: " + vcode  
                   + "\n\tDate: " + &summ.vdate.to_string() 
                   + "\n\tDays since 29/04/24: " + &summ.vdays.to_string()
                   + "\n"
                   + "\n\tTime data imported: " + &import_dt.format("%Y-%m-%d %H:%M:%S").to_string()
                   + "\n\tReport generated: " + &Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    append_to_file(output_file_path, &header_txt)?;

    let summary_txt = "\n\n\tENTITY NUMBERS\n\t".to_string() 
                    + "----------------------------------------------------------------------------------" 
                + &get_data_line("Organisations", summ.num_orgs) 
                + &get_data_line("Names", summ.num_names) 
                + &get_data_line("Types", summ.num_types) 
                + &get_data_line("Links", summ.num_links) 
                + &get_data_line("External Ids", summ.num_ext_ids) 
                + &get_data_line("Relationships", summ.num_rels) 
                + &get_data_line("Locations", summ.num_locations) 
                + &get_data_line("Domains", summ.num_domains) 
                + "\n";
    append_to_file(output_file_path, &summary_txt)?;
    Ok(())
}

async fn write_explanation(output_file_path: &PathBuf) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("ABBREVIATIONS AND TOTALS"))?;

    let expl_text = "\n\n\tN.B. In the data below:".to_string() 
    + "\n\t'nacro' refers to non-acronym names, i.e. labels and aliases considered"
    + "\n\ttogether."
    + "\n\t'cmps' refers to organisations designated as companies."
    + "\n\t'noncmp' refers to all organisations except companies."
    + "\n\t'nac-ncmp' refers to the non-acronym names of non-company organisations."
    + "\n\t'wolc' signifies names without a language code.";
    append_to_file(output_file_path, &expl_text)?;

    let expl_text = "\n\n\tThe totals under some columns in the tables are generated from".to_string() 
    + "\n\tthe table data, and were included to provide a visual check on that data."
    + "\n"
    + "\n\tThe occasional very small deviations from 100% for percentage totals are because"
    + "\n\tthe source data are stored in the database to only 2 decimal places of accuracy.";
    append_to_file(output_file_path, &expl_text)
    
}

async fn write_name_info(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>, 
                         singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {

    append_to_file(output_file_path, &get_hdr_line("NAMES"))?;

    let s1 = &singvals["added_labels"]; 
    let s2 = &singvals["dup_names"]; 

    let name_change_text = "\n".to_string() + &get_sing_hdr()+ &get_singleton_line(&s1.description, s1.number, s1.pc)
                           + &get_singleton_line(&s2.description, s2.number, s2.pc);

    append_to_file(output_file_path, &name_change_text)?;

    // Write name attribute summary - att_type 1

    let table_text = get_attrib_table(1, "Names", "TOTAL (excl. nacro lines)", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write count distributions - count_type: names, labels, aliases, acronyms

    let table_text = get_distrib_table("names", "names", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("labels", "labels", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("aliases", "aliases", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("acronyms", "acronyms", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

async fn write_name_wolc_info(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>
                                                   , singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
    let s1 = &singvals["total_wolc"]; 
    let s2 = &singvals["nacro_wolc"];
    let s3 = &singvals["nacncmp_wolc"];

    append_to_file(output_file_path, &get_hdr_line("NAMES WITHOUT LANGUAGE CODE (WOLC)"))?;
           
    let wolc_text = get_sing_hdr()+ &get_singleton_line(&s1.description, s1.number, s1.pc)
                           + &get_singleton_line(&s2.description, s2.number, s2.pc)
                           + &get_singleton_line(&s3.description, s3.number, s3.pc);
        
    append_to_file(output_file_path, &wolc_text)?;

    // Write name wolc attribute summary - att_type 11.

    let table_text = get_attrib_table(11, "Names without language codes (wolc)", "TOTAL (excl. nacro lines)", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // org type and lang code data 

    let sql = r#"select org_type, name_type, names_num, names_wolc, names_wolc_pc 
                 from smm.org_type_and_lang_code where vcode = '"#.to_string() + vcode + r#"' order by 
                 org_type, name_type;"#;
    let rows: Vec<OrgAndLangCode> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let mut tbl_text = "\n\n\tNumbers of name types without language codes for different organisational types:".to_string() 
                     + "\n\n\t                                           number          names           %age"
                       + "\n\torg type               name type            names           wolc           wolc"
                       + "\n\t----------------------------------------------------------------------------------";
    for r in rows {
        tbl_text += &get_orglc_line(&r.org_type, &r.name_type, r.names_num, r.names_wolc, r.names_wolc_pc);
    }
    tbl_text += "\n";
    append_to_file(output_file_path, &tbl_text)
}

async fn write_ranked_name_info(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>, 
                                                       singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("LANGUAGE AND SCRIPT USAGE"))?;

    // Singletons - Names that are not English

    let s1 = &singvals["names_ne"]; 
    let s2 = &singvals["acro_ne"];
    let s3 = &singvals["nacro_ne"];
    let s_text = get_sing_hdr() + &get_singleton_line(&s1.description, s1.number, s1.pc)
                   + &get_singleton_line(&s2.description, s2.number, s2.pc)
                   + &get_singleton_line(&s3.description, s3.number, s3.pc);
    append_to_file(output_file_path, &s_text)?;

    // Ranked languages other than English in use - Ranked Distributions, dist_type = 1.

    let tbl_hdr_text = "\n\n\t                                           number           %age           %age".to_string()
                       + "\n\tBy Language (nacro names only)            of names      non-en nacro    nacro names"
                       + "\n\t-----------------------------------------------------------------------------------";
    let tbl_text = get_ranked_distrib_table(1, vcode, pool).await?;
    append_to_file(output_file_path, &(tbl_hdr_text + &tbl_text))?;

    // Singleton - Names that are not in Latin script

    let s4 = &singvals["names_nl"]; 
    let s5 = &singvals["acro_nl"];
    let s6 = &singvals["nacro_nl"];
    let s_text = get_sing_hdr() + &get_singleton_line(&s4.description, s4.number, s4.pc)
                   + &get_singleton_line(&s5.description, s5.number, s5.pc)
                   + &get_singleton_line(&s6.description, s6.number, s6.pc);

    append_to_file(output_file_path, &s_text)?;

    // Ranked scripts other than Latin in use - Ranked Distributions, dist_type = 2.
    
    let tbl_hdr_text = "\n\n\t                                           number          %age           %age".to_string()
                       + "\n\tBy Script                                 of names     NonLtn names    total names"
                       + "\n\t----------------------------------------------------------------------------------";
    let tbl_text = get_ranked_distrib_table(2, vcode, pool).await?;
    append_to_file(output_file_path, &(tbl_hdr_text + &tbl_text))?;
    
    Ok(())
}


async fn write_ror_name_details(output_file_path: &PathBuf, singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("ROR NAMES"))?;
    
    // Write out singleton values, go from ids 10 to 16, requires index range (9..16).

    let s1 = &singvals["label_ror"]; 
    let s3 = &singvals["nlabel_ror"];
    let s4 = &singvals["ror_en"]; 
    let s5 = &singvals["ror_nen"];
    let s6 = &singvals["ror_wolc"];
    let s7 = &singvals["ror_wolc_ncmp"];
    let sng_text = get_sing_hdr()
      + &get_singleton_line(&s1.description, s1.number, s1.pc)
      + &get_singleton_line(&s3.description, s3.number, s3.pc)
      + "\n"
      + &get_singleton_line(&s4.description, s4.number, s4.pc)
      + &get_singleton_line(&s5.description, s5.number, s5.pc)
      + &get_singleton_line(&s6.description,s6.number, s6.pc)
      + "\n"
      + &get_singleton_line(&s7.description, s7.number, s7.pc);
    
    append_to_file(output_file_path, &sng_text)?;
    Ok(())
}

async fn write_type_details(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("ORGANISATION TYPES"))?;

    // Write type attribute summary - att_type 2

    let table_text = get_attrib_table(2, "Organisation types", "TOTAL", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // write count distributions - count_type: org_types

    let table_text = get_distrib_table("org_types", "organisation types", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

async fn write_location_details(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>, singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
       
    append_to_file(output_file_path, &get_hdr_line("LOCATIONS"))?;

    // Poly locations, sub-divisions, countries

    let s1 = &singvals["poly_locs"]; 
    let s2 = &singvals["poly_subdivs"];
    let s3 = &singvals["poly_countries"]; 

    let s_text = get_sing_hdr() + &get_singleton_line(&s1.description, s1.number, s1.pc)
                   + &get_singleton_line(&s2.description, s2.number, s2.pc)
                   + &get_singleton_line(&s3.description, s3.number, s3.pc);
    
    append_to_file(output_file_path, &s_text)?;

        // Count distribution.

    let table_text = get_distrib_table("locs", "locations", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // Ranked countries - Ranked Distributions, dist_type = 3
 
    let tbl_hdr_text ="\n\n\tNumber of locations by country:".to_string()
                    + "\n\n\t                                           number           %age           %age"
                      + "\n\tCountry                                    of locs       NonUS locs     total locs"
                      + "\n\t----------------------------------------------------------------------------------";
    let tbl_text = get_ranked_distrib_table(3, vcode, pool).await?;
    append_to_file(output_file_path, &(tbl_hdr_text + &tbl_text))?;

    Ok(())
}

async fn write_links_and_extid_details(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    append_to_file(output_file_path, &get_hdr_line("EXTERNAL IDS AND LINKS"))?;

    // Write link attribute summary - att_type 4

    let table_text = get_attrib_table(4, "Links", "TOTAL", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write count distributions.

    let table_text = get_distrib_table("links", "links", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;
        
    // Write ext id attribute summary - att_type 3
    let table_text = get_attrib_table(3, "External Ids", "TOTAL", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;
   
    // Write count distribution.
    
    let table_text = get_distrib_table("ext_ids", "external ids", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}

async fn write_relationship_details(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>, 
                                     singvals: &HashMap<String, Singleton>) -> Result<(), AppError> {
   
    append_to_file(output_file_path, &get_hdr_line("RELATIONSHIPS"))?;

    // Write relationship attribute summary - att_type 5
    
    let table_text = get_attrib_table(5, "Relationships", "TOTAL", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write count distributions.

    let table_text = get_distrib_table("parent orgs", "'has parent' relationships", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("child orgs", "'has child' relationships", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("related orgs", "'is related to' relationships", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;
    
    let table_text = get_distrib_table("predecessor orgs", "'has predecessor' relationships", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    let table_text = get_distrib_table("successor orgs", "'has successor' relationshipss", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    // Write out singleton values

    let s1 = &singvals["nrecip_pc"]; 
    let s2 = &singvals["nrecip_rr"];
    let s3 = &singvals["nrecip_ps"];
    let s_text =get_sing_hdr() + &get_singleton_line(&s1.description, s1.number, s1.pc)
                   + &get_singleton_line(&s2.description, s2.number, s2.pc)
                   + &get_singleton_line(&s3.description, s3.number, s3.pc);
    append_to_file(output_file_path, &s_text)?;

    // Org type and relationship data.

    let sql = r#"select org_type, rel_type, num_links, num_orgs, num_orgs_pc 
                 from smm.org_type_and_relationships 
                 where vcode = '"#.to_string() + vcode + r#"' order by org_type, rel_type;"#;

    let rows: Vec<OrgAndRel> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;                               
                                        
    let mut tbl_text = "\n\n\tNumbers of relationship links for different organisational types:".to_string() 
                     + "\n\n\t                                             number        number          %age"
                       + "\n\torg type               relationship          links          orgs         org type"
                       + "\n\t----------------------------------------------------------------------------------";
    for r in rows {
        tbl_text += &get_orgrel_line(&r.org_type, &r.rel_type, r.num_links, r.num_orgs, r.num_orgs_pc);
    }
    tbl_text += "\n";
    append_to_file(output_file_path, &tbl_text)?;

    Ok(())
}

async fn write_domain_details(output_file_path: &PathBuf, vcode: &String, pool: &Pool<Postgres>) -> Result<(), AppError> {
   
    append_to_file(output_file_path,  &get_hdr_line("DOMAINS"))?;
    
    let table_text = get_distrib_table("domains", "domains", vcode, pool).await?;
    append_to_file(output_file_path, &table_text)?;

    Ok(())
}


async fn get_attrib_table(att_type: i32, header_type: &str, total_text: &str, 
                          vcode: &String, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = r#"select name, number_atts, pc_of_atts, number_orgs, pc_of_orgs from smm.attributes_summary
    where vcode = '"#.to_string() + vcode + r#"' and att_type = "# + &att_type.to_string() + " order by id;";
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let mut tbl_text = "\n\n\t".to_string() + header_type + ", categories and numbers:"
          + "\n\n\t                              number         %age         number          %age"
            + "\n\tCategory                      in cat       all cats        orgs        total orgs"
            + "\n\t----------------------------------------------------------------------------------";
    let mut rt_numbercat: i32 = 0;
    let mut rt_numbercat_pc: f32 = 0.0;
    for r in rows {
        tbl_text = tbl_text + &get_attrib_line(&r.name, r.number_atts, r.pc_of_atts, r.number_orgs, r.pc_of_orgs);
        if !r.name.starts_with("nacro")  
        {
            rt_numbercat += r.number_atts;
            rt_numbercat_pc += r.pc_of_atts;
        }
    }
    tbl_text = tbl_text + "\n\t----------------------------------------------------------------------------------"
    + &get_attrib_line(total_text, rt_numbercat, rt_numbercat_pc, -1, -1.0);
    Ok(tbl_text + "\n")
}


fn get_attrib_line(category: &str, num_atts: i32, pc_atts: f32, num_orgs: i32, pc_orgs: f32)-> String {
    let spacer1 = " ".repeat(36 - category.chars().count() - num_atts.to_string().len());
    let pc1_as_string = format!("{:.2}", pc_atts);
    let spacer2 = " ".repeat(13 - pc1_as_string.len());
    let spacer3 = " ".repeat(15 - num_orgs.to_string().len());
    let pc2_as_string = format!("{:.2}", pc_orgs);
    let spacer4 = " ".repeat(15 - pc2_as_string.len());
    if num_orgs > 0 {
        "\n\t".to_string() + category + &spacer1 + &num_atts.to_string() 
           + &spacer2 + &pc1_as_string + &spacer3 + &num_orgs.to_string() + &spacer4 + &pc2_as_string
    }
    else {
        "\n\t".to_string() + category + &spacer1 + &num_atts.to_string() 
           + &spacer2 + &pc1_as_string
    }
}

async fn get_distrib_table(count_type: &str, header_type: &str, vcode: &String, 
                                             pool: &Pool<Postgres>) -> Result<String, AppError> {
let sql = r#"select count, num_of_orgs, pc_of_orgs from smm.count_distributions
                 where vcode = '"#.to_string() + vcode + r#"' and count_type = '"# + count_type + r#"' 
                 order by count;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let hdr_spacer = " ".repeat(33 - header_type.len());
    let mut tbl_text = "\n\n\tNumbers of organisations with specified       count        number         %age".to_string()
                       + "\n\tcount of " + header_type + &hdr_spacer + "                  orgs       total orgs"
                       + "\n\t----------------------------------------------------------------------------------";
    
    let mut rt_numberorgs: i32 = 0;
    let mut rt_numberorgs_pc: f32 = 0.0;
    for r in rows {
        tbl_text = tbl_text + &get_distrib_line(r.count, r.num_of_orgs, r.pc_of_orgs);
        rt_numberorgs += r.num_of_orgs;
        rt_numberorgs_pc += r.pc_of_orgs;
    }

    let spacer1 = " ".repeat(59 - rt_numberorgs.to_string().len());
    let pc_as_string = format!("{:.2}", rt_numberorgs_pc);
    let spacer2 = " ".repeat(15 - pc_as_string.len());

    tbl_text = tbl_text + "\n\t----------------------------------------------------------------------------------"
    + "\n\tTOTAL" + &spacer1 + &rt_numberorgs.to_string() + &spacer2 + &pc_as_string;
    Ok(tbl_text + "\n")
}

fn get_distrib_line(count: i32, num: i32, pc: f32)-> String {
    let count_as_string = " ".repeat(49 - count.to_string().len()) + &count.to_string();
    let spacer1 = " ".repeat(15 - num.to_string().len());
    let pc_as_string = format!("{:.2}", pc);
    let spacer2 = " ".repeat(15 - pc_as_string.len());
    "\n\t".to_string() + &count_as_string + &spacer1 + &num.to_string() 
           + &spacer2 + &pc_as_string
}

async fn get_ranked_distrib_table(dist_type: i32, vcode: &String, pool: &Pool<Postgres>) -> Result<String, AppError> {

    let sql = r#"SELECT entity, number, pc_of_entities, pc_of_base_set from smm.ranked_distributions 
                 where vcode = '"#.to_string() + vcode + r#"' and dist_type = "# + 
                 &dist_type.to_string() + " order by rank";
    let lang_rows: Vec<RankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    let mut tbl_text = "".to_string();
    let mut rt_numberents: i32 = 0;
    let mut rt_numberents_pc: f32 = 0.0;
    for r in lang_rows {
        tbl_text = tbl_text + &get_ranked_distrib_line(&r.entity, r.number, r.pc_of_entities, r.pc_of_base_set);
        if r.entity != "United States" {
            rt_numberents += r.number;
            rt_numberents_pc += r.pc_of_entities;
        }
    }
    tbl_text = tbl_text + "\n\t----------------------------------------------------------------------------------"
    + &get_ranked_distrib_line("TOTAL", rt_numberents, rt_numberents_pc, -1.0);
    Ok(tbl_text + "\n")
}

fn get_ranked_distrib_line(topic: &str, num: i32, pc1: f32, pc2: f32)-> String {
    let spacer1 = " ".repeat(49 - topic.chars().count() - num.to_string().len());
    let pc1_as_string = format!("{:.2}", pc1);
    let spacer2 = " ".repeat(15 - pc1_as_string.len());
    let pc2_as_string = format!("{:.2}", pc2);
    let spacer3 = " ".repeat(15 - pc2_as_string.len());
    if topic == "United States" {
           "\n\t".to_string() + topic + &spacer1 + &num.to_string() 
           + &" ".repeat(11) + "----"  + &spacer3 + &pc2_as_string
    }
    else {
        if pc2 > -0.1 {
            "\n\t".to_string() + topic + &spacer1 + &num.to_string() 
            + &spacer2 + &pc1_as_string + &spacer3 + &pc2_as_string
        }
        else {
            "\n\t".to_string() + topic + &spacer1 + &num.to_string() 
            + &spacer2 + &pc1_as_string
        }
    }
}
 
fn get_hdr_line(topic: &str) -> String {
    "\n\n\t==================================================================================".to_string()
    + "\n\t" + topic 
    + "\n\t=================================================================================="
}

fn get_sing_hdr() -> String {
    "\n\t                                                          number          %age\n".to_string()
}

fn get_data_line(topic: &str, num: i32) -> String {
    let spacer = " ".repeat(49 - topic.len() - num.to_string().len());
    "\n\t".to_string() + topic + &spacer + &num.to_string() 
}

fn get_orglc_line(org_type: &str, name_type: &str, names_num: i32, names_wolc: i32, names_wolc_pc: f32) -> String {
    let spacer1 = " ".repeat(23 - org_type.chars().count());
    let spacer2 = " ".repeat(26 - name_type.chars().count()- names_num.to_string().len());
    let spacer3 = " ".repeat(15 - names_wolc.to_string().len());
    let pc_as_string = format!("{:.2}", names_wolc_pc);
    let spacer4 = " ".repeat(15 - pc_as_string.to_string().len());
    "\n\t".to_string() + org_type + &spacer1 + name_type + &spacer2 + &names_num.to_string() 
           + &spacer3 + &names_wolc.to_string() + &spacer4 + &pc_as_string
}

fn get_orgrel_line(org_type: &str, rel_type: &str, num_links: i32, num_orgs: i32, num_orgs_pc: f32) -> String {
    let spacer1 = " ".repeat(23 - org_type.chars().count());
    let spacer2 = " ".repeat(26 - rel_type.chars().count() - num_links.to_string().len());
    let spacer3 = " ".repeat(15 - num_orgs.to_string().len());
    let pc_as_string = format!("{:.2}", num_orgs_pc);
    let spacer4 = " ".repeat(15 - pc_as_string.to_string().len());
    "\n\t".to_string() + org_type + &spacer1 + rel_type + &spacer2 + &num_links.to_string() 
           + &spacer3 + &num_orgs.to_string() + &spacer4 + &pc_as_string
}

fn get_singleton_line(topic: &str, num: i32, pc: Option<f32>) -> String {
    let spacer1 = " ".repeat(64 - topic.chars().count() - num.to_string().len());
    if pc.is_some() {
        let pc_as_string = format!("{:.2}", pc.unwrap());
        let spacer2 = " ".repeat(15 - pc_as_string.len());
        "\n\t".to_string() + topic + &spacer1 + &num.to_string() + &spacer2 + &pc_as_string
    }
    else {
        "\n\t".to_string() + topic + &spacer1 + &num.to_string() 
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

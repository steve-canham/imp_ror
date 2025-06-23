mod load;
mod prep;
mod names;
mod acros;

use sqlx::{Pool, Postgres};
use crate::AppError;
use log::info;


pub async fn load_data(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // These simply load the src data into matching tables in the 'ext' schema
    // of the ror DB. 
      
    load::create_ext_schema(pool).await?;
    load::load_orgs(pool).await?;
    load::load_names(pool).await?;
    load::load_rels(pool).await?;
    load::load_types(pool).await?;
    load::load_locs(pool).await?;
    load::reset_postgres_messaging(pool).await?;

    Ok(())
}


pub async fn prep_names(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // preparation of org names and addition of a 'name_to_match' field
    
    prep::remove_no_width_chars(pool).await?;
    prep::remove_peoples_space_in_names(pool).await?;

    // The 'name_to_match' form is lower-cased, shorn of full stops, 
    // commas and brackets, and has apostrophes replaced by single right quotes,
    // along with other 'standardising'  measures.

    prep::prepare_names_to_match(pool).await?;

    Ok(())
}


pub async fn apply_name_codes(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Ascribe source to those with an existing lang code

    update_lang_code_source("ror", pool).await?;

    // Update lang codes from scripts where possible, record lang code source type

    names::add_langs_for_nonlatin_codes(pool).await?;
    update_lang_code_source("script_auto", pool).await?;
    
    // If the org is a commercial company change the lang code to 'cm'
    // This makes it easier to see the gaps, though 'cm' needs to be added to the lang codes
    // This also over-rides any previous application of a language code to a company name
   
    names::add_cm_lang_code_to_comm_orgs(pool).await?;
    
    // Add languages if possible, using location of org and key words or word parts
    // Do language of acronyms where all other names have the same language
    // See what are left
  
    names::update_english_names(pool).await?;
    names::update_japanese_names(pool).await?;
    names::update_chinese_names(pool).await?;
    names::update_french_names(pool).await?;
    names::update_indian_names(pool).await?;
    names::update_iranian_names(pool).await?;
    names::update_russian_names(pool).await?;
    names::update_ukrainian_names(pool).await?;
    names::update_norwegian_names(pool).await?;
    names::update_serbian_names(pool).await?;
    names::update_bulgarian_names(pool).await?;
    names::update_israeli_names(pool).await?;
    names::update_korean_names(pool).await?;
    names::update_greek_names(pool).await?;

    update_lang_code_source("lex_auto", pool).await?;

        // israel
        // greece ?
        // korea
        // taiwan +
        // india +
        // russia +
        
    // Do acronym language codes....

    /*
    
    names::obtain_manual_coding_list(pool).await?;
    names::apply_manual_coding_list(pool).await?;

    update_lang_code_source("manual", pool).await?;

    // There are about 1600 names that begin with 'The '
    // These are often presented in source material without the 'The '.
    // 400 of them already include a name variant without the 'The ', but
    // this call results in the remaining 1200+ also having a 'the-less'
    // version of the name added.
    // This is done after lang codes have been applied to make sure maximum 
    // information is transferred to the new records

    prep::add_names_without_thes(pool).await?;

    // Need to modify the company data to make a single entry from multiple national 
    // subsidiaries - get companies in a parent child relationship - 
    // remove the children but possibly keep a name if it is different as an alt name. 
    // Keep the parent entry as 'the' company ROR entry.
*/
    Ok(())
}


pub async fn apply_acro_codes(_pool : &Pool<Postgres>) -> Result<(), AppError>
{

    // Need to consider acronyms (type = 10)
    // get all the acronyms into a table with the id, acronym
    // ???, and space for a matching name and language code

    // For each id where there is an acronym, get each name, name_to_match (?), language code, 
    // name minus 'the ', name minus ' of ', 
    // string with first letter of each word (minus 'the '), 
    // remove entries with just one word / initial letter
    // update table above with first letter of each word minus ' of ', where the name contains ' of '

    // match the atual acronyms with the derived acronyms, and create a table with the result.
    // Add to the table additional ercords where the acronym matches the 'of-less' records.
    // (might be some similar additions, e.g. removing ' and ')
    
    Ok(())
}


async fn update_lang_code_source(srce: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = format!(r#"update ext.names
            set lang_source = '{}'
            where lang_source is null
            and lang_code is not null;"#, srce );
 
    let res = sqlx::raw_sql(&sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql))?;
        info!("{} records updated with '{}' as language source", res.rows_affected(), srce);

    Ok(())
}

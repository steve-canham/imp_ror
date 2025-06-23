use sqlx::{Pool, Postgres};
use crate::AppError;
use log::info;


pub async fn remove_no_width_chars (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // remove any of the set of zero width characters 

    let mut no_width_chars = 0;
    no_width_chars += remove_unicode_from_names("200B", pool).await?;  // zero width space
    no_width_chars += remove_unicode_from_names("200C", pool).await?;  // zero width no join
    no_width_chars += remove_unicode_from_names("200D", pool).await?;  // zero width join
    no_width_chars += remove_unicode_from_names("200E", pool).await?;  // left-to-right mark
    no_width_chars += remove_unicode_from_names("200F", pool).await?;  // right-to-left mark
    no_width_chars += remove_unicode_from_names("2060", pool).await?;  // word joiner
    no_width_chars += remove_unicode_from_names("FEFF", pool).await?;  // zero width no-break space / BOM
    info!("{} no width characters removed from names to match", no_width_chars);
   
    Ok(())
}


pub async fn remove_peoples_space_in_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Some corrections of chinese "people 's" required

    let sql  = r#"update ext.names
            set name_to_match = replace(name_to_match, 'people ''s', 'people’s')
            where name_to_match like '%people ''s%'; "#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} 'people 's' errors fixed in names to match", res.rows_affected());

    Ok(())
}


pub async fn prepare_names_to_match  (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // punctuation

    info!("{} periods removed from names to match", remove_from_names(".", pool).await?);
    info!("{} commas removed from names to match", remove_from_names(",", pool).await?);
    info!("{} colons removed from names to match", remove_from_names(":", pool).await?);
    info!("{} semi-colons removed from names to match", remove_from_names(";", pool).await?);
    
    // brackets

    info!("{} left parantheses replaced by spaces in names to match", replace_in_names("(", " ", pool).await?);
    info!("{} right parantheses removed from names to match", remove_from_names(")", pool).await?);
    info!("{} left brackets replaced by spaces in names to match", replace_in_names("[", " ", pool).await?);
    info!("{} right brackets removed from names to match", remove_from_names("]", pool).await?);

    // double quotes
    
    info!("{} straight double quotes removed from names to match", remove_unicode_from_names("0022", pool).await?);
    info!("{} left curved double quotes removed from names to match", remove_unicode_from_names("201C", pool).await?);
    info!("{} right curved quotes removed from names to match", remove_unicode_from_names("201D", pool).await?);
    info!("{} left bottom quotes removed from names to match", remove_unicode_from_names("201E", pool).await?);
    info!("{} left upper reversed quotes removed from names to match", remove_unicode_from_names("201F", pool).await?);
    info!("{} left guillemets removed from names to match", remove_unicode_from_names("00AB", pool).await?);
    info!("{} right guillemets removed from names to match", remove_unicode_from_names("00BB", pool).await?);
    info!("{} twin single apostrophes removed from names to match", remove_from_names("''''", pool).await?);

    // single quotes

    info!("{} low single quotes changed to left single quotes", replace_unicode_in_names("201A", "‘", pool).await?);    
    info!("{} reverse single quotes changed to left single quotes", replace_unicode_in_names("201B", "‘", pool).await?);

    info!("{} modifier turned commas changed to left single quotes", replace_unicode_in_names("02BB", "‘", pool).await?);
    info!("{} modifier apostrophes changed to right single quotes", replace_unicode_in_names("02BC", "’", pool).await?);
    info!("{} modifier reversed commas changed to left single quotes", replace_unicode_in_names("02BD", "‘", pool).await?);
    info!("{} right half rings changed to right single quotes", replace_unicode_in_names("02BE", "’", pool).await?);
    info!("{} left half rings changed to left single quotes", replace_unicode_in_names("02BF", "‘", pool).await?);

    // standardise spaces

    info!("{} non breaking spaces changed to spaces", replace_unicode_in_names("00A0", " ", pool).await?);
    info!("{}  m quad spaces changed to spaces", replace_unicode_in_names("2001", " ", pool).await?);
    info!("{}  m spaces changed to spaces", replace_unicode_in_names("2002", " ", pool).await?);
    info!("{}  n spaces changed to spaces", replace_unicode_in_names("2003", " ", pool).await?);
    info!("{}  punctuation spaces changed to spaces", replace_unicode_in_names("2008", " ", pool).await?);
    info!("{}  ideographic spaces changed to spaces", replace_unicode_in_names("3000", " ", pool).await?);
  
    // apostrophes 
    // At beginning, or after space changed to left single quotes, otherwise to right single quotes

    info!("{} full width apostrophes changed to apostrophes", replace_unicode_in_names("FF01", "''", pool).await?);
    info!("{} apostrophes after spaces changed to left single quotes", replace_in_names(" ''", " ‘", pool).await?);

    let sql  = r#"update ext.names
            set name_to_match = '‘'||substring(name_to_match, 2)
            where name_to_match like '''%'; "#;
    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} apostrophes at beginning of name changed to left single quotes", res.rows_affected());

    info!("{} remaining apostrophes changed to right single quotes", replace_in_names("''", "’", pool).await?);
    
    // bullet points

    info!("{} bullets changed to spaces in names to match", replace_unicode_in_names("2022", " ", pool).await?);
    info!("{} hyphen bullets changed to spaces in names to match", replace_unicode_in_names("2043", " ", pool).await?);
    info!("{} raised dots changed to spaces in names to match", replace_unicode_in_names("2219", " ", pool).await?);
    info!("{} small square changes to spaces in names to match", replace_unicode_in_names("25AA", " ", pool).await?);
    info!("{} katakana middle dots changed to spaces in names to match", replace_unicode_in_names("30FB", " ", pool).await?);

    // standardise hyphens

    info!("{} hyphens changed to ascii hyphens in names to match", replace_unicode_in_names("2010", "-", pool).await?);
    info!("{} non-breaking hyphens changed to hyphens in names to match", replace_unicode_in_names("2011", "-", pool).await?);
    info!("{} figure dashes changed to hyphens in names to match", replace_unicode_in_names("2012", "-", pool).await?);
    info!("{} n dashes changed to hyphens in names to match", replace_unicode_in_names("2013", "-", pool).await?);
    info!("{} m dashes changed to hyphens in names to match", replace_unicode_in_names("2014", "-", pool).await?);
    info!("{} horizontal bars changed to hyphens in names to match", replace_unicode_in_names("2015", "-", pool).await?);

    // standardise hyphen spacing

    info!("{} left spaces removed from hyphens", replace_in_names(" -", "-", pool).await?);
    info!("{} right spaces removed from hyphens", replace_in_names("- ", "-", pool).await?);
   

    info!("{} double spaces replaced by single in names to match", replace_in_names("  ", " ", pool).await?);

    // Not currently required - may need to check periodically 
    // info!("{}  3 per m spaces changed to spaces", replace_unicode_char_in_names("2004", " ", pool).await?);
    // info!("{}  4 per m spaces changed to spaces", replace_unicode_char_in_names("2005", " ", pool).await?);
    // info!("{}  6 per m spaces changed to spaces", replace_unicode_char_in_names("2006", " ", pool).await?);
    // info!("{}  figure spaces changed to spaces", replace_unicode_char_in_names("2007", " ", pool).await?);
    // info!("{}  thin spaces changed to spaces", replace_unicode_char_in_names("2009", " ", pool).await?);
    // info!("{}  hair spaces changed to spaces", replace_unicode_char_in_names("200A", " ", pool).await?);
    // info!("{}  narrow non breaking spaces changed to spaces", replace_unicode_char_in_names("202F", " ", pool).await?);
    // info!("{}  medium mathematical spaces changed to spaces", replace_unicode_char_in_names("205F", " ", pool).await?);

    // final trim of name 

    let sql = r#"update ext.names 
    set name_to_match = trim(name_to_match);"#;
    sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    Ok(())
}


async fn remove_from_names(char: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ext.names
            set name_to_match = replace(name_to_match, '{}', '')
            where name_to_match like '%{}%'; "#, char, char);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn replace_in_names(char: &str, rep_str: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ext.names
            set name_to_match = replace(name_to_match, '{}', '{}')
            where name_to_match like '%{}%'; "#, char, rep_str, char);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn remove_unicode_from_names(unicode: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ext.names
            set name_to_match = replace(name_to_match, U&'\{}', '')
            where name_to_match like U&'%\{}%'; "#, unicode, unicode);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn replace_unicode_in_names(unicode: &str, rep_str: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ext.names
            set name_to_match = replace(name_to_match, U&'\{}', '{}')
            where name_to_match like U&'%\{}%'; "#, unicode, rep_str, unicode);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}

/* 
pub async fn add_names_without_thes(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"insert into ext.names (id, name, name_to_match, name_type, lang_code, lang_source, script_code)
        select n.* from 
            (select id, 
                substring(name, 5, length(name) - 4) as name,
                substring(name_to_match, 5, length(name_to_match) - 4) as name_to_match,
                2 as name_type, lang_code, lang_source, script_code
            from ext.names
            where name_to_match like 'the %'
            and array_length(string_to_array(name_to_match, ' '), 1) > 2) as n
        left join 
        ext.names r
        on n.id = r.id
        and n.name_to_match = r.name_to_match
        where r.id is null;"#;

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} additional name records added with initial 'the ' removed", res.rows_affected());
  
    Ok(())
}
*/


use sqlx::{Pool, Postgres};
use crate::AppError;
use log::info;

pub async fn apply_script_codes(pool: &Pool<Postgres>) -> Result<(), AppError> {

    prepare_names_for_script_codes(pool).await?;
    add_script_codes(pool).await?;
    clean_japanese_script_codes(pool).await?;
    clean_double_script_codes(pool).await?;
    apply_script_codes_to_names(pool).await?;

    Ok(())
}

async fn prepare_names_for_script_codes(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Set up the 'names_pad' table - a 'scratch pad' in whichg to calculate 
    // script codes - as a copy of the value (name) column.

    let sql = r#"Insert into ppr.names_pad (id, original_name, name, lang_code, script_code)
            select id, value, value, lang_code, ''
            from ppr.names; "#;

    sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("Names copied for processing prior to script coding");

    // Scripts are mainbly applied by examining the unicode of name characters
    // and matching them against the different unicode pages for different scripts.

    // First remove characters classed as 'latin' that could be in non latin names, 
    // that would otherwise give a 'false-latin' result.
    
    let mut punctuation = 0;                        // commas, semi-colons and full stops
    punctuation += remove_char(".", pool).await?;
    punctuation += remove_char(",", pool).await?;
    punctuation += remove_char(";", pool).await?;
    punctuation += remove_char(":", pool).await?;
    info!("{} commas, full stops, colons and semi-colons removed from name copies", punctuation);

    let mut brackets = 0;                           // parentheses and brackets
    brackets += remove_char("(", pool).await?;
    brackets += remove_char(")", pool).await?;
    info!("{} parantheses characters removed from name copies", brackets);

    let mut brackets = 0;
    brackets += remove_char("[", pool).await?;
    brackets += remove_char("]", pool).await?;
    info!("{} bracket characters removed from name copies", brackets);

    let res = remove_char("\"", pool).await?;        // double quotes, apostrophes, guillemets
    info!("{} double quotes removed from name copies", res);
    let res  = remove_char("''", pool).await?;
    info!("{} apostrophes removed from name copies", res);
    let mut guillemets = 0;
    guillemets += remove_unicode_char("00AB", pool).await?;
    guillemets += remove_unicode_char("00BB", pool).await?;
    info!("{} guillemets characters removed from name copies", guillemets);

    let mut punctuation = 0;                       // Hyphens, ampersands, slashes
    punctuation += remove_char("-", pool).await?;
    punctuation += remove_char("&", pool).await?;
    punctuation += remove_char("/", pool).await?;
    punctuation += remove_char("|", pool).await?;
    info!("{} sundry punctuation removed from name copies", punctuation);
   
    let mut bullets = 0;                            // Bullets
    bullets += remove_char("·", pool).await?;       // middle dot, U+00b7
    bullets += remove_char("・", pool).await?;      // katakana middle dot, U+30fb
    info!("{} Bullets removed from name copies", bullets);
  
    // Finally remove all underscores and spaces
    // (underscore removal affects all records as it acts as a wildcard)

    remove_char("_", pool).await?;  
    remove_char(" ", pool).await?; 
    info!("spaces removed from name copies");

    Ok(())
}


async fn remove_char(char: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ppr.names_pad
            set name = replace(name, '{}', '')
            where name like '%{}%'; "#, char, char);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn remove_unicode_char(unicode: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ppr.names_pad
            set name = replace(name, U&'\{}', '')
            where name like U&'%\{}%'; "#, unicode, unicode);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn add_script_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {
  
    // Examines the names and looks at the Unicode value of its first character. Uses that to 
    // determine the script (but checks for leading bracket - if present use the second character)
    
    #[derive(sqlx::FromRow)]
    struct Script {
        code: String,
        ascii_start: i32,
        ascii_end: i32,
        hex_start: String, 
        hex_end: String,
    }

    // Get the Unicode scripts with their hex code boundaries.

    let sql  = r#"select code, ascii_start, ascii_end, hex_start, hex_end
    from lup.lang_scripts
    where ascii_end <> 0
    order by ascii_start;"#;

    let unicodes: Vec<Script> = sqlx::query_as(sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("Unicode script characteristics obtained");

    let mut n = 0;
    for r in unicodes {

        // In most cases (hex boundaries <=4 in length), a regex can be used against 
        // the range to add the script name to 'script_code' if any character matches.
        
        if r.hex_start.len() < 5 {
            let sql  = format!(r#"update ppr.names_pad
                    set script_code = script_code||', '||'{}' 
                    where name ~ '[\u{:0>4}-\u{:0>4}]'"#, r.code, r.hex_start, r.hex_end);

            sqlx::query(&sql).execute(pool).await
                .map_err(|e| AppError::SqlxError(e, sql))?;
        }
        else {
            
            // In a few (very obscure) cases hex boundaries are > 4 in length and
            // the regex cannot be used - instead the initial characters is tested.
            
            let sql  = format!(r#"update ppr.names_pad
            set script_code = script_code||', '||'{}'  
            where ascii(substr(name, 1, 1)) >= {}
            and ascii(substr(name, 1, 1)) <= {}"#, r.code, r.ascii_start, r.ascii_end);
    
            sqlx::query(&sql).execute(pool).await
                .map_err(|e| AppError::SqlxError(e, sql))?;
        }

        n +=1;
        if n % 10 == 0 {
            info!("{} scripts processed...", n.to_string());
        }
    }

    // Remove the initial ', '.
    
    let sql  = r#"update ppr.names_pad          
    set script_code = substring(script_code, 3)
    where length(script_code) > 3 "#;

    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Simplify where only extended latin has been used.

    let sql  = r#"update ppr.names_pad
        set script_code = 'Latn'
        where script_code in ('Latn, Latn2')"#;

    let res = sqlx::query(sql).execute(pool).await
         .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} records with extended latin content simplified", res.rows_affected()); 

    Ok(())
}


async fn clean_japanese_script_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Japanese is a writing system that uses three different scripts.
    // Names may include 1,2 or all 3 of these scripts. Scripts 'Kana' and 'Hira' 
    // are specific to Japan - 'Hani' is also used in Chinese and related lamguages
    
    let mut japanese_nonlatin_names = 0;

    let sql  = r#"update ppr.names_pad
    set script_code = 'Jpan'
    where script_code in ('Kana', 'Hira', 'Hira, Kana, Hani')"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = 'Jpan'
    where lang_code = 'ja' 
    and script_code = 'Hani'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = 'Jpan'
    where script_code in ('Kana, Hira', 'Hira, Kana', 'Kana, Hani', 'Hira, Hani')"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = 'Latn, Jpan'
    where script_code like 'Latn, %'
    and (script_code like '%Kana%'
        or script_code like '%Hira%')"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = 'Latn, Jpan'
    where lang_code = 'ja' 
    and script_code like 'Latn, Hani%'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    info!("{} japanese non-latin scripts recoded to 'Jpan'", japanese_nonlatin_names); 

    Ok(())
}


async fn clean_double_script_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Many names that ostensibly have two scripts have only a very small portion 
    // in the minority script - sometimes just a numeral or two. This routine
    // identifies the different portions of the mixed-script names to allow their 
    // characterisation. In many cases the scripts listed are simplified to a single
    // script, but genuine mixed script names are retained as such.
    
    let sql  = r#"update ppr.names_pad n
    set latin = combined_array
    from
        (SELECT id, name, array_to_string(array_agg(latin), '') AS combined_array
        FROM 
            (select id, name, 
            REGEXP_MATCHES(name,'[\u0000-\u02FF]+', 'g') as latin
            from ppr.names_pad
            where length(script_code) > 4
            and script_code like '%Latn%') as t
        GROUP BY id, name ) m
        where n.id = m.id
        and n.name = m.name"#;

    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        
    let sql  = r#"update ppr.names_pad n
    set nonlatin = combined_array
    from
        (SELECT id, name, array_to_string(array_agg(nonlatin), '') AS combined_array
        FROM 
            (select id, name, 
            REGEXP_MATCHES(name,'[\u0300-\uD800]+', 'g') as nonlatin
            from ppr.names_pad
            where length(script_code) > 4
            and script_code like '%Latn%') as t
        GROUP BY id, name ) m
        where n.id = m.id
        and n.name = m.name"#;

    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // if latin are just numbers in russian, greek, arabic - 
    // make the script_code the non-latin script

    let mut rga_names = 0;

    let sql  = r#"update ppr.names_pad
    set script_code = 'Cyrl'
    where script_code = 'Latn, Cyrl'
    and latin ~ '^\d*$'"#;
    
    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = 'Grek'
    where script_code = 'Latn, Grek' 
    and latin ~ '^\d*$'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = 'Arab'
    where script_code =  'Latn, Arab'
    and latin ~ '^\d*$'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    info!("{} Russian, Greek and Arabic names with numbers recoded", rga_names);

    // For Ukranian and Byelorussian names, 'i' and 'ý' seems to be allowed (not in Russian)
    // and is therefore not an indicator of a latin script

    let sql  = r#"update ppr.names_pad
    set latin = replace(latin, 'i', '')
    where latin like '%i%' 
    and lang_code in ('be', 'uk'); "#;

    sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql  = r#"update ppr.names_pad
    set latin = replace(latin, 'ý', '')
    where latin like '%ý%' 
    and lang_code in ('be', 'uk'); "#;

    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Recode double scripts with only a very small (relatively) amount of 
    // one script to be the major script exclusively

    let mut singletons = 0;

    let sql  = r#"update ppr.names_pad
    set script_code = 'Latn'
    where length(script_code) > 4
    and char_length(nonlatin) < 3 
    and length(latin) > 5 "#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    singletons += res.rows_affected();

    let sql  = r#"update ppr.names_pad
    set script_code = substring(script_code, 7)
    where length(script_code) > 4
    and length(latin) < 3 
    and char_length(nonlatin) > 5"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    singletons += res.rows_affected();

    info!("{} Double scripted names with relatively short second script characters recoded", singletons); 

    let sql  = r#"select count(*) 
    from ppr.names_pad
    where length(script_code) > 4"#;

    let res : i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} names found using two or more scripts", res); 

    Ok(())
}


async fn apply_script_codes_to_names (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql  = r#"update ppr.names n
    set script_code = p.script_code
    from ppr.names_pad p
    where n.id = p.id
    and n.value = p.original_name"#;

    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("Language script codes applied to names table"); 
    info!(""); 

    // Remove ppr.names_pad
    
    let sql  = r#"drop table ppr.names_pad"#;
    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(())
}
 

use sqlx::{Pool, Postgres};
use crate::AppError;
use log::info;

   //to do --- clean_double_script_codes(pool).await?;

pub async fn prepare_match_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // initially construct the match value as a copy of the lang_value

    let sql = r#"update rec.names 
        set match_name = lang_name; "#;
    sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("lang names copied to match names");
    info!("match names further simplified and standardised");
    
    // remaining dashes need to be standardised to hyphens
    // and the spaces arund them removed, and hyphen spacing regiularised
    
    replace_unicode_char("2013", 504, "n dash", "-", pool).await?;  
    replace_unicode_char("2014", 505, "m dash", "-", pool).await?;  
    replace_unicode_char("2015", 506, "horizontal bar", "-", pool).await?; 
    replace_chars("- ", "-", 507, pool).await?;
    replace_chars(" -", "-", 508, pool).await?;
   
    // Most punctuation already removed, when constructimn the lang_names.
    // Needs a bit more standardisation.
    // N.B.The same processes MUST be applied to input names
  
    // Simplify the match value a little more

    remove_chars("&", 510, pool).await?;
    remove_chars("·", 511, pool).await?;       // middle dot, U+00b7
    remove_chars("・", 512, pool).await?;      // katakana middle dot, U+30fb
    replace_unicode_char("005f", 504, "underscore", " ", pool).await?;  
    
    // make double spaces single...

    replace_chars("  ", " ", 514, pool).await?; 
        
    // some simple stop words removed...
    
    replace_chars(" and ", " ", 520, pool).await?; 
    replace_chars(" et ", " ", 521, pool).await?;
    replace_chars(" und ", " ", 522,  pool).await?;
    replace_chars(" y ", " ", 523,  pool).await?;
    replace_chars(" of ", " ", 524,  pool).await?;
    replace_chars(" the ", " ", 525,  pool).await?;
    replace_chars(" for ", " ", 526,  pool).await?;
    replace_chars(" de ", " ", 527,  pool).await?;
    replace_chars(" le ", " ", 528,  pool).await?;
    replace_chars(" la ", " ", 529, pool).await?;
    replace_chars(" les ", " ", 530, pool).await?;
    replace_chars(" des ", " ", 531, pool).await?;
    replace_chars(" del ", " ", 532, pool).await?;

    // remove initial 'the' unless it is the first of two words

    let sql  = r#"update rec.names
        set match_name = regexp_replace(match_name, '^the ', '')
        where match_name ~ '^the '
        and array_length(string_to_array(match_name, ' '), 1) > 2 "#;

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();
    info!("{res} initial 'the's removed from match_names");

    replace_chars("  ", " ", 514, pool).await?;   // make double spaces single (again)...
    info!("");
    Ok(())
}

pub async fn prepare_script_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Remove spaces from the match_name and transfer the result to the script_name
    
    let sql  = r#"update rec.names
            set script_name = replace(match_name, ' ', ''); "#;
    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();
    
    info!("{res} script_names created");
    info!("");
    Ok(())
}
 

async fn remove_chars(chars: &str, rep_type: i32, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let ch_type = format!("({chars}) removed from match_name");
    
    let sql  = format!(r#"update rec.names
            set match_name = replace(match_name, '{chars}', ''),
            changed = true,
            change_type_id = case when change_type_id is null then '{rep_type}'
                else change_type_id||', '||'{rep_type}'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where match_name like '%{chars}%'; "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        if n == 1 {
            info!("{ch_type} ({})", "1 record");
        } 
        else {
            info!("{ch_type} ({})", format!("{n} records").as_str());
        };
    }
    Ok(())
}


async fn replace_chars(chars: &str, replacement: &str, rep_type: i32, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let ch_type = if replacement == "" {
        format!("({chars}) replaced by single space in match_name")
    }
    else {
        format!("({chars}) replaced by ({replacement}) in match_name")
    };
    
    let sql  = format!(r#"update rec.names
            set match_name = replace(match_name, '{chars}', '{replacement}'),
            changed = true,
            change_type_id = case when change_type_id is null then '{rep_type}'
                else change_type_id||', '||'{rep_type}'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where match_name like '%{chars}%'; "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        if n == 1 {
            info!("{ch_type} ({})", "1 record");
        } 
        else {
            info!("{ch_type} ({})", format!("{n} records").as_str());
        };
    }
    Ok(())
}


async fn replace_unicode_char(unicode_char: &str, rep_type: i32, char_description: &str, 
    replacement: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {
   
    let ch_type = if replacement == "-" {
        format!("(\\u{unicode_char}, {char_description}) replaced by ascii hyphen in match name")
    }
    else {
        format!("(\\u{unicode_char}, {char_description}) replaced by ({replacement}) in match name")
    };
            
    let sql  = format!(r#"update rec.names
            set match_name = replace(match_name, U&'\{unicode_char}', '{replacement}'),
            changed = true,
            change_type_id = case when change_type_id is null then '{rep_type}'
                else change_type_id||', '||'{rep_type}'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where match_name ~ U&'\{unicode_char}'; "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        if n == 1 {
            info!("{ch_type} ({})", "1 record");
        } 
        else {
            info!("{ch_type} ({})", format!("{n} records").as_str());
        };
    }
    Ok(())
}


pub async fn add_script_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {
  
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
            let sql  = format!(r#"update rec.names
                    set der_script = der_script||', '||'{}' 
                    where script_name ~ '[\u{:0>4}-\u{:0>4}]'"#, r.code, r.hex_start, r.hex_end);

            let res = sqlx::query(&sql).execute(pool).await
                .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

            if res > 0 {
                info!("{res} records assigned '{}' script", r.code);
            }
        }
        else {
            
            // In a few (very obscure) cases hex boundaries are > 4 in length and
            // the regex cannot be used - instead the initial characters is tested.
            
            let sql  = format!(r#"update rec.names
            set der_script = der_script||', '||'{}'  
            where ascii(substr(script_name, 1, 1)) >= {}
            and ascii(substr(script_name, 1, 1)) <= {}"#, r.code, r.ascii_start, r.ascii_end);
    
            sqlx::query(&sql).execute(pool).await
                .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        }

        n +=1;
        if n % 10 == 0 {
            info!("{} scripts processed...", n.to_string());
        }
    }

    // Remove the initial ', '.
    
    let sql  = r#"update rec.names         
    set der_script = substring(der_script, 3)
    where length(der_script) > 3 "#;

    sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Simplify where only extended latin has been used.

    let sql  = r#"update rec.names   
        set der_script = 'Latn'
        where der_script in ('Latn, Latn2')"#;

    let res = sqlx::query(sql).execute(pool).await
         .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} records with extended latin content simplified", res.rows_affected()); 

    Ok(())
}


pub async fn clean_japanese_script_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Japanese is a writing system that uses three different scripts.
    // Names may include 1,2 or all 3 of these scripts. Scripts 'Kana' and 'Hira' 
    // are specific to Japan - 'Hani' is also used in Chinese and related lamguages
    
    let mut japanese_nonlatin_names = 0;

    let sql  = r#"update rec.names  
    set der_script = 'Jpan'
    where der_script in ('Kana', 'Hira', 'Hira, Kana, Hani')"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update rec.names  
    set der_script = 'Jpan'
    where lang = 'ja' 
    and der_script = 'Hani'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update rec.names  
    set der_script = 'Jpan'
    where der_script in ('Kana, Hira', 'Hira, Kana', 'Kana, Hani', 'Hira, Hani')"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update rec.names  
    set der_script = 'Latn, Jpan'
    where der_script like 'Latn, %'
    and (der_script like '%Kana%'
        or der_script like '%Hira%')"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update rec.names  
    set der_script = 'Latn, Jpan'
    where lang = 'ja' 
    and der_script like 'Latn, Hani%'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    info!("{} japanese non-latin scripts recoded to 'Jpan'", japanese_nonlatin_names); 

    info!("");
    Ok(())
}


/* 

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

*/
 
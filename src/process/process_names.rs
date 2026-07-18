use sqlx::{Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn clean_names1 (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // replace some silly things

    let n = replace_chars("''''", "\"", pool).await?;
    info!("Paired single apostrophes replaced by double quotes in {n} records");
   
    let n = replace_chars("[править | править вики-текст]", "", pool).await?;
    info!("'[%править | править вики-текст]', translated as 'edit | edit wiki-text' removed in {n} records");
    
    let n = replace_chars("[ Citation needed | edit wiki text ]", "", pool).await?;
    info!("'[ Citation needed | edit wiki text ]', removed in {n} records");

    
    

    // Apostrophes
   
    info!("{} apostrophes in names, to begin with", apos_num(pool).await?);

    // The Uzbek language includes a single left quote (though this seems to be being phased out now)
    
    let ch_type = format!("Uzbek apostrophe replaced by ‘ left single quote (usually O‘)");
    let sql  = format!(r#"update src.names
            set value = replace(value, '''', '‘'),
            changed = true,
            change_type = '{ch_type}'
            where value like '%''%'  and lang = 'uz'"#);
    let n = execute_sql(&sql, pool).await?;
    info!("Uzbek apostrophe replaced by ‘ in {n} records");

    // Consider names with both left and right limitinh apostrophes, usually quoting names
    // (May need a closer look!)

    let ch_type = format!("Left hand apostrophe of pair changed to left single quote");
    let sql  = format!(r#"update src.names
            set value = replace(value, ' ''', ' ‘'),
            changed = true,
            change_type = '{ch_type}'
    where orig_value like '% ''%' 
    and (orig_value like '%'' %' or orig_value like '%''')"#);
    let n = execute_sql(&sql, pool).await?;
    info!("First apostrophe of pair changed to left hand quote in {n} records");
    
    let ch_type = format!("Left hand apostrophe of pair changed to left single quote");
    let sql  = format!(r#"update src.names
            set value = replace(value, '''', '‘'),
            changed = true,
            change_type = '{ch_type}'
    where orig_value like '''%'
    and (orig_value like '%'' %' or orig_value like '%''')"#);
    let n = execute_sql(&sql, pool).await?;
    info!("First apostrophe of pair at start of name changed to left hand quote in {n} records");
    
    let ch_type = format!("Right hand apostrophe of pair changed to right single quote");
    let sql  = format!(r#"update src.names
            set value = replace(value, '''', ''),
            changed = true,
            change_type = '{ch_type}'
    where (orig_value like '% ''%' or orig_value like '''%')
    and orig_value like '%'' %'"#); 
    let n = execute_sql(&sql, pool).await?;
    info!("Last apostrophe of pair changed to right hand quote in {n} records");
    
    let ch_type = format!("Right hand apostrophe of pair changed to right single quote");
    let sql  = format!(r#"update src.names
            set value = replace(value, '''', ''),
            changed = true,
            change_type = '{ch_type}'
    where (orig_value like '% ''%' or orig_value like '''%')
    and orig_value like '%'''"#);
    let n = execute_sql(&sql, pool).await?;
    info!("Last apostrophe of pair at end of name changed to right hand quote in {n} records");

    // Simple replacements
    
    let n = replace_chars("Hawai''i", "Hawai‘i", pool).await?;
    info!("(Hawai'i) replaced by (Hawai‘i) in {n} records");
        
    let n = replace_chars("eople ''s", "eople’s", pool).await?;   // Odd Chinese names
    info!("(eople 's) replaced by (eople’s) in {n} records");

    let n = replace_chars("''s", "’s", pool).await?;    // Singular possessives
    info!("('s) replaced by (’s) in {n} records");

    let n = replace_chars("s'' ", "s’ ", pool).await?;   // Plural possessives
    info!("(s' ) replaced by (s’ ) in {n} records");

    let n = replace_chars(" l''", " l’", pool).await?;  // mostly French
    info!("( l') replaced by ( l’) in {n} records");

    let n = replace_chars("-l''", "-l’", pool).await?;
    info!("(-l') replaced by (-l’) in {n} records");
    
    let n = replace_chars("L''", "L’", pool).await?;
    info!("(L') replaced by (L’) in {n} records");

    let n = replace_chars("d''", "d’", pool).await?;
    info!("(d') replaced by (d’) in {n} records");

    let n = replace_chars("D''", "D’", pool).await?;
    info!("(D') replaced by (D’) in {n} records");
    
    let n = replace_chars("ell''", "ell’", pool).await?;    // ell, all, ull,
    info!("(ll') replaced by (ll’) in {n} records");

    let n = replace_chars("ca'' ", "ca’ ", pool).await?;    // italian for Casa
    info!("(ca' ) replaced by (ca’ ) in {n} records");

    let n = replace_chars("Ca'' ", "Ca’ ", pool).await?;
    info!("(Ca' ) replaced by (Ca’ ) in {n} records");

    let n = replace_chars(" ''t", " ’t", pool).await?;  // Used in Dutch for 'het'
    info!("( 't) replaced by ( ’t) in {n} records");
    
    let n = replace_chars("O''", "O’", pool).await?;    // Mostly Irish names
    info!("(O') replaced by (O’) in {n} records");

    let n = replace_chars("ant''", "ant’", pool).await?;   // as in Sant', in Italian, Portugese etc.
    info!("(ant') replaced by (ant’) in {n} records");

    let n = replace_chars("''45", "’45", pool).await?;
    info!("('45) replaced by (’45) in {n} records");

    let n = replace_chars("c''est", "c’est", pool).await?;
    info!("(c'est) replaced by (c’est) in {n} records");

    let n = replace_chars("I''m", "I’m", pool).await?;
    info!("(I'm) replaced by (I’m) in {n} records");

    let n = replace_chars("donn''ees", "données", pool).await?;
    info!("(donn'ees) replaced by (données) in {n} records");

    //ctive'inside
    
    let n = replace_chars("Bao''an", "Bao^an", pool).await?;
    info!("(Bao'an) temporarily replaced by (Bao^an) in {n} records");

    let n = replace_chars("Xi''an", "Xi^an", pool).await?;
    info!("(Xi'an) temporarily replaced by (Xi^an) in {n} records");

    let n = replace_chars("Ya''an", "Ya^an", pool).await?;
    info!("(Ya'an) temporarily replaced by (Ya^an) in {n} records");
    
    
    info!("{} apostrophes in names after processing", apos_num(pool).await?);
    info!("");

    // Most of the remaining apostrophes uses to indicate syllable boundaries 
    // in transliterated Chinese, Japanee, Arabic
    // Should be retained as apostrophes

    let n = replace_chars("^", "''", pool).await?;
    info!("(^) resored back to (') in {n} records");
    
    Ok(())
}



async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let res = sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(res.rows_affected())
}

/* 
async fn remove_chars(char: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update src.names
            set value = trim(replace(value, '{char}', ''))
            where value like '%{char}%'; "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn remove_unicode_char(unicode_char: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update src.names
            set name = trim(replace(value, U&'\{unicode_char}', ''))
            where value like U&'%\{unicode_char}%'; "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}
*/

async fn replace_chars(chars: &str, replacement: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let ch_type = format!("({chars}) replaced by ({replacement})");
    let sql  = format!(r#"update src.names
            set value = replace(value, '{chars}', '{replacement}'),
            changed = true,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like '%{chars}%' "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}

/* 
async fn replace_unicode_char(unicode_char: &str, replacement: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update src.names
            set value = replace(name, U&'\{unicode_char}', '{replacement}')
            where value like U&'%\{unicode_char}%'; "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}
*/

async fn apos_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from src.names 
    where value like '%''%'"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(r)
}


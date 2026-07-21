use sqlx::{Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn clean_names1 (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // remnove invisible characters
    
    remove_unicode_char("200B", "zero width space", pool).await?;
    remove_unicode_char("200C", "zero width no join", pool).await?;
    remove_unicode_char("200D", "zero width join",pool).await?;
    remove_unicode_char("200E", "left-to-right mark", pool).await?;
    remove_unicode_char("200F", "right-to-left mark", pool).await?;
    remove_unicode_char("2060", "word joiner", pool).await?;
    remove_unicode_char("FEFF", "byte order mark", pool).await?;
    remove_unicode_char("00AD", "soft hyphen", pool).await?;

    // replace possible 'odd' spaces (though don't seem to occur)
    
    replace_unicode_char("00A0", "non-breaking space", " ", pool).await?;  
    replace_unicode_char("2002", "m space", " ", pool).await?;  
    replace_unicode_char("2003", "n space", " ", pool).await?;  
    replace_unicode_char("2008", "punctuation space", " ", pool).await?;  
    replace_unicode_char("3000", "ideographic space", " ", pool).await?;  

    // standardise hyphens

    replace_unicode_char("2010", "non ascii hyphen", "-", pool).await?;  
    replace_unicode_char("2011", "non-breaking hyphen", "-", pool).await?;  
    replace_unicode_char("2012", "figure dash", "-", pool).await?;  
    replace_unicode_char("2013", "n dash", "-", pool).await?;  
    replace_unicode_char("2014", "m dash", "-", pool).await?;  
    replace_unicode_char("2015", "horizontal bar", "-", pool).await?;  

    // consider square brackets
    // ?? considr ampoersands
    // 
    // Do double spaces to single at end?
    // info!("{} double spaces replaced by single in names to match", replace_in_names("  ", " ", pool).await?);
    
    
    // repair or remove some very specific oddities
      
    let n = replace_chars("[править | править вики-текст]", "", pool).await?;
    info!("'[%править | править вики-текст]', translated as 'edit | edit wiki-text' removed in {n} records");
    
    let n = replace_chars("[ Citation needed | edit wiki text ]", "", pool).await?;
    info!("'[ Citation needed | edit wiki text ]', removed in {n} records");

    let n = replace_chars(" (Rybářství Litomyšl)", "", pool).await?;
    info!("Spurious repeated text removed in {n} records");

    let n = replace_chars("?>", "->", pool).await?;
    info!("Incorrect arrow formula replaced in {n} records");

    /* to add 
    update src.names set value = replace(value, '[', '') where value like '%['
    update src.names set value = replace(value, ';', '') where value like '%;'
    
    update src.names set value = translate(value, '[]', '')
    where orig_value like '%]' and orig_value like '[%'
    -- no equivalent gfor paranthese or curly btrackets
    
    update src.names 
    set value = replace(value, '[', '') 
    where value like '%[%' and value not like '%]%'
    
    update src.names 
    set value = replace(value, ']', '') 
    where value like '%]%' and value not like '%[%'
    */
    // Apostrophes

    //  First put all double quotes and equivalents as straight double quotes
    // (necessary to correct pre-existing errors)
    
    replace_quotes("“", "\"", pool).await?;
    replace_quotes("”", "\"", pool).await?;
    replace_quotes("«", "\"", pool).await?;
    replace_quotes("»", "\"", pool).await?;
    replace_quotes("„", "\"", pool).await?;
    replace_quotes("''''", "\"", pool).await?;

    /*
     * -- do before above
     * update src.names 
     set value = replace(value, ',,', '„')
     where value like '%,,%'
     */

    // need to add here repolapcing two commas by the low r quotes
    
    info!("{} names with double quotes, to begin with", double_quotes_num(pool).await?);

    // replace double doubles with single "" -> " (2 recs)
    // consider those records (2) with 5 "
    // In both cases drop the 5th " to make it records with 4 "
    // update src.names 
    // set value = trim(regexp_replace(value, '"', '', 1, 5))
    // where length(value) - length(replace(value, '"', '')) = 5

    // consider those records (32) with 3 "
    // Which one to drop will depend on specific record - selct by id

    /*
    update src.names 
    set value = trim(regexp_replace(value, '"', '', 1, 1))
    where length(value) - length(replace(value, '"', '')) = 3
    and id in('019j1v294', '01hprsv49', '01mp7gg57', '01vd5cb71', '020whct63', '028mtfb17', '02b47v767', '03dx8n755', '03q57f308', '03qc6zh37' , '03wn3aq07', '049j4jr36', '04a7dp661', '057tmwv53', '05kzawq90', '05pkv9t98', '05q23ne91', '05svms055')
    
    update src.names 
    set value = trim(regexp_replace(value, '"', '', 1, 3))
    where length(value) - length(replace(value, '"', '')) = 3
    and id in ('00aa7ab77', '00kysjz64', '00qbdg904', '00wsvb073', '013fj3d42', '033z59547', '03b0cj417', '03xdgrg08', '05pc7fv53')

    do before to make sure all hebrew names identified (they are at present)
    update src.names
    set lang = 'he' 
    where  value ~ '[\u0590-\u05FF]'
    and lang <> 'he'

    change a double quote to gershayim (u05F4)
    if it is the only doouble quoyte in the name

    update src.names
    set value = replace(value, '"', U&'\05F4')
    where lang = 'he'
    and length(value) - length(replace(value, '"', '')) = 1
   
    
    The geresh 〈׳〉, is the Hebrew equivalent of a period in abbreviations (e.g. abbrev.), in addition to being attached to Hebrew letters to indicate sounds like soft g [dʒ] and ch [tʃ] in foreign names such as Charles (צ׳ארלס‎) and Jake (ג׳ייק‎). The gershayim 〈״〉, is a Hebrew symbol indicating that a sequence of characters is an acronym, and is placed before the last character of the word. Owing to a Hebrew keyboard's having neither a geresh nor gershayim, they are usually replaced online with, respectively, the visually similar apostrophe 〈'〉 and quotation mark 〈"⟩. The quotation mark and apostrophe are higher than the geresh and gershayim: where the latter are placed level with the top of Hebrew letters, the apostrophe and quotation marks are above them.

    Then can consider names with just a single doble quote
    In many cases add an additional quote to the end, but not in all

    
    -- put in front
    update src.names
    set value = '"'||value
    where id in ('00a9b0g29', '00vrtwn56', '01g7a7y43', '03mgprp21', '052q58629', '05bpnjz66')
    and length(value) - length(replace(value, '"', '')) = 1
    
    -- lose
    update src.names
    set value = replace(value, '"', '')
    where id in ('04cnfv189')
    and length(value) - length(replace(value, '"', '')) = 1
    
    -- put behind
    update src.names
    set value = value||'"'
    where length(value) - length(replace(value, '"', '')) = 1
    
    
    Three more specific oddities

    update src.names set value = 'Polemikí Aeroporía'
    where value = 'Polemikí Aeroporía, literally "Military Aviation"'
    
    update src.names set value = 'Public Komatsu University'
    where value = 'literally Public Komatsu University'
    
    update src.names set value = replace(value, '... ', '')
    where value like '%... %'

    -- Finally change all the paired double quotes to 'proper' 66 -- 99 quotes
    
    update src.names 
    set value = regexp_replace(value, '"(.*)"(.*)"(.*)"', '“\1”\2“\3”') 
    where length(value) - length(replace(value, '"', '')) = 4
    
    update src.names 
    set value = regexp_replace(value, '"(.*)"', '“\1”') 
    where length(value) - length(replace(value, '"', '')) = 2
    
    // ???? put left and right quote choices in the config file...
    // US pattern is the default but others can ber used...
    
    Ensure quotes are 'tight' to the words

    update src.names
    set value = trim(replace(value, '“ ', ' “')) 
    where value like '%“ %'
    
    update src.names
    set value = trim(replace(value, ' ”', '” '))
    where value like '% ”%'

    // (after paired single quotes have been done)
    // do a final replace with the user's selected qupte marks , if necessary

    
    */

    
    // Consider names with paired double quotes
    // The problem is that several have three double quotes, a pair and an 'odd' one

    


    
    //  Then put all single quotes and equivalents as straight apostrophes
    // (necessary to correct pre-existing errors)
    
    replace_quotes("‘", "''", pool).await?;
    replace_quotes("’", "''", pool).await?;
    
    info!("{} names with apostrophes, to begin with", apos_num(pool).await?);

    
    /*
     
    -- 's
    Much of this already sorted...
    update src.names set value = replace(value, 'eople ''s', 'eople’s')
    where value like '%eople ''s%'
    --7

    update src.names 
    set value = regexp_replace(value, '([a-zA-Z0-9])''s ', '\1’s ') 
    where value ~ '[a-zA-Z0-9]''s '
    --2479
    
    update src.names 
    set value = regexp_replace(value, '([a-zA-Z0-9])''s$', '\1’s') 
    where value ~ '[a-zA-Z0-9]''s$'
    --41

    -- another odd one
    update src.names 
    set value = replace(value, 'Breeders''Association', 'Breeders’ Association')
    where value ~ 'Breeders''Association'
    --2

    update src.names 
    set value = regexp_replace(value, 's''', 's’')
    where value ~ 's'' '
    or value ~ 's''$'
    --217
    
    -- d'
    -- do some odd ones first
    update src.names
    set value = replace(value, ' d'' ', ' d’')
    where value like '% d'' %' 
    --5

    update src.names
    set value = regexp_replace(value, '([ eou-])d''([AÁEÉHIÎOUXY])', '\1d’\2', 'gi')
    where value ~* '([ eou-])d''([AÁEÉHIÎOUXY])'
    --1730

    update src.names
    set value = regexp_replace(value, '^D''([AEÉHIÎOUXY])', 'D’\1', 'i')
    where value ~* '^d''([AEÉHIÎOUXY])'
    --3

    --l'
    update src.names
    set value = regexp_replace(value, '([ l])l'' ' , '\1l’')
    where value ~ '[ l]l'' '  
    --5

    update src.names
    set value = regexp_replace(value, '^L'' ' , 'L’')
    where value ~ '^L'' '
    --1
    
    update src.names
    set value = regexp_replace(value, '([ l-])l''([AÁEÉèHIÎOlœUXY])', '\1l’\2', 'gi')
    where value ~* '([ l-])l''([AÁEÉèHIÎOœUXY])'
    --1291
    
    update src.names
    set value = regexp_replace(value, '^l''([AÁEÉHIÎOUXY])', 'L’\1', 'gi')
    where value ~* '^l''([AÁEÉHIÎOUXY])'
    --84
    
     */
    
    
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


    /* 
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
    */
    
    // Simple replacements
    
    let n = replace_chars("Hawai''i", "Hawai‘i", pool).await?;
    info!("(Hawai'i) replaced by (Hawai‘i) in {n} records");
        
    //let n = replace_chars("eople ''s", "eople’s", pool).await?;   // Odd Chinese names
    //info!("(eople 's) replaced by (eople’s) in {n} records");

    //let n = replace_chars("''s", "’s", pool).await?;    // Singular possessives
    //info!("('s) replaced by (’s) in {n} records");

    //let n = replace_chars("s'' ", "s’ ", pool).await?;   // Plural possessives
    //info!("(s' ) replaced by (s’ ) in {n} records");

    //let n = replace_chars(" l''", " l’", pool).await?;  // mostly French
    //info!("( l') replaced by ( l’) in {n} records");

    //let n = replace_chars("-l''", "-l’", pool).await?;
    //info!("(-l') replaced by (-l’) in {n} records");
    
    //let n = replace_chars("L''", "L’", pool).await?;
    //info!("(L') replaced by (L’) in {n} records");

    //let n = replace_chars("d''", "d’", pool).await?;
    //info!("(d') replaced by (d’) in {n} records");

    //let n = replace_chars("D''", "D’", pool).await?;
    //info!("(D') replaced by (D’) in {n} records");
    
    //let n = replace_chars("ell''", "ell’", pool).await?;    // ell, all, ull,
    //info!("(ll') replaced by (ll’) in {n} records");

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
    
    
    info!("{} names with apostrophes after processing", apos_num(pool).await?);
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
*/

async fn remove_unicode_char(unicode_char: &str, char_description: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let ch_type = format!("(\\u{unicode_char}, {char_description}) removed");
    let sql  = format!(r#"update src.names
            set value = trim(replace(value, U&'\{unicode_char}', '')),
            changed = true,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like U&'%\{unicode_char}%'; "#);
     
    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?.rows_affected();

    if n > 0 {
        info!("{char_description} characters removed from {n} records");
    }

    Ok(())
}


async fn replace_quotes(chars: &str, replacement: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {
   
    let sql  = format!(r#"update src.names
            set value = replace(value, '{chars}', '{replacement}'),
            where value like '%{chars}%' "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?.rows_affected();

    Ok(n)
}


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


async fn replace_unicode_char(unicode_char: &str, char_description: &str, 
    replacement: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let ch_type = format!("(\\u{unicode_char}, {char_description}) replaced by ({replacement})");
    let sql  = format!(r#"update src.names
            set value = replace(name, U&'\{unicode_char}', '{replacement}'),
            changed = true,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like U&'%\{unicode_char}%'; "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?.rows_affected();

    if n > 0 {
        info!("{char_description} characters replaced by ({replacement}) in {n} records");
    }

    Ok(())
}


async fn apos_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from src.names 
    where value like '%''%'"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(r)
}

async fn double_quotes_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from src.names 
    where value like '%"%'"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(r)
}


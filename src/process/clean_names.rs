use sqlx::{Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn create_rec_names (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"insert into rec.names(ident, id, orig_name, display_name, 
       name_type, is_ror_name, ror_lang)
       select ident, id, value, value, 
       case 
           when name_type = 'label' then 5
           when name_type = 'alias' then 7
           when name_type = 'acronym' then 10
           else 0
       end,
       case
           when is_ror_name = true then true
           else false
       end,
       lang
       from src.names;"#;

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} records created in rec.names table", res.rows_affected()); 
 
    Ok(())
}

pub async fn create_countries (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"insert into src.countries(id, country_code)
        select distinct id, country_code
        from src.locations;"#;
    
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} country records created", res.rows_affected()); 
    Ok(())
}

pub async fn update_rec_names_with_country_data  (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update rec.names rn
    set num_countries = c.n
    from (
        select id, count(country_code) as n
        from src.countries 
        group by id) c
    where rn.id = c.id;"#;

    sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = r#"update rec.names r
        set country_code = c.country_code
        from src.countries c
        where r.id = c.id
        and r.num_countries = 1;"#;

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} Country codes inserted into rec.names table", res.rows_affected());
    info!(""); 
    Ok(())
}


pub async fn remove_invisible_chars (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // remnove invisible characters
    info!("Removing Invisible Characters"); 
    replace_unicode_char("200B", 1, "zero width space", "", pool).await?;
    replace_unicode_char("200C", 1, "zero width no join", "", pool).await?;
    replace_unicode_char("200D", 1, "zero width join", "", pool).await?;
    replace_unicode_char("200E", 1, "left-to-right mark", "", pool).await?;
    replace_unicode_char("200F", 1, "right-to-left mark", "", pool).await?;
    replace_unicode_char("2060", 1, "word joiner", "", pool).await?;
    replace_unicode_char("FEFF", 1, "byte order mark", "", pool).await?;
    replace_unicode_char("00AD", 1, "soft hyphen", "", pool).await?;

    // replace possible 'odd' spaces (though don't seem to occur)
    
    replace_unicode_char("00A0", 2, "non-breaking space", " ", pool).await?;  
    replace_unicode_char("2002", 2, "m space", " ", pool).await?;  
    replace_unicode_char("2003", 2, "n space", " ", pool).await?;  
    replace_unicode_char("2008", 2, "punctuation space", " ", pool).await?;  
    replace_unicode_char("3000", 2, "ideographic space", " ", pool).await?;  

    // standardise hyphens

    replace_unicode_char("2010", 3, "non ascii hyphen", "-", pool).await?;  
    replace_unicode_char("2011", 3, "non-breaking hyphen", "-", pool).await?;  
    replace_unicode_char("2012", 3, "figure dash", "-", pool).await?;  
    info!(""); 
    Ok(())
}


pub async fn repair_typos (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // deal with some very specific oddities (clearing them out of the way)
    
    info!("Repairing Typos and Oddities");   
    replace_chars("[править | править вики-текст]", "", "'[%править | править вики-текст]', translated as 'edit | edit wiki-text' removed", 10, pool).await?;
    replace_chars("[ Citation needed | edit wiki text ]", "", "'[ Citation needed | edit wiki text ]', removed", 11, pool).await?;
    replace_chars(" (Rybářství Litomyšl)", "", "Spurious repeated text 'Rybářství Litomyšl' removed", 12, pool).await?;
    replace_chars("?>", "->", "Incorrect arrow formula replaced in ~", 13, pool).await?;
    replace_chars("Polemikí Aeroporía, literally \"Military Aviation\"", "Polemikí Aeroporía", "'literally' folowed by translation removed", 14, pool).await?;
    replace_chars("literally Public Komatsu University", "Public Komatsu University", "'literally' removed", 15, pool).await?;
    replace_chars("... ", "", "ellipsis removed", 16, pool).await?;

    let sql = r#"update rec.names set display_name = replace(display_name, '[', '') where display_name like '%['"#;
    execute_sql(sql, "final left bracket removed", 21, pool).await?;
    let sql = r#"update rec.names set display_name = replace(display_name, ';', '') where display_name like '%;'"#;
    execute_sql(sql, "final semi-colon removed", 22, pool).await?;
    let sql = r#"update rec.names set display_name = translate(display_name, '[]', '')
    where display_name like '%]' and display_name like '[%'"#;
    execute_sql(sql, "paired outer brackets removed", 23, pool).await?;

    //  N.B. No current equivalent for paranthese or curly btrackets
    
    let sql = r#"update rec.names set display_name = replace(display_name, '[', '') 
    where display_name like '%[%' and display_name not like '%]%'"#;
    execute_sql(sql, "unpaired left bracket removed", 24, pool).await?;
    let sql = r#"update rec.names set display_name = replace(display_name, ']', '') 
    where display_name like '%]%' and display_name not like '%[%'"#;
    execute_sql(sql,"unpaired right bracket removed", 25, pool).await?;

    replace_chars("I'information", "l''information", "I'information repaired", 31, pool).await?;
    replace_chars("I'industrie", "l''industrie", "I'industrie repaired", 31, pool).await?;
    replace_chars("I'INSU", "l''INSU", "I'INSU repaired", 31, pool).await?;

    let sql = r#"update rec.names set display_name = replace(display_name, 'eople ''s', 'eople''s')
    where display_name like '%eople ''s%'"#;
    execute_sql(sql, "name with odd ‘people 's’ repaired", 32, pool).await?;

    let sql = r#"update rec.names set display_name = replace(display_name, ' d'' ', ' d’')
    where display_name like '% d'' %'"#; 
    execute_sql(sql, "d' followed by a space re-attached to following word", 32, pool).await?;
    
    replace_chars("Children's' ", "Children''s ", "name with odd ‘Children's'’ repaired", 33, pool).await?;
    replace_chars("Seiryo WOMEN'S ", "Seiryo Women''s ", "name with odd ‘WOMEN'S’ repaired", 34, pool).await?;
    replace_chars("Women'S ", "Women''s ", "name with odd ‘women'S’ repaired", 35, pool).await?;
    replace_chars("Breeders'Association", "Breeders'' Association", "name with ‘Breeders'Association’ repaired", 36, pool).await?;
    
    replace_chars("THE Japan WRITERS' Association", "The Japan Writers'' Association", "name with ‘THE and WRITERS’ repaired", 37, pool).await?;
    replace_chars("Japan WRITERS' Association", "Japan Writers'' Association", "name with ‘WRITERS’ repaired", 37, pool).await?;
    replace_chars("SEAMEN'S Employment", "Seamen''s Employment", "name with ‘SEAMEN'S’ repaired", 37, pool).await?;
    replace_chars("Glass MANUFACTURERS' ", "Glass Manufacturers'' ", "name with ‘MANUFACTURERS’ repaired", 37, pool).await?;

    replace_chars("'М.Д. Інститут кардіології ім. Стражеска", "''М.Д. Інститут кардіології ім. Стражеска''", "apostrophe added to 'М.Д. Інститут кардіології ім. Стражеск", 38, pool).await?;
    
    replace_chars("'Scientific and Research Institute Voskhod", "''Scientific and Research Institute Voskhod''", "apostrophe added to 'Scientific and Research Institute Voskhod", 38, pool).await?;
    
    replace_chars("Foundation ''Villa Joep", "Foundation ''Villa Joep''", "apostrophe added to Foundation ''Villa Joep", 38, pool).await?;

    let sql = r#"update rec.names set display_name = replace(replace(display_name, '''', ''), '’', '')
    where display_name like '%Workers ''and Peasants''%'"#;
    execute_sql(sql, "spurious apostrophes in Workers 'and Peasants'’ removed", 39, pool).await?;
    
    replace_chars("'École nationale supérieure des postes", "École nationale supérieure des postes",
         "apostrophe removed from 'École nationale supérieure des postes", 39, pool).await?;
    replace_chars("'Αμφισσας", "Αμφισσας", "apostrophe removed in 'Αμφισσας (Greek town)", 52, pool).await?;
    
    replace_chars("donn'ees", "données", "apostrophe replaced by accent, in donn'ees", 44, pool).await?;
    replace_chars("Unita'", "Unità", "apostrophe replaced by accent, in Unita'", 45, pool).await?;
    replace_chars("Regge' ", "Reggè", "apostrophe replaced by accent, in Regge' ", 46, pool).await?;
    
    replace_chars("Area 'A' Crab", "Area A Crab", "spurious apostrophes removed, in Area 'A' Crab'", 50, pool).await?;
    replace_chars("Art Fund_", "Art Fund", "spurious trailing underscore removed, in Art Fund_", 51, pool).await?;

    let sql = r#"update rec.names set display_name = replace(display_name, 'universite', 'université')
    where  display_name ~* 'universite ' or display_name ~* 'universite$'"#;
    execute_sql(sql, "mis-spelled universite repaired to université", 54, pool).await?;
    
    replace_chars("Univeristy", "University", "mis-spelled Univeristy repaired to University", 56, pool).await?;
    replace_chars("Univesity", "University", "mis-spelled Univesity repaired to University", 56, pool).await?;
    
    let sql = r#"update rec.names set display_name = replace(display_name, 'universit', 'university')
    where  display_name ~* 'universit ' or display_name ~* 'universit$'"#;
    execute_sql(sql, "mis-spelled universit repaired to university", 56, pool).await?;
    
    info!(""); 
    Ok(())
}


pub async fn standardise_double_quotes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // First put all double quotes and equivalents as straight double quotes
    // and all single quotes as apostrophes
    // (necessary to correct pre-existing errors and inconsistencies)
    
    info!("Standardising Double Quotes");
    replace_chars("“", "\"", "left double quotes replaced by straight quotes", 101, pool).await?;
    replace_chars("”", "\"", "right double quotes replaced by straight quotes", 102, pool).await?;
    replace_chars("«", "\"", "left guillemets replaced by straight quotes", 103, pool).await?;
    replace_chars("»", "\"", "right guillemets replaced by straight quotes", 104, pool).await?;
    
    replace_chars(",,", "„", "double commas replaced by low right quotes", 105, pool).await?;  // necessary precursor for a few records
    replace_chars("„", "\"", "low right quotes replaced by straight quotes", 106, pool).await?;
    
    replace_chars("''", "\"", "pairs of apostrophes replaced by straight quotes", 107, pool).await?;  // needed for a few records
    replace_chars("\"\"", "\"", "pairs of double quotes made into a single double quote", 108, pool).await?;  // AS few records with doubled double quotes
        
    info!("{} names with double quotes, to begin with", double_quotes_num(pool).await?);

    // First deal with hebrew names. These have double quotes standing in for the 
    // the gershayim 〈״〉, which is a Hebrew symbol indicating that a sequence of characters is an 
    // acronym, placed before the last character of the wod. As an intiial step first
    // ensure that all hebrew names are recognised as hebrew, then replace the quotes 
    // with the unicode gershayim symbol.

    let sql = r#"update rec.names set der_lang = 'he' 
    where display_name ~ '[\u0590-\u05FF]'"#;
    execute_sql(sql, "hebrew language label (re-)applied", 111, pool).await?;
    
    // change a double quote to gershayim (u05F4)
    // if it is the only double quote in the name

    let sql = r#"update rec.names set display_name = replace(display_name, '"', U&'\05F4')
    where der_lang = 'he'
    and length(display_name) - length(replace(display_name, '"', '')) = 1"#;
    execute_sql(sql, "double quotes replaced by gershayim symbol in hebrew names",112, pool).await?;

    // Can now proceed with dealing with the remaining double quotes.
    // Consider those few records (2) with 5 "
    // Drop the spurious 5th " so the records have 4 "

    let sql = r#"update rec.names set display_name = trim(regexp_replace(display_name, '"', '', 1, 5))
    where length(display_name) - length(replace(display_name, '"', '')) = 5"#;
    execute_sql(sql, "final double quotes removed in records with 5 double quotes", 115, pool).await?;
    
    // consider those records (30+) with 3 "
    // Which one to drop will depend on specific record - select by id

    let sql = r#"update rec.names set display_name = trim(regexp_replace(display_name, '"', '', 1, 1))
    where length(display_name) - length(replace(display_name, '"', '')) = 3
    and id in('019j1v294', '01hprsv49', '01mp7gg57', '01vd5cb71', '020whct63', '028mtfb17', '02b47v767', '03dx8n755', '03q57f308', '03qc6zh37' , '03wn3aq07', '049j4jr36', '04a7dp661', '057tmwv53', '05kzawq90', '05pkv9t98', '05q23ne91', '05svms055')"#;
    execute_sql(sql, "inital double quotes removed in records with 3 double quotes", 116, pool).await?;
    
    let sql = r#"update rec.names set display_name = trim(regexp_replace(display_name, '"', '', 1, 3))
    where length(display_name) - length(replace(display_name, '"', '')) = 3
    and id in ('00aa7ab77', '00kysjz64', '00qbdg904', '00wsvb073', '013fj3d42', '033z59547', '03b0cj417', '03xdgrg08', '05pc7fv53')"#;
    execute_sql(sql, "final double quotes removed in records with 3 double quotes", 117, pool).await?;
        
    // Then can consider names with just a single doble quote
    // In many cases add an additional quote to the end, but not in all
    
    let sql = r#"update rec.names set display_name = '"'||display_name
    where id in ('00a9b0g29', '00vrtwn56', '01g7a7y43', '03mgprp21', '052q58629', '05bpnjz66')
    and length(display_name) - length(replace(display_name, '"', '')) = 1"#;
    execute_sql(sql, "additional double quote added at beginning to form a pair", 120, pool).await?;
    
    let sql = r#"update rec.names set display_name = replace(display_name, '"', '')
    where id in ('04cnfv189')
    and length(display_name) - length(replace(display_name, '"', '')) = 1"#;
    execute_sql(sql, "spurious unpaired double quote removed", 121, pool).await?;

    let sql = r#"update rec.names set display_name = display_name||'"'
    where length(display_name) - length(replace(display_name, '"', '')) = 1"#;
    execute_sql(sql, "additional double quote added at end to form a pair", 122, pool).await?;
    
    // Finally change all the paired double quotes to 'proper' 66 -- 99 quotes
    
    let sql = r#"update rec.names set display_name = regexp_replace(display_name, '"(.*)"(.*)"(.*)"', '“\1”\2“\3”') 
        where length(display_name) - length(replace(display_name, '"', '')) = 4"#;
    execute_sql(sql, "2 pairs of double quotes changed to smart quotes", 123, pool).await?;
    
    let sql = r#"update rec.names set display_name = regexp_replace(display_name, '"(.*)"', '“\1”') 
    where length(display_name) - length(replace(display_name, '"', '')) = 2"#;
    execute_sql(sql, "paired double quotes changed to smart quotes", 124, pool).await?;
    
    // Ensure quotes are 'tight' to the words
    let sql = r#"update rec.names set display_name = trim(replace(display_name, '“ ', ' “')) 
    where display_name like '%“ %'"#;
    execute_sql(sql, "left double quotes followed by a space brought tight to word", 125, pool).await?;
     
    let sql = r#"update rec.names set display_name = trim(replace(display_name, ' ”', '” '))
    where display_name like '% ”%'"#;
    execute_sql(sql, "right double quotes preceded by a space brought tight to word", 126, pool).await?;
    
    // Put left and right double quote choices in the config file...
    // US pattern is the default but others can be used...
    // After paired single quotes have been done
    // do a final replace with the user's selected quote marks , if necessary
    
    info!("");
    
    Ok(())
}

pub async fn standardise_single_quotes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    info!("Standardising Single Quotes");
    replace_chars("‘", "''", "left single quote replaced by apostrophes", 201, pool).await?;
    replace_chars("’", "''", "right single quote replaced by apostrophes", 202, pool).await?;
        
    info!("{} names with apostrophes, to begin with", apos_num(pool).await?);

    /////////////////////////////////////////////////////////
    // Deal with some non European apostrophes
    ////////////////////////////////////////////////////////// 

    // Hawaiian -- left quote used to denote a glottal stop
    
    replace_chars("awai'i", "awai‘i", "apostrophe replaced by left quote in Hawai'i", 203, pool).await?;

    // Uzbek language names - left quote added to some vowels - chiefly after o

    execute_regex_replace(r"'O''', 'O‘', 'g'", "display_name ~ 'O'''  and lang = 'uz'", 
        "Uzbek capital o and apostrophe replaced by O left quote", 204, pool).await?;
    
    execute_regex_replace(r"'o''', 'o‘', 'g'", "display_name ~ 'o'''  and lang = 'uz'", 
        "Uzbek lower case o and apostrophe replaced by o left quote", 205, pool).await?;
    
    // Ukranian and Belarussian

    replace_chars("'я", "^я", "Orthographic apostrophe in cyrillic 'я replaced by caret", 210, pool).await?;
    replace_chars("'є", "^є", "Orthographic apostrophe in cyrillic 'є replaced by caret", 211, pool).await?;
    replace_chars("'ю", "^ю", "Orthographic apostrophe in cyrillic 'ю replaced by caret", 212, pool).await?;
    replace_chars("'ї", "^ї", "Orthographic apostrophe in cyrillic 'ї replaced by caret", 213, pool).await?;

    // Hebrew geresh symbol

    let sql = r#"update rec.names set display_name = replace(display_name, '''', U&'\05F3')
    where lang = 'he'
    and length(display_name) - length(replace(display_name, '''', '')) = 1"#;
    execute_sql(sql, "isolated apostrophe replaced by geresh symbol in hebrew names", 220, pool).await?;
    
    /////////////////////////////////////////////////////////
    // Deal with 's and s'
    //////////////////////////////////////////////////////////
    
    // Need to deal with some oddities first, including some capital Ss

    replace_chars("FU'S LAB", "Fu’s Lab", "name with ‘FU’s repaired", 222, pool).await?;
    replace_chars("Y'S Therap", "Y’s Therap", "name with ‘Y'S’ repaired", 223, pool).await?;

    replace_chars("IT'S TIME TEXAS", "It’s Time Texas", "name with oddly cased ‘IT'S TIME TEXAS’ repaired", 224, pool).await?;
    replace_chars("KELLEY'S LOGISTICS SUPPORT SYSTEMS", "Kelley’s Logisitics Support Systems", "name with oddly cased ‘Kelley's Logisitics Support Systems’ repaired", 224, pool).await?;
    replace_chars("VADASKERT FOUNDATION FOR CHILDREN'S MENTAL HEALTH", "Vadaskert Foundation for Children’s Mental Health", "name with oddly cased ‘Vadaskert Foundation for Children'S Mental Health’ repaired", 224, pool).await?;
    replace_chars("EUR'ORBEM", "Eur’Orbem", "name EUR'ORBEM repaired", 224, pool).await?;
    replace_chars("ST. MARY'S CATHOLIC MISSION HOSPITAL", "St. Mary’s Catholic Mission Hospital", "name with oddly cased ‘St. Mary's Catholic Mission Hospital’ repaired", 224, pool).await?;
 
    replace_chars("S'Klallam", "S’Klallam", "apostrophe in S'Klallam replaced", 230, pool).await?;
    replace_chars("Genes'ink", "Genes’ink", "apostrophe in Genes'ink replaced ", 231, pool).await?;
    replace_chars("s'i", "s^i", "other apostrophe in s'i retained", 251, pool).await?;
    replace_chars("A'Sharqiyah", "A^Sharqiyah", "apostrophe in A'Sharqiyah retained", 232, pool).await?;
    
    replace_chars("M'Sila", "M’Sila", "apostrophe in M'Sila replaced", 233, pool).await?;
    replace_chars("M'sila", "M’sila", "apostrophe in M'sila replaced", 234, pool).await?;
    replace_chars("P.D.V.V.P.F'S", "P.D.V.V.P.F’s", "apostrophe in P.D.V.V.P.F'S replaced", 235, pool).await?;
    replace_chars("3G'S", "3G’S", "apostrophe in 3G'S replaced", 236, pool).await?;
    replace_chars("AGTI'S", "AGTI’s", "apostrophe in AGTI'S replaced", 237, pool).await?;
    replace_chars("T'Sou", "T’Sou", "apostrophe in T'Sou replaced", 238, pool).await?;
        
    execute_regex_replace(r"'([a-zA-Z0-9])''s([ ,-])', '\1’s\2' , 'g'", "display_name ~ '[a-zA-Z0-9]''s[ ,-]'", 
        "possessive apostrophe replaced, 's to ’s", 240, pool).await?;
    
    execute_regex_replace(r"'([a-zA-Z0-9])''s$', '\1’s'", "display_name ~ '[a-zA-Z0-9]''s$'", 
        "possessive apostrophe at end replaced, 's to ’s", 241, pool).await?;

    execute_regex_replace(r"'s''', 's’', 'g'", "display_name ~ 's'' ' or display_name ~ 's''$'", 
        "possessive apostrophe replaced for plural nouns, s' to s’", 242, pool).await?;

    // N.B. Last change masks some paired apostrophes, that should become double quotes
    // Need to go back later to repair this
    
    execute_regex_replace(r"'''s ', '’s '", "display_name ~ '^''s '", 
        "apostrophe replaced, in initial 's (Dutch abbreviation)", 245, pool).await?;
    
    execute_regex_replace(r"' ''t ', ' ’t '", "display_name ~ ' ''t '", 
        "apostrophe replaced, in free floating 't (Dutch abbreviation)", 246, pool).await?;
        
    /*
   
    -- finish off the s
    update rec.names set value = replace(value, '''s', '^s')
    where value ~ '''s'
    update rec.names set value =  replace(value, 's''', 's^') 
    where value ~ 's'''
    */

    /////////////////////////////////////////////////////////
    // Deal with d' and D'
    //////////////////////////////////////////////////////////

    execute_regex_replace(r"'([ eou-])d''([AÁEÉHIÎOUXY])', '\1d’\2', 'gi'", "display_name ~* '[ eou-]d''[AÁEÉHIÎOUXY]'", 
        "apostrophe replaced, in d' followed by a vowel or a few consonants", 254, pool).await?;

    execute_regex_replace(r"'^D''([AEÉHIÎOUXY])', 'D’\1', 'i'", "display_name ~* '^D''[AEÉHIÎOUXY]'", 
         "apostrophe replaced, in initial D", 255, pool).await?;
    
    /////////////////////////////////////////////////////////
    // Deal with l' and L'
    //////////////////////////////////////////////////////////

    execute_regex_replace(r"'([ l])l'' ' , '\1l’'", "display_name ~ '[ l]l'' '", 
        "apostrophe replaced, in l'-space following space or l", 260, pool).await?;

    execute_regex_replace(r"'^L'' ' , 'L’'", "display_name ~ '^L'' '", 
        "apostrophe replaced, in initial L' followed by space", 261, pool).await?;

    execute_regex_replace(r"'([ l-])l''([AÁEÉèHIÎOlœUXY])', '\1l’\2', 'gi'", "display_name ~* '[ l-]l''[AÁEÉèHIÎOœUXY]'", 
        "apostrophe replaced, in l' following space or l", 262, pool).await?;

    execute_regex_replace(r"'^l''([AÁEÉHIÎOUXY])', 'L’\1', 'gi'", "display_name ~* '^l''[AÁEÉHIÎOUXY]'", 
        "apostrophe replaced, in initial L'", 263, pool).await?;

    // Remainder preserved as apostrophes or equivalewnt
       
    replace_chars("l'", "l^", "Remainder l' retained as apostrophe", 266, pool).await?;

    /////////////////////////////////////////////////////////
    // Other apostrophes
    //////////////////////////////////////////////////////////

    replace_chars("POLYTECH'LAB", "Polytech’Lab", "name with oddly cased ‘POLYTECH'LAB’ repaired", 270, pool).await?;
    replace_chars("Polytech'Lab", "Polytech’Lab", "apostrophe in Polytech'Lab replaced", 271, pool).await?;
    replace_chars("OCCI'FOOD", "OCCI’FOOD", "apostrophe in OCCI'FOOD replaced", 272, pool).await?;
    
    replace_chars("ca' ", "ca’ ", "apostrophe replaced, in ca' ", 275, pool).await?;
    replace_chars("Ca' ", "Ca’ ", "apostrophe replaced, in Ca' ", 276, pool).await?;

    execute_regex_replace(r"'([aāáeěiíou])''([aāáeěiíou])', '\1^\2', 'g'", 
        "display_name ~* '[aāáeěiíou]''[aāáeěiíou]'", 
       "apostrophe retained when between vowels'", 277, pool).await?;
     
    replace_chars("O'", "O’", "apostrophe replaced, in non-Uzbek O'", 280, pool).await?;
    replace_chars("Sant'", "Sant’", "apostrophe replaced, in Sant'", 281, pool).await?;
    replace_chars("c'est", "c’est", "apostrophe replaced, in c'est", 282, pool).await?;
    replace_chars("I'm", "I’m", "apostrophe replaced, in I'm", 283, pool).await?;
    replace_chars("T'Sou", "T’Sou", "apostrophe replaced, in T'Sou", 284, pool).await?;
    replace_chars("Activ'Inside", "Activ’Inside", "apostrophe replaced in Activ'Inside", 285, pool).await?;
    replace_chars("Ex'pression", "Ex’pression", "apostrophe replaced, in Ex'pression", 286, pool).await?;
    replace_chars("t'l", "t’l", "apostrophe replaced, in t'l", 287, pool).await?;
    replace_chars("Qu'Appelle", "Qu’Appelle", "apostrophe replaced, in Qu'Appelle", 288, pool).await?;
    replace_chars("Maiz'Europ'", "Maiz’Europ’", "apostrophe replaced, in Maiz'Europ'", 289, pool).await?;
    replace_chars("Int'Air", "Int’Air", "apostrophe replaced, in Int'Air", 290, pool).await?;
    replace_chars("In'Tech", "In’Tech", "apostrophe replaced, in In'Tech", 291, pool).await?;
    replace_chars("ISCrim'", "ISCrim’", "apostrophe replaced, in ISCrim'", 292, pool).await?;
    replace_chars("Vizion'R", "Vizion’R", "apostrophe replaced, in Vizion'R", 293, pool).await?;
    replace_chars("DINÂMIA'CET", "DINÂMIA’CET", "apostrophe replaced, in DINÂMIA'CET", 294, pool).await?;
    replace_chars("En'Urga", "En’Urga", "apostrophe replaced, in En'Urga", 295, pool).await?;

    replace_chars("Institut P'", "Institut P^", "apostrophe retained, in Institut P'", 300, pool).await?;
    replace_chars("অসম ডনব'স্ক' বিশ্ববিদ্যালয়", "অসম ডনব^স্ক^ বিশ্ববিদ্যালয়", "apostrophes retained, in অসম ডনব'স্ক' বিশ্ববিদ্যালয়", 301, pool).await?;
    replace_chars("'Aisyiyah", "^Aisyiyah", "apostrophe retained, in 'Aisyiyah", 302, pool).await?;
    replace_chars("Sh'or", "Sh^or", "apostrophe retained, in Sh'or", 303, pool).await?;
    replace_chars("VERN'", "VERN^", "apostrophe retained, in VERN'", 304, pool).await?;
    replace_chars("U'budiyah", "U^budiyah", "apostrophe retained, in U'budiyah", 305, pool).await?;

    execute_regex_replace(r"'''([0-9])', '’\1', 'g'", "display_name ~ '''[0-9]'", 
        "apostrophe replaced when immediately before numerals (usually years)", 320, pool).await?;
       
    // An odd one that needs to be done first, then n', N' retained 
    
    replace_chars("En'owkin", "En’owkin", "apostrophe replaced, in En'owkin", 322,  pool).await?;

    execute_regex_replace(r"'t''([a-z])', 't^\1', 'g'", "display_name ~ 't''[a-z]'", 
         "apostrophe retained when after remaining t", 330, pool).await?;
    
    execute_regex_replace(r"'a''([a-zA-Z])', 'a^\1', 'g'", "display_name ~ 'a''[a-zA-Z]'", 
        "apostrophe retained when after 'a'", 331, pool).await?;
    
    execute_regex_replace(r"'([a-zA-Z])''a', '\1^a', 'g'", "display_name ~ '[a-zA-Z]''a'", 
        "apostrophe retained when before 'a'", 332, pool).await?;
    
    // At this stage possible to safely do those names with paired apostrophes 
    // turning them into 66 99 quotes
    
    execute_regex_replace(r"'''(.*)''', '“\1”'", "display_name ~ '''[a-zA-Z. -]*'''", 
        "paired single quotes changed to smart double quotes", 333, pool).await?;
    
    execute_regex_replace(r"'''(.*)’', '“\1”'", "display_name ~ '''[a-zA-Z. -]*’'",  
        "paired single / right quotes changed to smart double quotes", 334, pool).await?;
    
    // The n' at least needs to go after the paired single quotes above
    
    replace_chars("y'", "y’", "apostrophe replaced, in y' (cantral African names)", 340, pool).await?;
    replace_chars(" 'n ", " ’n ", "apostrophe replaced, in 'n ", 341, pool).await?;
    replace_chars("n' ", "n’ ", "apostrophe replaced, in n' ", 342, pool).await?;
    replace_chars("ta' ", "ta’ ", "apostrophe replaced, in ta' ", 343, pool).await?;
    replace_chars("SUP'", "SUP’", "apostrophe replaced, in SUP'", 350, pool).await?;
    replace_chars("Sup'", "Sup’", "apostrophe replaced, in Sup'", 351, pool).await?;
    replace_chars("de'Montmorency", "de’Montmorency", "apostrophe replaced, in de'Montmorency", 352, pool).await?;
    replace_chars("KE'KEN", "KE’KEN", "apostrophe replaced, in KE'KEN", 353, pool).await?;
    replace_chars("ENET'Com", "ENET’Com", "apostrophe replaced, in ENET'Com", 354, pool).await?;
    replace_chars("ISET'COM", "ISET’COM", "apostrophe replaced, in ISET'COM", 355, pool).await?;
    replace_chars("Ovar'coming", "Ovar’coming", "apostrophe replaced, in Ovar'coming", 360, pool).await?;
    replace_chars("N'Djamena", "N’Djamena", "apostrophe replaced, in N'Djamena", 360, pool).await?;
    replace_chars("N'gourma", "N’gourma", "apostrophe replaced, in N'gourma", 361, pool).await?;
    replace_chars("Lu'ma", "Lu’ma", "apostrophe replaced, in Lu'ma", 362, pool).await?;
    replace_chars("Klet'", "Klet’", "apostrophe replaced, in Klet'", 363, pool).await?;
    replace_chars("Oniversiten'Antananarivo", "Oniversiten’Antananarivo", "apostrophe replaced, in Oniversiten'Antananarivo", 365, pool).await?;

    // Do double spaces to single at end?
    // info!("{} double spaces replaced by single in names to match", replace_in_names("  ", " ", pool).await?);
    

    info!("{} names with apostrophes after processing", apos_num(pool).await?);
    info!("");

    // Most of the remaining apostrophes uses to indicate syllable boundaries 
    // in transliterated Chinese, Japanee, Arabic
    // Should be retained as apostrophes

    //replace_chars("^", "''", "", pool).await?;
    //info!("(^) resored back to (') in {n} records");
    
    Ok(())
}


async fn execute_sql(sql: &str, change: &str, rep_type: i32, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let change2 = if change.contains("'") {change.replace("'", "''")} else {change.to_string()};
    
    let sql = sql.replace(" where ", format!(r#",
    changed = true,
    change_type_id = case when change_type_id is null then '{rep_type}'
        else change_type_id||', {rep_type}'
    end,
    change_type = 
        case when change_type is null then '{change2}'
        else change_type||', '||'{change2}'
    end 
    where "#).as_str());
    let n = sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();
   
    if n > 0 {
        if n == 1 {
            info!("{change} ({})", "1 record");
        } 
        else {
            info!("{change} ({})", format!("{n} records").as_str());
        };
    }
    Ok(())
}


async fn execute_regex_replace(regex: &str, wh: &str, change: &str, rep_type: i32, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let change2 = if change.contains("'") {change.replace("'", "''")} else {change.to_string()};
    
    let sql = format!(r#"update rec.names 
    set display_name = regexp_replace(display_name, {regex}),
    changed = true,
    change_type_id = case when change_type_id is null then '{rep_type}'
        else change_type_id||', {rep_type}'
    end,
    change_type = 
        case when change_type is null then '{change2}'
        else change_type||', '||'{change2}'
    end 
    where {wh}"#);
    
    let n = sqlx::query(&sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();
   
    if n > 0 {
        if n == 1 {
            info!("{change} ({})", "1 record");
        } 
        else {
            info!("{change} ({})", format!("{n} records").as_str());
        };
    }
    Ok(())
}


async fn replace_chars(chars: &str, replacement: &str, description: &str, 
              rep_type: i32, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let chars2 = if chars.contains("'") {chars.replace("'", "''")} else {chars.to_string()};
    
    let ch_type = format!("({chars2}) replaced by ({replacement})");
    let sql  = format!(r#"update rec.names
            set display_name = replace(display_name, '{chars2}', '{replacement}'),
            changed = true,
            change_type_id = case when change_type_id is null then '{rep_type}'
                else change_type_id||', {rep_type}'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where display_name like '%{chars2}%' "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        if n == 1 {
            info!("{description} ({})", "1 record");
        } 
        else {
            info!("{description} ({})", format!("{n} records").as_str());
        };
    }
    Ok(())
}


async fn replace_unicode_char(unicode_char: &str, rep_type: i32, char_description: &str, 
    replacement: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {
    let ch_type = if replacement == "" {
        format!("(\\u{unicode_char}, {char_description}) removed")
    }
    else {
        if replacement == "-" {
            format!("(\\u{unicode_char}, {char_description}) replaced by ascii hyphen")
        }
        else {
            format!("(\\u{unicode_char}, {char_description}) replaced by ({replacement})")
        }
    };
    let sql  = format!(r#"update rec.names
            set display_name = replace(display_name, U&'\{unicode_char}', '{replacement}'),
            changed = true,
            change_type_id = case when change_type_id is null then '{rep_type}'
                else change_type_id||', '||'{rep_type}'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where display_name like U&'%\{unicode_char}%'; "#);

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

async fn apos_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from rec.names 
    where display_name like '%''%'"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(r)
}

async fn double_quotes_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from rec.names 
    where display_name like '%"%'"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(r)
}


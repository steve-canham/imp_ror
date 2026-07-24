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

    // First put all double quotes and equivalents as straight double quotes
    // and all single quotes as apostrophes
    // (necessary to correct pre-existing errors and inconsistencies)

    replace_chars("“", "\"", "left double quotes replaced by straight quotes in ~", pool).await?;
    replace_chars("”", "\"", "right double quotes replaced by straight quotes in ~", pool).await?;
    replace_chars("«", "\"", "left guillemets replaced by straight quotes in ~", pool).await?;
    replace_chars("»", "\"", "right guillemets replaced by straight quotes in ~", pool).await?;
    
    replace_chars(",,", "„", "double commas replaced by low right quotes in rr", pool).await?;  // necessary precursor for a few records
    replace_chars("„", "\"", "low right quotes replaced by straight quotes in ~", pool).await?;
    
    replace_chars("''''", "\"", "pairs of apostrophes replaced by straight quotes in ~", pool).await?;  // needed for a few records
    replace_chars("\"\"", "\"", "pairs of double quotes made into a single double quote in ~", pool).await?;  // AS few records with doubled double quotes
    
    replace_chars("‘", "''", "left single quote replaced by apostrophes in ~", pool).await?;
    replace_chars("’", "''", "right single quote replaced by apostrophes in ~", pool).await?;

    
    // deal with some very specific oddities (clearing them out of the way)
      
    replace_chars("[править | править вики-текст]", "", "'[%править | править вики-текст]', translated as 'edit | edit wiki-text' removed in ~", pool).await?;
    replace_chars("[ Citation needed | edit wiki text ]", "", "'[ Citation needed | edit wiki text ]', removed in ~", pool).await?;
    replace_chars(" (Rybářství Litomyšl)", "", "Spurious repeated text 'Rybářství Litomyšl' removed in ~", pool).await?;
    replace_chars("?>", "->", "Incorrect arrow formula replaced in ~", pool).await?;

    let sql = r#"update src.names set value = replace(value, '[', '') where value like '%['"#;
    execute_sql(sql, "final left bracket removed from ~", pool).await?;
    let sql = r#"update src.names set value = replace(value, ';', '') where value like '%;'"#;
    execute_sql(sql, "final semi-colon removed from ~", pool).await?;
    let sql = r#"update src.names set value = translate(value, '[]', '')
    where orig_value like '%]' and orig_value like '[%'"#;
    execute_sql(sql, "Paired outer brackets removed from ~", pool).await?;
    //  N.B. No current equivalent for paranthese or curly btrackets
    
    let sql = r#"update src.names set value = replace(value, '[', '') 
    where value like '%[%' and value not like '%]%'"#;
    execute_sql(sql, "unpaired left bracket removed from ~", pool).await?;
    let sql = r#"update src.names set value = replace(value, ']', '') 
    where value like '%]%' and value not like '%[%'"#;
    execute_sql(sql, "unpaired right bracket removed from ~", pool).await?;

    replace_chars("Polemikí Aeroporía, literally \"Military Aviation\"", "Polemikí Aeroporía", "'literally' folowed by translation removed from ~", pool).await?;
    replace_chars("literally Public Komatsu University", "Public Komatsu University", "'literally' removed from ~", pool).await?;
    replace_chars("... ", "", "ellipsis removed from ~", pool).await?;
    
    //////////////////////////////////////////////////////////
    // Deal with Double quotes
    //////////////////////////////////////////////////////////
    
    info!("{} names with double quotes, to begin with", double_quotes_num(pool).await?);

    // First deal with hebrew names. These have double quotes standing in for the 
    // the gershayim 〈״〉, which is a Hebrew symbol indicating that a sequence of characters is an 
    // acronym, placed before the last character of the wod. As an intiial step first
    // ensure that all hebrew names are recognised as hebrew, then replace the quotes 
    // with the unicode gershayim symbol.

    let sql = r#"update src.names set lang = 'he' 
    where  value ~ '[\u0590-\u05FF]' and lang <> 'he'"#;
    execute_sql(sql, "hebrew language label (re-)applied to ~", pool).await?;
    
    // change a double quote to gershayim (u05F4)
    // if it is the only double quote in the name

    let sql = r#"update src.names set value = replace(value, '"', U&'\05F4')
    where lang = 'he'
    and length(value) - length(replace(value, '"', '')) = 1"#;
    execute_sql(sql, "double quotes replaced by gershayim symbol in hebrew names, in ~", pool).await?;

    // Can now proceed with dealing with the remaining double quotes.
    // Consider those few records (2) with 5 "
    // Drop the spurious 5th " so the records have 4 "

    let sql = r#"update src.names set value = trim(regexp_replace(value, '"', '', 1, 5))
    where length(value) - length(replace(value, '"', '')) = 5"#;
    execute_sql(sql, "final double quotes removed in records with 5 double quotes, in ~", pool).await?;
    
    // consider those records (30+) with 3 "
    // Which one to drop will depend on specific record - select by id

    let sql = r#"update src.names set value = trim(regexp_replace(value, '"', '', 1, 1))
    where length(value) - length(replace(value, '"', '')) = 3
    and id in('019j1v294', '01hprsv49', '01mp7gg57', '01vd5cb71', '020whct63', '028mtfb17', '02b47v767', '03dx8n755', '03q57f308', '03qc6zh37' , '03wn3aq07', '049j4jr36', '04a7dp661', '057tmwv53', '05kzawq90', '05pkv9t98', '05q23ne91', '05svms055')"#;
    execute_sql(sql, "inital double quotes removed in records with 3 double quotes, in ~", pool).await?;
    
    let sql = r#"update src.names set value = trim(regexp_replace(value, '"', '', 1, 3))
    where length(value) - length(replace(value, '"', '')) = 3
    and id in ('00aa7ab77', '00kysjz64', '00qbdg904', '00wsvb073', '013fj3d42', '033z59547', '03b0cj417', '03xdgrg08', '05pc7fv53')"#;
    execute_sql(sql, "final double quotes removed in records with 3 double quotes, in ~", pool).await?;
        

    // Then can consider names with just a single doble quote
    // In many cases add an additional quote to the end, but not in all
    
    let sql = r#"update src.names set value = '"'||value
    where id in ('00a9b0g29', '00vrtwn56', '01g7a7y43', '03mgprp21', '052q58629', '05bpnjz66')
    and length(value) - length(replace(value, '"', '')) = 1"#;
    execute_sql(sql, "additional double quote added at beginning to form a pair in ~", pool).await?;
    
    let sql = r#"update src.names set value = replace(value, '"', '')
    where id in ('04cnfv189')
    and length(value) - length(replace(value, '"', '')) = 1"#;
    execute_sql(sql, "spurious unpaired double quote removed in ~", pool).await?;

    let sql = r#"update src.names set value = value||'"'
    where length(value) - length(replace(value, '"', '')) = 1"#;
    execute_sql(sql, "additional double quote added at end to form a pair in ~", pool).await?;
    
    // Finally change all the paired double quotes to 'proper' 66 -- 99 quotes
    
    let sql = r#"update src.names set value = regexp_replace(value, '"(.*)"(.*)"(.*)"', '“\1”\2“\3”') 
        where length(value) - length(replace(value, '"', '')) = 4"#;
    execute_sql(sql, "paired double quotes changed to smart quotes in ~ with 4 double quotes", pool).await?;
    
    let sql = r#"update src.names set value = regexp_replace(value, '"(.*)"', '“\1”') 
    where length(value) - length(replace(value, '"', '')) = 2"#;
    execute_sql(sql, "paired double quotes changed to smart quotes in ~ with 2 double quotes", pool).await?;
    
    // Ensure quotes are 'tight' to the words
    let sql = r#"update src.names set value = trim(replace(value, '“ ', ' “')) 
    where value like '%“ %'"#;
    execute_sql(sql, "left double quotes followed by a space brought tight to word in ~", pool).await?;
     
    let sql = r#"update src.names set value = trim(replace(value, ' ”', '” '))
    where value like '% ”%'"#;
    execute_sql(sql, "right double quotes preceded by a space brought tight to word in ~", pool).await?;
    
    // Put left and right double quote choices in the config file...
    // US pattern is the default but others can be used...
    // After paired single quotes have been done
    // do a final replace with the user's selected quote marks , if necessary

    /////////////////////////////////////////////////////////
    // Deal with Single quotes
    //////////////////////////////////////////////////////////
        
    info!("{} names with apostrophes, to begin with", apos_num(pool).await?);

    /////////////////////////////////////////////////////////
    // Deal with some non European apostrophes
    ////////////////////////////////////////////////////////// 

    // Hawaiian -- left quote used to denote a glottal stop
    
    replace_chars("awai'i", "awai‘i", "apostrophe replaced by left quote in Hawai'i in ~", pool).await?;

    // Uzbek language names - left quote added to some vowels - chiefly after o

    let sql  = r#"update src.names set value = regexp_replace(value, 'O''', 'O‘', 'g')
    where value ~ 'O'''  and lang = 'uz'"#;
    execute_sql(sql, "Uzbek capital o and apostrophe replaced by O left quote in ~", pool).await?;
    
    let sql  = r#"update src.names set value = regexp_replace(value, 'o''', 'o‘', 'g')
    where value ~ 'o'''  and lang = 'uz'"#;
    execute_sql(sql, "Uzbek lower case o and apostrophe replaced by o left quote in ~", pool).await?;

    // Ukranian and Belarussian

    replace_chars("'я", "^я", "Orthographic apostrophe in cyrillic 'я replaced by caret in ~", pool).await?;
    replace_chars("'є", "^є", "Orthographic apostrophe in cyrillic 'є replaced by caret in ~", pool).await?;
    replace_chars("'ю", "^ю", "Orthographic apostrophe in cyrillic 'ю replaced by caret in ~", pool).await?;
    replace_chars("'ї", "^ї", "Orthographic apostrophe in cyrillic 'ї replaced by caret in ~", pool).await?;

    // Hebrew

    let sql = r#"update src.names set value = replace(value, '''', U&'\05F3')
    where lang = 'he'
    and length(value) - length(replace(value, '''', '')) = 1"#;
    execute_sql(sql, "isolated apostrophe replaced by geresh symbol in hebrew names, in ~", pool).await?;
    

    /////////////////////////////////////////////////////////
    // Deal with 's and s'
    //////////////////////////////////////////////////////////
    
    // Need to deal with some oddities first
    
    let sql = r#"update src.names set value = replace(value, 'eople ''s', 'eople’s')
    where value like '%eople ''s%'"#;
    execute_sql(sql, "name with odd 'people 's' repaired, in ~", pool).await?;

    replace_chars("Children's' ", "Children’s ", "name with odd 'Children's' ' repaired, in ~", pool).await?;
    replace_chars("Seiryo WOMEN'S ", "Seiryo Women’s ", "name with odd 'WOMEN'S' repaired, in ~", pool).await?;
    replace_chars("Women'S ", "Women’s ", "name with odd 'women'S' repaired, in ~", pool).await?;
    
    replace_chars("THE Japan WRITERS' Association", "The Japan Writers’ Association", "name with odd 'THE and WRITERS' repaired, in rr", pool).await?;
    replace_chars("Japan WRITERS' Association", "Japan Writers’ Association", "name with odd 'WRITERS' repaired, in ~", pool).await?;
    replace_chars("SEAMEN'S Employment", "Seamen’s Employment", "name with odd 'SEAMEN''S repaired, in ~", pool).await?;
    replace_chars("Glass MANUFACTURERS' ", "Glass Manufacturers’ ", "name with odd 'MANUFACTURERS' repaired, in r", pool).await?;

    replace_chars("FU'S LAB", "Fu’s Lab", "name with odd 'FU'' repaired, in ~", pool).await?;
    replace_chars("Y'S Therap", "Y’s Therap", "name with odd 'Y'S' repaired, in ~", pool).await?;

    replace_chars("Breeders'Association", "Breeders’ Association", "name with odd 'Breeders'Association' repaired, in ~", pool).await?;
    replace_chars("IT'S TIME TEXAS", "It’s Time Texas", "name with odd 'IT'S TIME TEXAS' repaired, in ~", pool).await?;

    replace_chars("KELLEY'S LOGISTICS SUPPORT SYSTEMSn", "Kelley’s Logisitics Support Systems", "name with odd Kelley's Logisitics Support Systems repaired, in ~", pool).await?;
    
    replace_chars("VADASKERT FOUNDATION FOR CHILDREN'S MENTAL HEALTH", "Vadaskert Foundation for Children’s Mental Health", "name with odd Vadaskert Foundation for Children'S Mental Health repaired, in ~", pool).await?;
    
    replace_chars("ST. MARY'S CATHOLIC MISSION HOSPITAL", "St. Mary’s Catholic Mission Hospital", "name with odd St. Mary's Catholic Mission Hospital repaired, in ~", pool).await?;

    replace_chars("S'Klallam", "S’Klallam", "S'Klallam repaired, in ~", pool).await?;
    replace_chars("Genes'ink", "Genes’ink", "Genes'ink repaired, in ~", pool).await?;

    replace_chars("A'Sharqiyah", "A^Sharqiyah", "A'Sharqiyah repaired, in ~", pool).await?;
    replace_chars("M'Sila", "M’Sila", "M'Sila repaired, in ~", pool).await?;
    replace_chars("M'sila", "M’sila", "M'sila repaired, in ~", pool).await?;
    replace_chars("P.D.V.V.P.F'S", "P.D.V.V.P.F’s", "P.D.V.V.P.F'S repaired, in ~", pool).await?;
    replace_chars("3G'S", "3G’S", "3G'S repaired, in ~", pool).await?;
    replace_chars("AGTI'S", "AGTI’s", "AGTI'S repaired], in ~", pool).await?;
    replace_chars("T'Sou", "T’Sou", "T'Sou repaired, in ~", pool).await?;
        
    let sql = r#"update src.names set value = regexp_replace(value, '([a-zA-Z0-9])''s([ ,-])', '\1’s\2' , 'g') 
    where value ~ '[a-zA-Z0-9]''s[ ,-]'"#;
    execute_sql(sql, "apostrophe replaced, 's to ’s , in ~", pool).await?;
    
    let sql = r#"update src.names 
    set value = regexp_replace(value, '([a-zA-Z0-9])''s$', '\1’s') 
    where value ~ '[a-zA-Z0-9]''s$'"#;
    execute_sql(sql, "apostrophe replaced, final 's to ’s , in ~", pool).await?;

 
    let sql = r#"update src.names set value = regexp_replace(value, 's''', 's’', 'g')
    where value ~ 's'' ' or value ~ 's''$'"#;
    execute_sql(sql, "apostrophe replaced, s' to s’ , in ~", pool).await?;

    // N.B. Last change masks some paired apostrophes, that should become double quotes
    // Need to go back later to repair this

    let sql = r#"update src.names set value = regexp_replace(value, '''s ', '’s ' ) 
    where value ~ '^''s '"#;
    execute_sql(sql, "apostrophe replaced, in initial 's (Dutch abbreviation), in ~", pool).await?;

    let sql = r#"update src.names set value = regexp_replace(value, ' ''t ', ' ’t ' ) 
    where value ~ ' ''t '"#;
    execute_sql(sql, "apostrophe replaced, in free floating 't (Dutch abbreviation), in ~", pool).await?;

    /*
   
    -- finish off the s
    update src.names set value = replace(value, '''s', '^s')
    where value ~ '''s'
    update src.names set value =  replace(value, 's''', 's^') 
    where value ~ 's'''
    */

    /////////////////////////////////////////////////////////
    // Deal with d' and D'
    //////////////////////////////////////////////////////////
    
    // A few odd ones first    
    let sql = r#"update src.names set value = replace(value, ' d'' ', ' d’')
    where value like '% d'' %'"#; 
    execute_sql(sql, "apostrophe replaced, in d' followed by a space, in ~", pool).await?;

    let sql = r#"update src.names set value = regexp_replace(value, '([ eou-])d''([AÁEÉHIÎOUXY])', '\1d’\2', 'gi')
    where value ~* '([ eou-])d''([AÁEÉHIÎOUXY])'"#;
    execute_sql(sql, "apostrophe replaced, in d' followed by a vowel and a few consonants, in ~", pool).await?;

    let sql = r#"update src.names set value = regexp_replace(value, '^D''([AEÉHIÎOUXY])', 'D’\1', 'i')
    where value ~* '^D''([AEÉHIÎOUXY])'"#;
    execute_sql(sql, "apostrophe replaced, in initial D', in ~", pool).await?;

    /////////////////////////////////////////////////////////
    // Deal with l' and L'
    //////////////////////////////////////////////////////////

    // Odd three need repariring first
    replace_chars("I'information", "l’information", "I'information repaired], in ~", pool).await?;
    replace_chars("I'industrie", "l’industrie", "I'industrie repaired], in ~", pool).await?;
    replace_chars("I'INSU", "l’INSU", "I'INSU repaired], in ~", pool).await?;
    
    let sql = r#"update src.names set value = regexp_replace(value, '([ l])l'' ' , '\1l’')
    where value ~ '[ l]l'' '"#;  
    execute_sql(sql, "apostrophe replaced, in l'-space following space or l, in ~", pool).await?;

    let sql = r#"update src.names set value = regexp_replace(value, '^L'' ' , 'L’')
    where value ~ '^L'' '"#;
    execute_sql(sql, "apostrophe replaced, in initial L' followed by space, in ~", pool).await?;
    
    let sql = r#"update src.names set value = regexp_replace(value, '([ l-])l''([AÁEÉèHIÎOlœUXY])', '\1l’\2', 'gi')
    where value ~* '([ l-])l''([AÁEÉèHIÎOœUXY])'"#;
    execute_sql(sql, "apostrophe replaced, in l' following space or l, in ~", pool).await?;
    
    let sql = r#"update src.names set value = regexp_replace(value, '^l''([AÁEÉHIÎOUXY])', 'L’\1', 'gi')
    where value ~* '^l''([AÁEÉHIÎOUXY])'"#;
    execute_sql(sql, "apostrophe replaced, in initial L', in ~", pool).await?;
    
    // Remainder preserved as apostrophes or equivalewnt
       
    replace_chars("l'", "l^", "Remainder l' retained as apostrophe, in ~", pool).await?;

    /////////////////////////////////////////////////////////
    // Other apostrophes
    //////////////////////////////////////////////////////////

    replace_chars("ca' ", "ca’ ", "apostrophe replaced, in ca', in ~", pool).await?;
    replace_chars("Ca' ", "Ca’ ", "apostrophe replaced, in Ca', in ~", pool).await?;
    
    let sql = r#"update src.names
    set value = regexp_replace(value, '([aeiou])''([aeiou])', '\1^\2', 'g')
    where value ~* '[aeiou]''[aeiou]'"#;
    execute_sql(sql, "apostrophe retained when between vowels', in ~", pool).await?;

    replace_chars("O'", "O’", "apostrophe replaced, in O', in ~", pool).await?;
    replace_chars("Sant'", "Sant’", "apostrophe replaced, in Sant', in ~", pool).await?;
    replace_chars("c'est", "c’est", "apostrophe replaced, in c'est, in ~", pool).await?;
    replace_chars("I'm", "I’m", "apostrophe replaced, in I'm, in ~", pool).await?;
    replace_chars("donn'ees", "données", "apostrophe replaced, in donn'ees, in ~", pool).await?;
    replace_chars("T'Sou", "T’Sou", "apostrophe replaced, in T'Sou, in ~", pool).await?;
    replace_chars("Activ'Inside", "Activ’Inside", "apostrophe replaced, in Activ'Inside, in ~", pool).await?;
    replace_chars("Unita'", "Unità", "apostrophe replaced, in Unita', in ~", pool).await?;
    replace_chars("Ex'pression'", "Ex’pression", "apostrophe replaced, in Ex'pression', in ~", pool).await?;
    replace_chars("t'l", "t’l", "apostrophe replaced, in t'l', in ~", pool).await?;
    replace_chars("Qu'Appelle", "Qu’Appelle", "apostrophe replaced, in Qu'Appelle, in ~", pool).await?;
    replace_chars("Maiz'Europ'", "Maiz’Europ’", "apostrophe replaced, in Maiz'Europ', in ~", pool).await?;
    replace_chars("Institut P'", "Institut P^", "apostrophe retained, in Institut P', in ~", pool).await?;
    replace_chars("অসম ডনব'স্ক' বিশ্ববিদ্যালয়", "অসম ডনব^স্ক^ বিশ্ববিদ্যালয়", "apostrophes retained, in অসম ডনব'স্ক' বিশ্ববিদ্যালয়', in ~", pool).await?;
    replace_chars("VERN'", "VERN^", "apostrophe replaced, in VERN', in ~", pool).await?;
    replace_chars("Area 'A' Crab", "Area A Crab", "apostrophes removed, in Area 'A' Crab', in ~", pool).await?;
    
    let sql = r#"update src.names
    set value = regexp_replace(value, '''([0-9])', '’\1', 'g')
    where value ~ '''[0-9]'"#;
    execute_sql(sql, "apostrophe replaced when immediately before numerals (usually years), in ~", pool).await?;
       
    // An odd one that needs to be done first, then n', N' retained 
    
    replace_chars("En'owkin", "En’owkin", "apostrophe replaced, in En'owkin, in ~", pool).await?;
    let sql = r#"update src.names
    set value = regexp_replace(value, 'n''([a-z])', 'n^\1', 'g')
    where value ~ 'n''[a-z]'"#;
    execute_sql(sql, "apostrophe retained when after other n, in ~", pool).await?;
    
    let sql = r#"update src.names
    set value = regexp_replace(value, 'N''([a-z])', 'N^\1', 'g')
    where value ~ 'N''[a-z]'"#;
    execute_sql(sql, "apostrophe retained when after N, in ~", pool).await?;

    replace_chars("t'l", "t’l", "apostrophe replaced, in t'l', in ~", pool).await?;
    let sql = r#"update src.names
    set value = regexp_replace(value, 't''([a-z])', 't^\1', 'g')
    where value ~ 't''[a-z]'"#;
    execute_sql(sql, "apostrophe retained when after other t, in ~", pool).await?;

    let sql = r#"update src.names
    set value = regexp_replace(value, 'a''([a-zA-Z])', 'a^\1', 'g') 
    where value ~ 'a''[a-zA-Z]'"#;
    execute_sql(sql, "apostrophe retained when after a, in ~", pool).await?;

    let sql = r#"update src.names
    set value = regexp_replace(value, '([a-zA-Z])''a', '\1^a', 'g') 
    where value ~ '[a-zA-Z]''a'"#;
    execute_sql(sql, "apostrophe retained when before a, in ~", pool).await?;

    replace_chars("'М.Д. Інститут кардіології ім. Стражеск", "''М.Д. Інститут кардіології ім. Стражеск''", "apostrophe added to М.Д. Інститут кардіології ім. Стражеск, in ~", pool).await?;
     
    /* 
    -- remve odd superflous apostrophes
        update src.names
    set value = trim(replace(value, '''', ''))
    where (value like '''%' or value like '%''')
    and length(value) - length(replace(value, '''', '')) = 1

    -- some nother odd ones (before looking at pairs)
    

    -- at this stage possible to safely do those names with paired apostrophes 
    -- turning them into 66 99 quotes
    
    update src.names 
    set value = regexp_replace(value, '''(.*)''', '“\1”') 
    where length(value) - length(replace(value, '''', '')) = 2
    
    */


    
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



async fn execute_sql(sql: &str, description: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let n = sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();
    if n > 0 {
        if n == 1 {
            info!("{}", description.replace ("~", "1 record"));
        } 
        else {
            info!("{}", description.replace ("~", format!("{n} records").as_str()));
        };
    }
    Ok(())
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
            change_type_id = case when change_type_id is null then '1'
                else change_type_id||', 1'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like U&'%\{unicode_char}%'; "#);
     
    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        info!("{char_description} characters removed from {n} records");
    }

    Ok(())
}


async fn replace_chars(chars: &str, replacement: &str, description: &str, 
              pool: &Pool<Postgres>) -> Result<(), AppError> {

    let chars2 = if chars.contains("'") {chars.replace("'", "''")} else {chars.to_string()};
    let ch_type = format!("({chars2}) replaced by ({replacement})");
    let sql  = format!(r#"update src.names
            set value = replace(value, '{chars2}', '{replacement}'),
            changed = true,
            change_type_id = case when change_type_id is null then '5'
                else change_type_id||', 5'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like '%{chars2}%' "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        if n == 1 {
            info!("{}", description.replace ("~", "1 record"));
        } 
        else {
            info!("{}", description.replace ("~", format!("{n} records").as_str()));
        };
    }
    Ok(())

}


async fn replace_unicode_char(unicode_char: &str, char_description: &str, 
    replacement: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let ch_type = format!("(\\u{unicode_char}, {char_description}) replaced by ({replacement})");
    let sql  = format!(r#"update src.names
            set value = replace(value, U&'\{unicode_char}', '{replacement}'),
            changed = true,
            change_type_id = case when change_type_id is null then '3'
                else change_type_id||', 3'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like U&'%\{unicode_char}%'; "#);

    let n = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?.rows_affected();

    if n > 0 {
        info!("{char_description} characters replaced by ({replacement}) in {n} records");
    }

    Ok(())
}

/* 
async fn apos_to_right_single_quote(chars: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let replacement = chars.replace("'", "’");
    let ch_type = format!("({chars}) replaced by ({replacement})");
    let chars2 = chars.replace("'", "''");
    let sql  = format!(r#"update src.names
            set value = replace(value, '{chars2}', '{replacement}'),
            changed = true,
            change_id = case when change_id is null then '100'
                else change_id||', 100'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like '%{chars2}%' "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(res.rows_affected())
}


async fn apo_to_exponent_sign(chars: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let replacement = chars.replace("'", "^");
    let ch_type = format!("({chars}) replaced by ({replacement})");
    let sql  = format!(r#"update src.names
            set value = replace(value, '{chars}', '{replacement}'),
            changed = true,
            change_id = case when change_id is null then '102'
                else change_type||', 102'
            end,
            change_type = 
                case when change_type is null then '{ch_type}'
                else change_type||', '||'{ch_type}'
            end
            where value like '%{chars}%' "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

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

async fn double_quotes_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from src.names 
    where value like '%"%'"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(r)
}


use sqlx::{Pool, Postgres};
use log::info;
use crate::AppError;


pub async fn create_lang_names (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update rec.names rn
        set lang_name = lower(display_name)"#;

    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("lower case version of display names copied to lang names");
    info!("lang names to be simplified to remove punctuation");
    simplify_lang_names(pool).await?;
        
    Ok(())
}

async fn combine_lang_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update rec.names rn
        set lang = case 
        when der_lang is not null then der_lang
        else ror_lang
        end"#;

    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(())
}

async fn simplify_lang_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
       
    // Remove punctuation from lc names (Will also be used to support match and script name creation)
     
    let mut punctuation = remove_char(".", pool).await?;    // commas, semi-colons and full stops
    punctuation += remove_char(",", pool).await?;
    punctuation += remove_char(";", pool).await?;
    punctuation += remove_char(":", pool).await?;
    info!("{} commas, full stops, colons and semi-colons removed from lang names", punctuation);
                        
    let mut brackets = remove_char("(", pool).await?;   // parentheses and brackets
    brackets += remove_char(")", pool).await?;
    info!("{} parantheses characters removed from lang names", brackets);

    let mut brackets = remove_char("[", pool).await?;
    brackets += remove_char("]", pool).await?;
    info!("{} bracket characters removed from lang names", brackets);
    
    let mut smart_single_quotes = remove_char("‘", pool).await?;
    smart_single_quotes += remove_char("’", pool).await?;
    info!("{} smart quote characters removed from lang names", smart_single_quotes);

    let mut smart_double_quotes = remove_char("“", pool).await?;
    smart_double_quotes += remove_char("”", pool).await?;
    smart_double_quotes += remove_char("«", pool).await?;
    smart_double_quotes += remove_char("»", pool).await?;
    smart_double_quotes += remove_char("„", pool).await?;
    smart_double_quotes += remove_char(",,", pool).await?;
    info!("{} smart quote characters removed from lc names", smart_double_quotes);

    let res  = remove_char("''", pool).await?;
    info!("{} apostrophes removed from lang names", res);
    let res  = remove_char("\"", pool).await?;
    info!("{} stright double quotes removed from lang names", res);

    punctuation += remove_char("/", pool).await?;
    punctuation += remove_char("|", pool).await?;
    info!("{} sundry punctuation removed from lang names", punctuation);
   
    info!("");
    Ok(())
}


async fn remove_char(char: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update rec.names
            set lang_name = replace(lang_name, '{char}', '')
            where lang_name like '%{char}%'; "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(res.rows_affected())
}


pub async fn derive_lang_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    info!("{} names with, initially, no derived language code", blank_der_langs_num(pool).await?);
    let nonacro = blank_nonacro_der_langs_num(pool).await?;
    info!("{nonacro} non-acronym names with, initially, no derived language code");
    info!("");
    
    // Add languages if possible, using location of org and key words or word parts
    
    add_langs_for_nonlatin_codes(pool).await?;
    
    update_hospital_names_1(pool).await?;
    update_hospital_names_2(pool).await?;

    update_university_names_1(pool).await?;
    update_university_names_2(pool).await?;

    update_institute_names_1(pool).await?;
    update_institute_names_2(pool).await?;

    update_spanish_names(pool).await?;
    update_portuguese_names(pool).await?;
    update_japanese_names(pool).await?;
    update_chinese_names(pool).await?;
    update_french_names(pool).await?;
    update_german_names(pool).await?;
    update_italian_names(pool).await?;
    update_dutch_names(pool).await?;
    update_danish_names(pool).await?;
    update_swedish_names(pool).await?;
    update_finnish_names(pool).await?;
    update_norwegian_names(pool).await?;
    update_indian_names(pool).await?;
    update_iranian_names(pool).await?;
    update_russian_names(pool).await?;
    update_ukrainian_names(pool).await?;
    update_serbian_names(pool).await?;
    update_bulgarian_names(pool).await?;
    update_israeli_names(pool).await?;
    update_korean_names(pool).await?;
    update_greek_names(pool).await?;
    update_english_names_1(pool).await?;
    update_english_names_2(pool).await?;

    add_bd_lang_code_to_comm_orgs(pool).await?;
    
    // Do language of acronyms where all other names have the same language
    // See what are left
   
    info!("");
    info!("{} remaining names with blank derived language", blank_der_langs_num(pool).await?);
    let new_nonacro = blank_nonacro_der_langs_num(pool).await?;
    info!("{new_nonacro} remaining non-acronym names with blank derived language");
    info!("{:.2}% - percentage of non-acronym names without language codes", 100.0 * new_nonacro as f32 / nonacro as f32);
    
    combine_lang_codes(pool).await?;
    info!("Derived and ror-sourced language codes combined");
    
    let new_comb_nonacro = blank_nonacro_langs_num(pool).await?;
    info!("{new_comb_nonacro} remaining non-acronym names with blank language code");
    info!("{:.2}% - percentage of non-acronym names without language codes", 100.0 * new_comb_nonacro as f32 / nonacro as f32);
    info!("");
    Ok(())
}


async fn blank_der_langs_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from rec.names 
    where der_lang is null"#;

    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(r)
}

async fn blank_nonacro_der_langs_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from rec.names 
    where der_lang is null
    and name_type <> 10"#;
   
    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(r)
}

async fn blank_nonacro_langs_num(pool: &Pool<Postgres>) -> Result<i64, AppError> {

    let sql  = r#"select count(*) from rec.names 
    where lang is null
    and name_type <> 10"#;
   
    let r: i64 = sqlx::query_scalar(sql).fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(r)
}


pub async fn assign_lang(names: Vec<&str>, lang_code: &str, countries: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let mut word_list = "".to_string();
    for i in 0..names.len() {
        let comparator = if names[i].starts_with("^") { 
            format!("lang_name like '{}%'", &names[i][1..]) 
        }
        else { 
            format!("lang_name like '%{}%'", names[i]) 
        };
        let comparison = format!(" {}{comparator}", if i > 0 {"or "} else {""});
        word_list += comparison.as_str();
    }

    let sql = if countries == "" {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null and name_type <> 10
                and ({word_list});"#)
    } 
    else {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null and name_type <> 10
                and country_code in ({countries})
                and ({word_list});"#)
    };
    
    let res = sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql))?;
    Ok(res.rows_affected())
}


pub async fn assign_lang_using_display_name(names: Vec<&str>, lang_code: &str, countries: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let mut word_list = "".to_string();
    for i in 0..names.len() {
        let comparator = if names[i].starts_with("^") { 
            format!("display_name like '{}%'", &names[i][1..]) 
        }
        else { 
            format!("display_name like '%{}%'", names[i]) 
        };
        let comparison = format!(" {}{comparator}", if i > 0 {"or "} else {""});
        word_list += comparison.as_str();
    }

    let sql = if countries == "" {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null and name_type <> 10
                and ({word_list});"#)
    } 
    else {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null and name_type <> 10
                and country_code in ({countries})
                and ({word_list});"#)
    };
    
    let res = sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql))?;
    Ok(res.rows_affected())
}


pub async fn add_bd_lang_code_to_comm_orgs(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update rec.names n
                set der_lang = 'bd'
                from src.type t
                where n.id = t.id
                and der_lang is null
                and t.org_type = 'company'"#;

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} remaining names of commercial organisations given 'bd' language code", res.rows_affected());
  
    Ok(())
}


pub async fn update_hospital_names_1(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["ospedale"], "it", "", pool).await?; 
    records_affected += assign_lang(vec!["ziekenhuis", "ziekenhuizen"], "nl", "", pool).await?; 
    records_affected += assign_lang(vec!["sykehus"], "no", "", pool).await?; 
    records_affected += assign_lang(vec!["sjukhus"], "sv", "", pool).await?; 
    records_affected += assign_lang(vec!["sairaala"], "fi", "", pool).await?; 

    records_affected += assign_lang(vec!["spitalzentrum centre hospitalier"], "de, fr", "", pool).await?;
    records_affected += assign_lang(vec!["hôpita", "hospitalo-universitaire", "hospitalier", "hospitalière"], "fr", "", pool).await?;
    
    records_affected += assign_lang(vec!["krankenhaus", "klinikum", "klinische"], "de", "", pool).await?; 
    records_affected += assign_lang(vec!["hospitalorden", "konventhospital", "bürgerhospital", "clemenshospital", "franziskushospital", 
                    "franziskus hospital", "katharinenhospital", "luisenhospital"], "de", "'DE', 'AT', 'CH', 'LI', 'LU'", pool).await?;
    records_affected += assign_lang(vec!["hospital zum", "marienhospital", "marien-hospital", "marien hospital", "antonius-hospital", 
                    "^hospital "], "de", "'DE', 'AT', 'CH', 'LI', 'LU'", pool).await?;
    records_affected += assign_lang(vec!["elisabeth-hospital", "johannes-hospital", "josef-hospital", "josefs-hospital", "josefs hospital", 
                    "westfalen-lippe hospital"], "de", "'DE', 'AT', 'CH', 'LI', 'LU'", pool).await?;

    records_affected += assign_lang(vec!["hospitalet", "hospitalernes", "universitetshospital", "hospitaler", "hospitalsenhed", 
                     "kommunehospital", "gigthospital", "privathospital", "regionshospital", "psykiatriske hospital",
                     "midt- og vestsjællands hospital"], "da", "'DK'", pool).await?;
    
    records_affected += assign_lang(vec!["nemocnice"], "cs", "", pool).await?; 
    records_affected += assign_lang(vec!["nemocnica"], "sk", "'SK'", pool).await?; 
    records_affected += assign_lang(vec!["bolnica"], "hr", "'HR'", pool).await?; 
    records_affected += assign_lang(vec!["bolnica"], "bs", "'BA'", pool).await?; 
    records_affected += assign_lang(vec!["bolnišnica"], "sl", "'SL'", pool).await?; 
    records_affected += assign_lang(vec!["hospitalarius"], "la", "", pool).await?;
    
    records_affected += assign_lang(vec!["^hospital of ", "university hospital", "general hospital", "childrens hospital", "maternity hospital", "womens hospital", "dental hospital", "eye hospital"], 
    "en", "", pool).await?;
        
    records_affected += assign_lang(vec!["hospitality", "hospitalist"], "en", "", pool).await?;

    info!("{} language codes added to hospital names", records_affected);
    
    Ok(())
}


pub async fn update_hospital_names_2(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["hospitalet", "hospitales", "hospitalario", "hospitalaria", "hospital universitario",  
                     "hospital infantil", "hospital nacional", "del hospital", "hospital general"], "es",
                     "'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 
                     'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
    
    records_affected += assign_lang(vec!["hospitalari", "hospital universitari", "hospital comarcal", "hospital verge", "hospital sant",
                     "hospitalàries", "hospital de sant", "hospital mare de déu", "hospital dolot i comarcal",
                     "hospital de la santa creu", "hospital del mar dinvestigacions mèdiques", "hospital de tortosa verge",
                     "pius hospital", "ajuntament"], "ca", "'ES'", pool).await?;
    
    records_affected += assign_lang(vec!["hospital infantil", "hospital nacional", "del hospital", 
                    "hospital del", "^hospital ", "especializado hospital"], "es", 
                    "'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 
                    'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
    records_affected += assign_lang(vec!["en méxico hospital", "hospital zonal", "el hospital", "investigación hospital", 
                    "hospital italiano", "hospital evangélico", "diabetología hospital"], "es", 
                    "'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 
                    'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
    
    records_affected += assign_lang(vec!["hospitaleira", "hospitalar", "^hospital "], "pt", 
                   "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;
    records_affected += assign_lang(vec!["real hospital", "hospital das ", "hospital da ","hospital de ",
                   "hospital-escola", "cuf infante santo hospital", "cuf porto hospital", "clinicas hospital"], "pt", 
                   "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;

    records_affected += assign_lang(vec!["hospital de la", "hospital del", "^hospital general", "^hospital monte"], "es", "'US'", pool).await?;
    records_affected += assign_lang(vec!["^hospital"], "ms", "'MY', 'SG'", pool).await?;
    records_affected += assign_lang(vec!["hospital ya ", "hospitali"], "sw", "", pool).await?;
    records_affected += assign_lang(vec!["hospital geniko"], "el", "'GR'", pool).await?;

    // All the rest of the 'hospital's default to english
    
    records_affected += assign_lang(vec!["hospital"], "en", "", pool).await?;
   
    records_affected += assign_lang(vec!["hospitaal"], "nl", "'NL', 'BE'", pool).await?; 
    records_affected += assign_lang(vec!["hospitaal"], "af", "'ZA'", pool).await?; 
    records_affected += assign_lang(vec!["spitalor"], "sq", "'AL'", pool).await?; 
    records_affected += assign_lang(vec!["spitalul"], "ro", "'RO'", pool).await?;
    records_affected += assign_lang(vec!["^spital", " spital"], "de", "'DE', 'CH', 'AT'", pool).await?;
    
    info!("{} additional language codes added to hospital names", records_affected);
    
    Ok(())
}


pub async fn update_university_names_1(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["iunivesite"], "sm", "'WS'", pool).await?; 
    records_affected += assign_lang(vec!["iyunivesithi"], "xh", "'ZA'", pool).await?; 

    records_affected += assign_lang(vec!["universidades"], "es", 
        "'AR', 'BO', 'BY', 'BZ', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'GT', 'HN', 'MX', 
        'NI', 'PA', 'PE', 'PH', 'PR', 'PY', 'QA', 'SV', 'TT', 'US', 'UY', 'VE'", pool).await?; 
    records_affected += assign_lang(vec!["universidade"], "pt", 
        "'AO', 'BR', 'CV', 'GW', 'MO', 'MZ', 'PT', 'ST', 'TL'", pool).await?; 
    records_affected += assign_lang(vec!["universidade"], "gl", "'ES'", pool).await?; 
    records_affected += assign_lang(vec!["iese business school universidad de navarra"], "es, en", "", pool).await?;
    records_affected += assign_lang(vec!["universidad"], "es", "", pool).await?; 
    
    records_affected += assign_lang(vec!["universitaire"], "fr", "", pool).await?; 
     
    records_affected += assign_lang(vec!["universitaria"], "es", 
        "'AR', 'BO', 'BY', 'BZ', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'GT', 'HN', 'MX', 
        'NI', 'PA', 'PE', 'PH', 'PR', 'PY', 'QA', 'SV', 'TT', 'US', 'UY', 'VE'", pool).await?; 
    records_affected += assign_lang(vec!["universitária"], "pt", 
        "'AO', 'BR', 'CV', 'GW', 'MO', 'MZ', 'PT', 'ST', 'TL'", pool).await?; 
    records_affected += assign_lang(vec!["universitària"], "ca", "'ES'", pool).await?; 
  
    records_affected += assign_lang(vec!["universitario"], "es", 
        "'AR', 'BO', 'BY', 'BZ', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'GT', 'HN', 'MX', 
        'NI', 'PA', 'PE', 'PH', 'PR', 'PY', 'QA', 'SV', 'TT', 'US', 'UY', 'VE'", pool).await?; 
    records_affected += assign_lang(vec!["universitário", "universitario"], "pt", 
        "'AO', 'BR', 'CV', 'GW', 'MO', 'MZ', 'PT', 'ST', 'TL'", pool).await?; 

    records_affected += assign_lang(vec!["universitaria", "universitario", "universitari"], "it", "'CH', 'IT'", pool).await?; 
    records_affected += assign_lang(vec!["universitaris", "universitari"], "ca", "'ES'", pool).await?; 

    records_affected += assign_lang(vec!["universitare"], "sq", "'XK'", pool).await?; 
    records_affected += assign_lang(vec!["universitar", "universitară", "universitare", "universitatea"], "ro", "'RO'", pool).await?; 

    records_affected += assign_lang(vec!["universitas universal", "universitas digital teknologi digitech university"], "id, en", "'ID'", pool).await?; 
    records_affected += assign_lang(vec!["universitas"], "id", "'ID'", pool).await?; 
    records_affected += assign_lang(vec!["universitas", "universitatis"], "la", "", pool).await?; 
    
    records_affected += assign_lang(vec!["universitāte"], "lv", "'LV', 'LT'", pool).await?; 
    records_affected += assign_lang(vec!["universität", "universitäre", "universitaet"], "de", "", pool).await?; 
    records_affected += assign_lang(vec!["universitat"], "ca", "'AD', 'ES', 'FR', 'IT'", pool).await?; 
    
    records_affected += assign_lang(vec!["universita"], "he", "'IL'", pool).await?;
    records_affected += assign_lang(vec!["università"], "mt", "'MT'", pool).await?; 
    records_affected += assign_lang(vec!["università"], "it", "", pool).await?; 
    
    records_affected += assign_lang(vec!["universitét"], "uk", "'UA'", pool).await?; 
    records_affected += assign_lang(vec!["université"], "fr", "", pool).await?; 

    records_affected += assign_lang(vec!["universitehta"], "se", "'NO'", pool).await?;
    records_affected += assign_lang(vec!["universiteit"], "nl", "'AL', 'AW', 'BE', 'CW', 'NL', 'SR'", pool).await?; 
    records_affected += assign_lang(vec!["universiteit"], "af", "'ZA'", pool).await?; 

    records_affected += assign_lang(vec!["universitesi"], "tr", "'CY', 'TR'", pool).await?; 

    records_affected += assign_lang(vec!["universiteto", "universitetas"], "lt", "'LT', 'LV'", pool).await?; 
    records_affected += assign_lang(vec!["universitāte",], "lv", "'LV', 'LT'", pool).await?; 
    
    records_affected += assign_lang(vec!["universiteti"], "uz", "'AF', 'KG', 'SY', 'UZ'", pool).await?; 
    records_affected += assign_lang(vec!["universiteti"], "sq", "'AL', 'MK', 'XK'", pool).await?; 
    records_affected += assign_lang(vec!["universiteti"], "az", "'AZ', 'TR', 'GE'", pool).await?; 
            
    records_affected += assign_lang(vec!["universitet"], "sv", "'SE', 'FI'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "da", "'FO', 'GL', 'DK'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "no", "'NO'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "kk", "'KZ'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "bg", "'BG'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "uk", "'UA'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "ru", "'RU', 'BY'", pool).await?;
    records_affected += assign_lang(vec!["universitet"], "zh", "'CN'", pool).await?;
    
    records_affected += assign_lang(vec!["universite "], "fr", "'FR', 'HT', 'RW', 'UA'", pool).await?;

    records_affected += assign_lang(vec!["universities"], "en", "", pool).await?;

    records_affected += assign_lang(vec!["construction research center universiti teknologi"], "ms, en", "", pool).await?;
    records_affected += assign_lang(vec!["universiti"], "ms", "'BN', 'MY', 'SG'", pool).await?;

    /* 
     * 
     
     univ fr, FR			
     univ en, JP, SG
        
     universality  en, FR
     universalité  fr, FR
     universale  it, IT
     universalmuseum de, AT
     conselleria de sanitat universal i salut pública ca, ES

     %universal %  en
     %lunivers %   fr
     %lunivers^   fr
     %universe sciences%  en
     %the universe%   en
     %universe and %  en
          
     
     fundação centro de estudos do universo
     univesp
     univalor
     union postale universelle
     excellence cluster universe
     exzellenzcluster universe
     univation institut für evaluation dr beywl associates
     universum bremen
     universum science center
     walter brendel centre of experimental medicine wbex at the ludwig-maximilians-universität münchen
     univ mohamed boudiaf msila
     biodiversity research institute of the universiy of barcelona
     iese business school universidad de navarra
     universia foundation
     cnrs earth  universe
     cnrs terre et univers
     inria centre at université côte dazur
     inria centre at université de lorraine
     inria centre at université grenoble alpes
     inria saclay centre at université paris-saclay
     labex univearths
     observatory for universe sciences of franche-comté burgundy
     terres univia
     univearths
     typologie et universaux linguistiques
     universcience
     univers transport interfaces nanostructures atmosphère et environnement molécules
     univrab
     univtrinita
     kanchi mamunivar centre for post graduate studies
     centro universale del bel canto
     univers foundation
     the univers foundation
     kuniv (kuwait uni)
     american universal college
     univotec
     univers moldova
     universitam
     observatoire des sciences de lunivers de la réunion
     universeum

     */
   
    info!("{} language codes added to university names", records_affected);
    
    Ok(())
}


pub async fn update_university_names_2(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;
    
    records_affected += assign_lang(vec!["hochschule münchen university of"], "de, en", "", pool).await?;
    records_affected += assign_lang(vec!["university of coimbra centro de estudos"], "pt, en", "", pool).await?;
    records_affected += assign_lang(vec!["vysoká škola manažmentu city university of"], "sk, en", "", pool).await?;

    records_affected += assign_lang(vec!["university of"], "en", "", pool).await?;

    records_affected += assign_lang(vec!["корпоративный фонд university medical center"], "ru, en", "", pool).await?;
    records_affected += assign_lang(vec!["professionshøjskolen university college nordjylland"], "da, en", "", pool).await?;
    records_affected += assign_lang(vec!["university frères mentouri"], "fr, en", "", pool).await?;
    records_affected += assign_lang(vec!["científica del sur university"], "es, en", "", pool).await?;

    records_affected += assign_lang(vec!["university"], "en", "", pool).await?;
    
    records_affected += assign_lang(vec!["univerza"], "sl", "'AT', 'IT', 'SI'", pool).await?;
    records_affected += assign_lang(vec!["univerzita"], "cs", "'CZ', 'PL'", pool).await?;
    records_affected += assign_lang(vec!["univerzita"], "sk", "'SK'", pool).await?;

    records_affected += assign_lang(vec!["univerzitet sarajevo school of"], "en, bs", "", pool).await?;
    records_affected += assign_lang(vec!["univerzitet"], "bs", "'BA', 'ME'", pool).await?;
    records_affected += assign_lang(vec!["univerzitet"], "sr", "'RS'", pool).await?;
    records_affected += assign_lang(vec!["univerzitet"], "bg", "'MK'", pool).await?;
   
    records_affected += assign_lang(vec!["univerziteta", "univerzitetska", "univerzitetski"], "bs", "'BA'", pool).await?;
    records_affected += assign_lang(vec!["univerziteta", "univerzitetska", "univerzitetski"], "sr", "'RS'", pool).await?;
    
    records_affected += assign_lang(vec!["universytet"], "uk", "'UA'", pool).await?;
    
    records_affected += assign_lang(vec!["yunivarsiitii"], "om", "'ET'", pool).await?;  
    records_affected += assign_lang(vec!["yunivesithi"], "st", "'ZA'", pool).await?;  

    records_affected += assign_lang(vec!["daigaku", "daigakkō"], "ja", "'JP'", pool).await?; 
    records_affected += assign_lang(vec!["dàxué", "dàxúe", "daxue"], "zh", "'CN', 'TW', 'HK'", pool).await?;   
    records_affected += assign_lang(vec!["panepistim", "panepistímio"], "el", "'GR'", pool).await?;   

    records_affected += assign_lang(vec!["yliopisto"], "fi", "", pool).await?;     
    
    info!("{} additional language codes added to university names", records_affected);

    Ok(())
}


pub async fn update_institute_names_1(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["laboratory of the istituto nazionale"], "it, en", "", pool).await?;   
    records_affected += assign_lang(vec!["istituto"], "it", "", pool).await?;   

    records_affected += assign_lang(vec!["instituut"], "nl", "'NL', 'BE'", pool).await?;
    
    info!("{} language codes added to institute names", records_affected);

    Ok(())
}


pub async fn update_institute_names_2(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["institute of"], "en", "", pool).await?;   
    records_affected += assign_lang(vec!["institute for"], "en", "", pool).await?;   
    
    info!("{} language codes added to institute names", records_affected);

    Ok(())
}


/*
 * GL
 -- da - grønlands, dronning 
 -- kalaallisut (ki):  pinngortitaleriffik, ilisimatusarfik, napparsimmavissua, nunatsinni, nunatta, nka
 -- perorsaanermik ilinniarfik  college of social education = ki, en
 -- rest, including upi, = en


  FO

  da -- marbejde, færøernes
  fo -- føroya, norrønt, havstovan 
  
  landsbókasavnið - national library of the faroe islands = is, en
  is - savnið, umhvørvisstovan, starfið, garráðið, us
  ki - avannaani
  
  rest english, including FAMRI

  IS

  de - hochschule hólar
  
  is - ð, stofnun, kóli, spítali, veit, hjart, skógur, bók, sók, læknis, stofa, knisetur, félag
  -- also PFS, SAk, LFÍ, LOGS, ÍSOR, PSSÍ, RANNÍS, LMFÍ, FS, RMF
  
  icetec - bd 
  Matis - bd
  Össur - bd
  Origo - bd
  Kerecis - bd
  Star-Oddi - bd
  Oculis - bd
  deCODE Genetics - bd
  Marel - bd
  
  N.B. Reykjavík Energy (Iceland) - en
  Landsvirkjun (Iceland) - is 
  
  GRO, on its own, is a brand

  Rest is en
  
*/
 
pub async fn update_english_names_1(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec![" and ", " of ", " at ", " the ", "^the ", " by ",
        " to "," under "," over "," after "," on "," all ",], "en", "", pool).await?;

    records_affected += assign_lang(vec!["college", "polytechnic", "museum", "institute", "center", "clinic",
        "library", "society"], "en", "", pool).await?;
    
    records_affected += assign_lang(vec!["academic", " data ", "alliance", "advanced", "research", "agency",
        "systems", "technology", "environmental", "association", "infirmary", "council",], "en", "", pool).await?;

    info!("{} language codes added to english names", records_affected);
    Ok(())

}


pub async fn update_english_names_2(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["foundation", " trust", "laboratory", "laboratories", 
        "academy", "school", " zoo"," park", " garden","wikimedia%"], "en", "", pool).await?;
    
    records_affected += assign_lang(vec!["municipal", "medical", "health", "sanitorium", 
        "australian", "canadian", "children", "authority", "archive", "biological", 
        "genebank", "network"], "en", "", pool).await?;

    records_affected += assign_lang(vec!["conservancy", "department", "development", "fund", "government", 
        "group", "region", "survey", "test", "territory", 
        "directorate", "observatory", "observatories"], "en", "", pool).await?;
    
    let sql = r#"update rec.names n
                set der_lang = 'en'
                where der_lang is null and n.name_type <> 10
                and country_code NOT in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                             'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')      
                and (lang_name like '%centre%'
                or lang_name like '%science%'
                or lang_name like '%initiative%'
                );"#;
        
    let res = sqlx::raw_sql(sql).execute(pool)
                .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    records_affected += res.rows_affected();

    records_affected += assign_lang(vec!["people", "women", "kids", "mother", "father", 
        "boys", "girls", "black", "white", "yellow", "blue"], "en", "", pool).await?;   

    info!("{} additional language codes added to english names", records_affected);

    Ok(())

    // institite and centre??? - soplit between anglophone and francophone...
}


pub async fn update_japanese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["kabushiki", "nippon", "kaihatsu", "bijutsukan", "kenritsu", "dokuritsu",
        " kikō", "gakkō", "gakko", "gakkou ", "kaihatsu", "-shō",
        "bunka senta", "denryoku", "gakuen", "kagaku-kan", "bungaku-kan", "-chō",], "ja", "'JP'", pool).await?;
            
            // corporation                // Japan
            // development                // art museum
            // prefectural                // independent
            // organization               // school (3)
            // development                // -prize
            // cultural center            // electric power
            // academy                    // science building
            // literature building        // district
            // specialized school

     records_affected += assign_lang(vec!["chuobyoin", "shiritsu", "kenkyūjo", "kenkyujo", "kenkyūsho", "kenkei",   
         "kyōdō", "tankyu", "kenkyusho", "kenkyuu", "kokusai", "hakubutsukan", 
         "toshoken", "byoin", "byouin", "byōin"], "ja", "'JP'", pool).await?;

            // medical center            // municipal
            // research institute (3)            // survey
            // collaboration        // research facility
            // research laboratory        // research
            // international        // museums
            // libraries        // hospitals (2)

    records_affected += assign_lang(vec!["nihon", "kinzoku", "kenkyū", "kokudo", "jitsugyo", "fukusei",             "shiryokan", "gurūpu", "shiritsuchuobyoin", "kenkyuukikou", "shiminbyoin"], "ja", "'JP'", pool).await?;
   
             // Japan                     // metal
             // research                  // national land
             // practical business        // integrated
             // information center        // group
             // municipal hospital        // research organization
             // high school for advanced study        // municipal hospital

    info!("{} language codes added to japanese records", records_affected);

    Ok(())
}


pub async fn update_chinese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["zhōngyī", "xuéyuàn", "yīyuàn", "jīgòu", "yánjiū", 
        "mínguó", "yínháng"], "zh", "'CN', 'TW', 'HK'", pool).await?;   

        // zhōngyī     (traditional) Chinese medicine
        // xuéyuàn     Educational institute (school - conservatory - academy)
        // yīyuàn      hospital
        // jīgòu       Mechanism (body - agency)
        // yánjiū      Study (Research)
        // mínguó      Republic
        // yínháng     Bank

    records_affected += assign_lang(vec!["yīyún", "yánjiùyuàn", "ybówùguǎn", "xuéxiào", "shénxué", 
            "gōngyè", "zhèngfǔ", "guójiā", "shīfàn"], "zh", "'CN', 'TW', 'HK'", pool).await?;   

        // yīyún      hospital
        // yánjiùyuàn researcher
        // bówùguǎn   museum
        // xuéxiào    school
        // shénxué    theology
        // gōngyè     industry
        // zhèngfǔ    government
        // guójiā     state, country
        // shīfàn     school

    info!("{} language codes added to chinese records", records_affected);

    Ok(())
}


pub async fn update_french_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    // initial group seen as French whatever the country
    records_affected += assign_lang(vec!["école", "laboratoire", "réseau", "société", "santé", 
            "publique", "mondiale", "équipe", "maison", "bibliothèque"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;   

    records_affected += assign_lang(vec!["agence", "académie", "ecole", "environnement", "linstitut", 
            "système", " et ", "canadienne", "banque", "gouvernement"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;       

    records_affected += assign_lang(vec!["informatique", "unité", "français", "recherche", "développement", 
            "biologie", "génétique", "observatoire", "centre ", "fédération"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;       

    records_affected += assign_lang(vec!["musée", "pôle", "études", "chimie", "clinique", 
            "conseil", "département", "faculté", "fondation", "ministère", "plateforme"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;       
    
    records_affected += assign_lang(vec!["collège", "communauté", "espace", "archives", "centrale", 
            "château", "délégation", "génomique", "hôtel-dieu", "européenne", "ambassade"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;    

    records_affected += assign_lang(vec!["^institut de", "^institut dhistoire", "^institut du ", 
            "chambre", "comité", "caisse", "médecin", "ministre", "régie", "région"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;   

    records_affected += assign_lang_using_display_name(vec!["^Inserm ", "^CH ", "^CHU ", "^CIC ", "^EA", 
        "^ERL ", "^GDR", "^U ", "^UAR", "^UMR", "^UMRS ", "^UMR_S ", "^UMS ", "^UR", "^URP ", "^US"], 
            "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
            'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;   
    
        // CH    centre hospitalier
        // CHU   centre hospitalier universitaire
        // CIC   centres d’investigation clinique
        // EA    équipe d’accueil
        // ERL   ? équipe d’accueil laboratoire
        // GDR   groupement de recherche
        // U 9999  unité ...
        // UAR   unités d'appui et de recherche
        // UMR   unité mixte de recherche
        // UMRS  unité mixte de recherche et service
        // UMR_S unité mixte de recherche et service
        // UMS   unité mixte de service
        // UR    unité de recherche
        // URP   unité de recherche ?
        // US    ? unité de service

    records_affected += assign_lang_using_display_name(vec!["^BRGM ", "^CEA ", "^Cégep ", "^CHR ", "^CHP ",
        "^CISSS ", "^CIUSSS ", "^CNRS ", "^CRP ", "^ESC ", "^ESIEE ", "^ESPI ", "^GRC ", "^HES-SO ", 
         "^IMT ", "^INSA ", "^IUT ", "^Labex "], 
        "fr", "'FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
        'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU'", pool).await?;   

    info!("{} language codes added to french records", records_affected);
    Ok(())
}

 
pub async fn update_german_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec![" für ", " und ", "klinik", "bundesamt", "hochschule", 
        "zentrum", "akademie", "allgemeine", "deutsche", "gesellschaft", "krankenhaus", 
        "wissenschaft", "arbeit", "gemeinschaft"], "de", "'DE', 'AT', 'CH'", pool).await?;   

    records_affected += assign_lang(vec!["bundesverband", "europäische", "forschung", "kantonsschule",          
        "kantonsspital", "katholische", "österreichische", "stiftung", "technische", "vereinigung", "wasser"],
        "de", "'DE', 'AT', 'CH'", pool).await?;   

    info!("{} language codes added to german records", records_affected);
    
    Ok(())
}


pub async fn update_spanish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["academia", "unidad", "agencia", "asociación", "ayuntamiento", 
        "banco", "benemérita", "biblioteca", "centro", "ciencia", "científico", "clínica", 
        "clínico", "colegio", "comisión", "consejo", "consorcio"], 
        "es", " 'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;

    records_affected += assign_lang(vec!["corporación", "departamento", "dirección", "escuela", "esperança", 
        "española", "estación", "facultad", "fundacion", "gobierno", "grupo", "institución", 
        "instituto", "laboratorio"], 
        "es", " 'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
    
    records_affected += assign_lang(vec!["médico", "milenium", "ministerio", "museo", "nacional", 
        "observatorio", "organización", "parque", "pontificia", "salud", "sanitas", "secretaría", 
        "servicio", "sistema", "sociedad", "tecnológico", "tecnm"], 
        "es", " 'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
     
    info!("{} language codes added to spanish records", records_affected);
    
    Ok(())
}

 
pub async fn update_portuguese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
     
    let mut records_affected = 0;

    records_affected += assign_lang(vec!["agência", "associação", "autoridade", "biblioteca", "comissão", 
        "ciência", "conselho", "departamento", "direção", "escola", "estudos", "faculdade", 
        "federação", "fundação", "gabinete", "grupo"], 
        "pt", "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;

    records_affected += assign_lang(vec!["investigação", "instituto", "laboratório", "ministério", "museu", 
        "observatório", "ordem", "parque", "pesquisa", "sociedade", "tecnologia", "tecnológico", 
        "unidade"], 
        "pt", "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;

    records_affected += assign_lang(vec!["^INCT de", "centro", "nacional", "esperança", "ciencia", 
        "academia", "secretaria", "governo", "prefeitura", "companhia"], 
        "pt", "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;

    info!("{} language codes added to portuguese records", records_affected);
    
    Ok(())
}


pub async fn update_italian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["accademia", "agenzia", "archivio", "associazione", "azienda", 
        "centro di ", "conservatorio", "consorzio", "dipartimento", "federazione", "fondazione", "gruppo", 
        "liceo", "ministero", "museo"], "it", "'IT', 'CH'", pool).await?;

    records_affected += assign_lang(vec!["organizzazione", "osservatorio", "pontificia", "regione", "scuola", 
        "sistema", "societa", "ufficio", "unità"], "it", "'IT', 'CH'", pool).await?;

    info!("{} language codes added to italian records", records_affected);
    
    Ok(())
}


pub async fn update_dutch_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
 
    let mut records_affected = 0;

    records_affected += assign_lang(vec!["academisch", "gemeentelijke", "groot", "koninklijke",
        " voor ", "ministerie", "nationaal", "nederlandse", "kundige", "stichting", "vereniging", "zorg",
        "stedelijk"], 
        "nl", "'NL', 'BE'", pool).await?;
    
    records_affected += assign_lang(vec!["kliniek", "medisch", "meenschap", "centrum",
        "groep", "nederlandsche", "specialisten", "fonds", "gemeente", "hogeschool", "huisarts", "maatschap",
        "gasthuis", "gezondheid"], 
        "nl", "'NL', 'BE'", pool).await?;
  
    info!("{} language codes added to dutch records", records_affected);
    
    Ok(())
}


pub async fn update_danish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
 
    let mut records_affected = 0;

    records_affected += assign_lang(vec![" fonde", "fonden", "fondet", "kommune",
        "sygehus", "dansk", "foreningen", "danmarks", "klinik", " og ", "skole", "regionshospita",
        "rigshospitalet"], 
        "da", "'DK'", pool).await?;
    
    records_affected += assign_lang(vec!["teknolog", "arkiv", "hospitaler", "tekniske",
        "privathospital", "midt", "nordvest", "biblioteket", "gigthospital", "hospitalsenheden", "styrelsen", "nationalbanken", "kræftens", "vaern"], 
        "da", "'DK'", pool).await?;
    
    info!("{} language codes added to danish records", records_affected);
    
    Ok(())
}

 
pub async fn update_swedish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec![" för ", " i ", "västra", "akademin",
        "finlands", "finska", "folktandvården", "göteborgs", "högskolan", "kungliga", "landstinget", "svenska",
        "sverige", "trafikverket", "kommun"], 
        "sv", "'SE', 'FI'", pool).await?;
    
    records_affected += assign_lang(vec!["konstmuseum", "institutet", "ningen", "huset",
        "ringen", "forskning", "förbundet", "telsern", "landsting", "lasarett", "minnesfond", "stiftelse", "kliniken", "fonden", "centrum", "vägen"], 
        "sv", "'SE', 'FI'", pool).await?;

    records_affected += assign_lang(vec!["hälsocentral", "avfallshantering", "centrallasarettet", 
        "sällskap",  "hälsans", "utvecklings", "västra", "praktikertjänst", "skandinaviska", "skånes", 
        "rinkebyakademien", "pedagogiska", "transportforsk", "Wienerbageriet", "tunga"], 
        "sv", "'SE', 'FI'", pool).await?;
     
    info!("{} language codes added to swedish records", records_affected);
    Ok(())
}


pub async fn update_finnish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
     
    let mut records_affected = 0;

    records_affected += assign_lang(vec![" ja ", "säätiö", "suomi", "helsingin",  "juhani", 
        "kansainvälisen", "instituutti", "korkeakoulu", "lääketieteellisen", "norjan", 
        "pohjois", "ruotsin", "satakunnan", "lukko", "yhtymä"], 
        "fi", "'SE', 'FI'", pool).await?;

    records_affected += assign_lang(vec!["suomalainen", "suomen", "tampereen", "turun", "vaasan", 
        "yhteis", "sairaala", "liitto", " och ", "laitos", "musseura", "puolustusvoimat", "hoitopiirin",
        "taloustutkimus", "aktiebolag", "suomen", "kuntayhtymä"], 
        "fi", "'SE', 'FI'", pool).await?;

    records_affected += assign_lang(vec!["institutet", "akademi", "forsknings", "centrum", "keskus", 
        "centralen", "räjät", "topiiri", "korkeakoul", "ymparisto", "työsuojelurahasto", "suomalainen",
        "trafikverket", "tekonivelsairaala"], 
        "fi", "'SE', 'FI'", pool).await?;
 
    info!("{} language codes added to finnish records", records_affected);
    
    Ok(())
}


pub async fn update_norwegian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["ø", "skole", "skule", " i ", "direktoratet", 
        "registeret", "kommun", "instituut", "kunnskaps", "bibliotek", "musea", "havn", "regionen"],
        "no", "'NO'", pool).await?;

    records_affected += assign_lang(vec!["teknolog", "svaret", "klinikken", "dyrehospital", "sverk", 
        "sijte", "norge", "det ", "forskning", "institutt", "heise ", "senter", "forening",
        "kunnskaps", "råd", "departementet"], 
        "no", "'NO'", pool).await?;
        
    info!("{} language codes added to norwegian records", records_affected);
    
    Ok(())
}


pub async fn update_indian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang_using_display_name(vec!["^AIIMS", "^GCE", "^GMC", "^IIIT", "^IIM", 
        "^IISER", "^IIM", "^IISER", "^IIT", "^NIPER", "^NIT", "^RDC", "^REC",
        "^SKUAST", "^JNT", "centre"], 
        "en", "'IN'", pool).await?;
       
       // 'AIIMS%'  All India Institute of Medical Sciences
       // 'GCE%'    Government College of Engineering
       // 'GMC%'    Government Medical College
       // 'IIIT%'   International Institute of Information Technology
       //           Indian Institute of Information Technology Design & Manufacturing
       // 'IIM %'   Indian Institute of Management 
       // 'IISER%'  Indian Institute of Science Education and Research
       // 'IIT %'   Indian Institute of Technology
       // 'NIPER%'  National Institute of Pharmaceutical Education and Research
       // 'NIT %'   National Institute of Technology
       // 'RDC %'   Dental College & Hospital
       // 'REC %'   Regional / Rajkiya Engineering College 
       // 'SKUAST%' Sher-e-Kashmir University of Agricultural Sciences and Technology
       // 'JNT%'    Jawaharlal Nehru Technological University

    records_affected += assign_lang_using_display_name(vec!["^KVK "], "hi", "'IN'", pool).await?;
       
    records_affected += assign_lang(vec![" vigyan", " vishwavidyalaya", " sanstha", " sansthā", 
        " vidyālaya", "krishi", "samsthana"], "hi", "'IN'", pool).await?;
    
       // KVK     Krishi Vigyan Kendra  Farm Science Center
       // vigyan           science
       // vishwavidyalaya  university school
       // sanstha          organization
       // sansthā
       // vidyālaya        school
       // krishi           agriculture
       // samsthana        institution

       info!("{} language codes added to indian records", records_affected);
    
    Ok(())
}


pub async fn update_iranian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["dāneshgāh"], "fa", "'IR'", pool).await?;
    
    // dāneshgāh    university
    
    info!("{} language codes added to iranian records", records_affected);
    Ok(())
}


pub async fn update_russian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;
    
    records_affected += assign_lang(vec!["institut ", "akademiya", "akadémiya", "oblastnoy", 
        "federalnyy", "patologii", "khirurgii", "shkola", "kombinat", "tsentr"], "ru", "'RU'", pool).await?;

    records_affected += assign_lang_using_display_name(vec!["^JSC "], "ru", "'RU'", pool).await?;

    // JSC  Scientific research institute
   
    info!("{} language codes added to russian records", records_affected);
    
    Ok(())
}


pub async fn update_ukrainian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["ukrainsky", "ukrayinska", "ukrayiny"], "uk", "'UA'", pool).await?;

    info!("{} language codes added to ukranian records", records_affected);
    
    Ok(())
}


pub async fn update_serbian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["institut", "zvezdara"], "sr", "'RS'", pool).await?;

    info!("{} language codes added to serbian records", records_affected);
    
    Ok(())
}


pub async fn update_bulgarian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["institut", "akademiya", "ministerstvo", "obshtina", 
        "muzei", "medicinska"], "bg", "'BG'", pool).await?;

    info!("{} language codes added to bulgarian records", records_affected);
    Ok(())
}


pub async fn update_israeli_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["machon ", "merkaz ", "misrad ", "misgav ", 
        "mikhlelet", "miklelet"], "he", "'IL'", pool).await?;

            // machon   institution or foundation
            // merkaz   centre
            // misrad   office
            // misgav   refuge (hospital here)
            // mikhlelet college
            // miklelet  (law) school

    info!("{} language codes added to israeli records", records_affected);
    
    Ok(())
}


pub async fn update_korean_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["daehak", "hakkyo", "taehak"], "ko", "'KR'", pool).await?;
 
    info!("{} language codes added to korean records", records_affected);
    Ok(())
}


pub async fn update_greek_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["^tei ", "panepistimio", "panepistimiako", "ellinikon", 
        "institouto"], "el", "'GR', 'CY'", pool).await?;


            // tei     Technological Educational Institute
            // panepistimio    university
            // panepistimiako  university
            // ellinikon       greek
 
    info!("{} language codes added to greek records", records_affected);
    
    Ok(())
}


pub async fn add_langs_for_nonlatin_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let mut nonlatin_names = 0;

    nonlatin_names += update_lang_code_by_country("ru", "('RU')", pool).await?;
    nonlatin_names += update_lang_code_by_country("uk", "('UA')", pool).await?;
    nonlatin_names += update_lang_code_by_country("el", "('GR', 'CY')", pool).await?;
    nonlatin_names += update_lang_code_by_country("ja", "('JP')", pool).await?;
    nonlatin_names += update_lang_code_by_country("zh", "('CN', 'TW', 'HK')", pool).await?;
    nonlatin_names += update_lang_code_by_country("ko", "('KR')", pool).await?;
    nonlatin_names += update_lang_code_by_country("bg", "('BG')", pool).await?;
    nonlatin_names += update_lang_code_by_country("be", "('BY')", pool).await?;
    nonlatin_names += update_lang_code_by_country("ky", "('KG')", pool).await?;
    nonlatin_names += update_lang_code_by_country("kk", "('KZ')", pool).await?;
    nonlatin_names += update_lang_code_by_country("mn", "('MN')", pool).await?;
    nonlatin_names += update_lang_code_by_country("uz", "('UZ')", pool).await?;
    nonlatin_names += update_lang_code_by_country("hy", "('AM')", pool).await?;
    nonlatin_names += update_lang_code_by_country("tg", "('TJ')", pool).await?;
    nonlatin_names += update_lang_code_by_country("mk", "('MK')", pool).await?;
    nonlatin_names += update_lang_code_by_country("az", "('AZ')", pool).await?;
    nonlatin_names += update_lang_code_by_country("bs", "('BA')", pool).await?;
    nonlatin_names += update_lang_code_by_country("sr", "('RS')", pool).await?;
    nonlatin_names += update_lang_code_by_country("lt", "('LT')", pool).await?;

    nonlatin_names += update_lang_code_by_script("he", "('Hebr')", pool).await?;
    nonlatin_names += update_lang_code_by_script("bo", "('Tibt')", pool).await?;
    nonlatin_names += update_lang_code_by_script("kn", "('Knda')", pool).await?;
    nonlatin_names += update_lang_code_by_script("hi", "('Deva')", pool).await?;
    nonlatin_names += update_lang_code_by_script("th", "('Thai')", pool).await?;

    // This last group are US university societies or founations
    // that use Greek letter names as  their title. The abbreviations
    // are in a Greek script, but are derived from English words in the
    // sense that they use Greek letter names as English words.

    let sql  = r#"update rec.names
        set der_lang = 'en'
        where der_script = 'Grek'
        and country_code = 'US';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_names += res.rows_affected();

    info!("{} Non-latin language codes applied", nonlatin_names); 

    Ok(())
}


async fn update_lang_code_by_country(lang_code: &str, country_code: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update rec.names
        set der_lang = '{lang_code}'
        where der_lang is null 
        and der_script <> 'Latn'
        and country_code in {country_code};"#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn update_lang_code_by_script(lang_code: &str, script_code: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update rec.names n
        set der_lang = '{lang_code}'
        where der_lang is null 
        and der_script in {script_code} ;"#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


/* 
pub async fn obtain_manual_coding_list(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql  = r#"drop table if exists orgs.manual_codes;
                CREATE TABLE orgs.manual_codes (
                    id varchar NOT NULL,
                    name varchar NULL,
                    lc_value varchar NULL,
                    name_type varchar NULL,
                    lang_code varchar NULL,
                    notes varchar NULL
                );
                CREATE INDEX manual_codes_id ON orgs.manual_codes USING btree (id);
                CREATE INDEX manual_codes_name ON orgs.manual_codes USING btree (lc_value); "#;

    sqlx::raw_sql(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql  = r#"copy orgs.manual_codes FROM 'E:\Resources - Data\ROR\manual_coding.csv' DELIMITER ',' CSV HEADER; "#;

    let res = sqlx::raw_sql(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} manual coding records imported from file", res.rows_affected());

    Ok(())
}


pub async fn apply_manual_coding_list(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql  = r#"update orgs.ror_names n
                set lang_code = m.lang_code
                from orgs.manual_codes m
                where n.id = m.id
                and n.lc_value = m.lc_value
                and n.lang_code is null
                and n.name_type <> 10; "#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} language codes applied from manual coding data", res.rows_affected());

    Ok(())
}


pub async fn update_lang_code_source(srce: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = format!(r#"update orgs.ror_names
            set lang_source = '{}'
            where lang_source is null
            and lang_code is not null;"#, srce);
 
    let res = sqlx::raw_sql(&sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql))?;
        info!("{} records updated with '{}' as language source", res.rows_affected(), srce);

    Ok(())
}
*/
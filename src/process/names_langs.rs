use sqlx::{Pool, Postgres};
use log::info;
use crate::AppError;


pub async fn create_rec_names (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"insert into rec.names(ident, id, orig_value, value, 
       name_type, is_ror_name, lang)
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
    Ok(())
}


pub async fn create_lc_names (pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update rec.names rn
        set lc_value = lower(value)"#;

    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    simplify_lc_names(pool).await?;
        
    Ok(())
}


async fn simplify_lc_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Remove punctuation from lc names (Will also be used to support script detection)
                           
    let mut punctuation = remove_char(".", pool).await?;    // commas, semi-colons and full stops
    punctuation += remove_char(",", pool).await?;
    punctuation += remove_char(";", pool).await?;
    punctuation += remove_char(":", pool).await?;
    info!("{} commas, full stops, colons and semi-colons removed from lc names", punctuation);
                        
    let mut brackets = remove_char("(", pool).await?;   // parentheses and brackets
    brackets += remove_char(")", pool).await?;
    info!("{} parantheses characters removed from lc names", brackets);

    let mut brackets = remove_char("[", pool).await?;
    brackets += remove_char("]", pool).await?;
    info!("{} bracket characters removed from lc names", brackets);
    
    let mut smart_single_quotes = remove_char("‘", pool).await?;
    smart_single_quotes += remove_char("’", pool).await?;
    info!("{} smart quote characters removed from lc names", smart_single_quotes);

    let mut smart_double_quotes = remove_char("“", pool).await?;
    smart_double_quotes += remove_char("”", pool).await?;
    smart_double_quotes += remove_char("«", pool).await?;
    smart_double_quotes += remove_char("»", pool).await?;
    smart_double_quotes += remove_char("„", pool).await?;
    smart_double_quotes += remove_char(",,", pool).await?;
    info!("{} smart quote characters removed from lc names", smart_double_quotes);

    let res  = remove_char("''", pool).await?;
    info!("{} apostrophes removed from lc names", res);
    let res  = remove_char("\"", pool).await?;
    info!("{} stright double quiotes removed from lc names", res);

    //let mut punctuation = remove_char("-", pool).await?;  // ampersands, slashes
    let mut punctuation = remove_char("&", pool).await?;
    punctuation += remove_char("/", pool).await?;
    punctuation += remove_char("|", pool).await?;
    info!("{} sundry punctuation removed from lc names", punctuation);
   
    let mut bullets = remove_char("·", pool).await?;       // middle dot, U+00b7
    bullets += remove_char("・", pool).await?;      // katakana middle dot, U+30fb
    info!("{} Bullets removed from lc names", bullets);
    info!("");
    Ok(())
}


async fn remove_char(char: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update rec.names
            set lc_value = replace(lc_value, '{char}', '')
            where lc_value like '%{char}%'; "#);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(res.rows_affected())
}



pub async fn derive_lang_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {

    add_cm_lang_code_to_comm_orgs(pool).await?;
    
    // Add languages if possible, using location of org and key words or word parts
    
    update_hospital_names_1(pool).await?;
    update_hospital_names_2(pool).await?;

    update_university_names_1(pool).await?;
    /* 
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
    update_english_names(pool).await?;
    */
    
    // Do language of acronyms where all other names have the same language
    // See what are left
    info!("");
    Ok(())
}

pub async fn add_cm_lang_code_to_comm_orgs(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update rec.names n
                set der_lang = 'bd'
                from src.type t
                where n.id = t.id
                and t.org_type = 'company'"#;

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} names of commercial organisations given 'bd' language code", res.rows_affected());
  
    Ok(())
}


pub async fn update_hospital_names_1(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["ospedale"], "it", "", pool).await?; 
    records_affected += assign_lang(vec!["ziekenhuis"], "nl", "", pool).await?; 
    records_affected += assign_lang(vec!["sykehus"], "no", "", pool).await?; 
    records_affected += assign_lang(vec!["sjukhus"], "sv", "", pool).await?; 
    records_affected += assign_lang(vec!["sairaala"], "fi", "", pool).await?; 

    records_affected += assign_lang(vec!["spitalzentrum centre hospitalier"], "de, fr", "", pool).await?;
    records_affected += assign_lang(vec!["hôpita", "hospitalo-universitaire", "hospitalier", "hospitalière"], "fr", "", pool).await?;
    
    records_affected += assign_lang(vec!["krankenhaus", "klinikum", "klinische"], "de", "", pool).await?; 
    records_affected += assign_lang(vec!["hospitalorden", "konventhospital", "bürgerhospital", "clemenshospital", "franziskushospital", 
                    "franziskus hospital", "katharinenhospital", "luisenhospital"], "de", "'DE', 'AT', 'CH', 'LI', 'LU'", pool).await?;
    records_affected += assign_lang(vec!["hospital zum", "marienhospital", "marien-hospital", "marien hospital", "antonius-hospital", 
                    "$hospital "], "de", "'DE', 'AT', 'CH', 'LI', 'LU'", pool).await?;
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
    
    records_affected += assign_lang(vec!["$hospital of ", "university hospital", "general hospital", "childrens hospital",
                        "maternity hospital", "womens hospital", "dental hospital", "eye hospital"], "en", "", pool).await?;
    /*
    records_affected += assign_lang(vec!["teaching hospital", "central hospital", "state hospital", "provincial hospital",
                        "regional hospital", "community hospital", "orthopaedic hospital", "mental hospital" ], "en", "", pool).await?;
    records_affected += assign_lang(vec!["psychiatric hospital", "psychiatry hospital", "rehabilitation hospital", 
                        "district hospital", "memorial hospital", "day hospital", "cottage hospital"], "en", "", pool).await?;
    records_affected += assign_lang(vec!["hospitals for children", "military hospital", "naval hospital", "cantonal hospital",
                        "college hospital", "city hospital", "and hospital", "british hospital"], "en", "", pool).await?;
    records_affected += assign_lang(vec!["emergency hospital", "center hospital", "diseases hospital", "heart hospital", 
                                         "army hospital", "county hospital"], "en", "", pool).await?;
    */
    
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
                    "hospital del", "$hospital ", "especializado hospital"], "es", 
                    "'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 
                    'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
    records_affected += assign_lang(vec!["en méxico hospital", "hospital zonal", "el hospital", "investigación hospital", 
                    "hospital italiano", "hospital evangélico", "diabetología hospital"], "es", 
                    "'AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'IT', 'GI', 'GQ', 'GT', 'HN', 'MX', 'MW', 'NI', 'PA', 'PE', 
                    'PR', 'PY', 'SV', 'TT', 'UY', 'VE'", pool).await?;
    
    records_affected += assign_lang(vec!["hospitaleira", "hospitalar", "$hospital "], "pt", 
                   "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;
    records_affected += assign_lang(vec!["real hospital", "hospital das ", "hospital da ","hospital de ",
                   "hospital-escola", "cuf infante santo hospital", "cuf porto hospital", "clinicas hospital"], "pt", 
                   "'PT', 'BR', 'CV', 'AO', 'MO', 'MZ', 'GW', 'ST', 'TL'", pool).await?;

    records_affected += assign_lang(vec!["hospital de la", "hospital del", "$hospital general", "$hospital monte"], "es", "'US'", pool).await?;
    records_affected += assign_lang(vec!["$hospital"], "ms", "'MY', 'SG'", pool).await?;
    records_affected += assign_lang(vec!["hospital ya ", "hospitali"], "sw", "", pool).await?;
    records_affected += assign_lang(vec!["hospital geniko"], "el", "'GR'", pool).await?;

    // All the rest of the 'hospital's default to english
    
    records_affected += assign_lang(vec!["hospital"], "en", "", pool).await?;
   
    records_affected += assign_lang(vec!["hospitaal"], "nl", "'NL', 'BE'", pool).await?; 
    records_affected += assign_lang(vec!["hospitaal"], "af", "'ZA'", pool).await?; 
    records_affected += assign_lang(vec!["spitalor"], "sq", "'AL'", pool).await?; 
    records_affected += assign_lang(vec!["spitalul"], "ro", "'RO'", pool).await?;
    records_affected += assign_lang(vec!["$spital", " spital"], "de", "'DE', 'CH', 'AT'", pool).await?;
    
    info!("{} language codes added to hospital names", records_affected);
    
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

    records_affected += assign_lang(vec!["universiti"], "ms", "'BN', 'MY', 'SG'", pool).await?;

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

    records_affected += assign_lang(vec!["univerzitet"], "bs", "'BA', 'ME'", pool).await?;
    records_affected += assign_lang(vec!["univerzitet"], "sr", "'RS'", pool).await?;
    records_affected += assign_lang(vec!["univerzitet"], "bg", "'MK'", pool).await?;
   
    records_affected += assign_lang(vec!["univerziteta", "univerzitetska", "univerzitetski"], "bs", "'BA'", pool).await?;
    records_affected += assign_lang(vec!["univerziteta", "univerzitetska", "univerzitetski"], "sr", "'RS'", pool).await?;
    
    records_affected += assign_lang(vec!["universytet"], "uk", "'UA'", pool).await?;
    
    records_affected += assign_lang(vec!["yunivarsiitii"], "om", "'ET'", pool).await?;  
    records_affected += assign_lang(vec!["yunivesithi"], "st", "'ZA'", pool).await?;  
    
    records_affected += assign_lang(vec!["yliopisto"], "fi", "", pool).await?;     


    /* 
     * 
     
     univ fr, FR			
     univ en, JP, SG
     
     univeristy  en, NE
     universit   en, JP
     univesity   en, IN, RO
               
'univeristy', 'universit$', 'univesity'
   
     
     universality  en, FR
     universalité  fr, FR
     universale  it, IT
     universalmuseum de, AT
     conselleria de sanitat universal i salut pública ca, ES

     %universal %  en
     %lunivers %   fr
     %lunivers$   fr
     %universe sciences%  en
     %the universe%   en
     %universe and %  en

          
     
     fundação centro de estudos do universo
     univesp
     univalor
     union postale universelle
     excellence cluster universe
     exzellenzcluster universe
     univation institut für evaluation dr beywl  associates
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
     well-being organizations digital habitability education universality relations knowledge
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
     construction research center universiti teknologi malaysia
     observatoire des sciences de lunivers de la réunion
     bryansk regional scientific universal library f i tyutchev
     universeum
         

     */
   
    info!("{} language codes added to university names", records_affected);
    
    Ok(())
}


pub async fn assign_lang(names: Vec<&str>, lang_code: &str, countries: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let mut word_list = "".to_string();
    for i in 0..names.len() {
        let comparator = if names[i].starts_with("$") { 
            format!("lc_value like '{}%'", &names[i][1..]) 
        }
        else { 
            format!("lc_value like '%{}%'", names[i]) 
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


/* 
pub async fn update_english_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;
    
    let sql = r#"update rec.names n
                set der_lang = 'en'
        where der_lang is null and n.name_type <> 10
        and (lc_value like '% and %' 
        or lc_value like '% of %'
        or lc_value like '% the %'
        or lc_value like 'the %'
        );"#;
   
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'en'
        where der_lang is null and n.name_type <> 10
        and (lc_value like '%university%' 
        or lc_value like '%college%'
        or lc_value like '%polytechnic%'
        or lc_value like '%museum%'
        or lc_value like '%institute%'
        or lc_value like '%center%'
        or lc_value like '%clinic%'
        or lc_value like '%library%'
        or lc_value like '%society%');"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    
    let sql = r#"update rec.names n
                set der_lang = 'en'
        where der_lang is null and n.name_type <> 10
        and (lc_value like '%academic%' 
        or lc_value like '% data %'
        or lc_value like '%alliance%'
        or lc_value like '%advanced%'
        or lc_value like '%research%'
        or lc_value like '%agency%'
        or lc_value like '%systems%'
        or lc_value like '%technology%'
        or lc_value like '%environmental%'
        or lc_value like '%association%'
        or lc_value like '%infirmary%'
        or lc_value like '%council%'
        );"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'en'
                where der_lang is null and n.name_type <> 10
        and (lc_value like '%foundation%'
        or lc_value like '% trust%'
        or lc_value like '%laboratory%'
        or lc_value like '%laboratories%'
        or lc_value like '%academy%'
        or lc_value like '%school%'
        or lc_value like '% zoo%'
        or lc_value like '% park%'
        or lc_value like '% garden%'
        or lc_value like '%wikimedia%');"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'en'
                where der_lang is null and n.name_type <> 10
        and (lc_value like '%municipal%'
        or lc_value like '%medical%'
        or lc_value like '%health%'
        or lc_value like '%sanitorium%'
        or lc_value like '%australian%'
        or lc_value like '%canadian%'
        or lc_value like '%children%'
        or lc_value like '%authority%'
        or lc_value like '%archive%'
        or lc_value like '%biological%'
        or lc_value like '%genebank%');"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                    set der_lang = 'en'
                    where der_lang is null and n.name_type <> 10
            and (lc_value like '%conservancy%'
            or lc_value like '%department%'
            or lc_value like '%development%'
            or lc_value like '%fund%'
            or lc_value like '%government%'
            or lc_value like '%group%'
            or lc_value like '%region%'
            or lc_value like '%survey%'
            or lc_value like '%test%'
            or lc_value like '%territory%'
            or lc_value like '%directorate%'
            or lc_value like '%genebank%');"#;
    
        let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        total_records_affected += res.rows_affected();

        let sql = r#"update rec.names n
                        set der_lang = 'en'
                        where der_lang is null and n.name_type <> 10
                and country_code NOT in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                             'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')      
                and (lc_value like '%centre%'
                or lc_value like '%science%'
                or lc_value like '%initiative%'
                );"#;
        
            let res = sqlx::raw_sql(sql).execute(pool)
                .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
            total_records_affected += res.rows_affected();

    
    let sql = r#"update rec.names n
                set der_lang = 'en'
                where der_lang is null and n.name_type <> 10
        and (n.lc_value like '%observatory%'
        or n.lc_value like '%observatories%')
        and n.lc_value not like '%ПМФ%'
    "#;

    let res = sqlx::raw_sql(sql).execute(pool)
    .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
   
    let sql = r#"update rec.names n
                set der_lang = 'en'
                where der_lang is null and n.name_type <> 10
    and lc_value like '%network%'  
    and lc_value not like '%researcherenye%'"#;
    
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to english names", total_records_affected);

    Ok(())

    // institite and centre??? - soplit between anglophone and francophone...
}


pub async fn update_japanese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'ja'
            where der_lang is null and n.name_type <> 10
            and country_code = 'JP'
            and 
            (lc_value like '%daigaku%'
            or lc_value like '%daigakkō%'
            or lc_value like '%kabushiki%'
            or lc_value like '%nippon%' 
            or lc_value like '%kaihatsu%' 
            or lc_value like '%bijutsukan%');"#;   
            
            // university
            // college
            // corporation
            // Japan
            // development
            // art museum
                        
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'ja'
            
            
            where der_lang is null and n.name_type <> 10
            and country_code = 'JP'
            and 
            (lc_value like '%kenritsu%' 
            or lc_value like '%dokuritsu%'  
            or lc_value like '% kikō%'
            or lc_value like '%gakkō%'
            or lc_value like '%gakko%'
            or lc_value like '%gakkou%'
            or lc_value like '%kaihatsu%'
            or lc_value like '%-shō%'
            or lc_value like '%bunka senta%' 
            or lc_value like '%denryoku%'  
            or lc_value like '%gakuen%'
            or lc_value like '%kagaku-kan%'
            or lc_value like '%bungaku-kan%'
            or lc_value like '%-chō%');"#;

            // prefectural
            // independent
            // organization
            // school (3)
            // development
            // -prize
            // cultural center
            // electric power
            // academy
            // science building
            // literature building
            // district
            // specialized school

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'ja'
            where der_lang is null and n.name_type <> 10
            and country_code = 'JP'
            and (lc_value like '%chuobyoin%'
            or lc_value like '%shiritsu%'  
            or lc_value like '%kenkyūjo%'
            or lc_value like '%kenkyujo%'
            or lc_value like '%kenkyūsho%'
            or lc_value like '%kenkei%'
            or lc_value like '%kyōdō%');"#;
            
            // medical center
            // municipal
            // research institute (3)
            // survey
            // collaboration
            
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'ja'
        where der_lang is null and n.name_type <> 10
        and country_code = 'JP'
        and (lc_value like '%tankyu%'
        or lc_value like '%kenkyusho%'
        or lc_value like '%kenkyuu%'
        or lc_value like '%kokusai%'
        or lc_value like '%hakubutsukan%'
        or lc_value like '%toshoken%'
        or lc_value like '%byoin%'
        or lc_value like '%byōin%');"#;
        
        // research facility
        // research laboratory
        // research
        // international
        // museums
        // libraries
        // hospitals (2)
        
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'ja'
        where der_lang is null and n.name_type <> 10
        and country_code = 'JP'
        and (lc_value like '%nihon%' 
        or lc_value like '%kinzoku%'  
        or lc_value like '%kenkyū%'
        or lc_value like '%kokudo%'
        or lc_value like '%jitsugyo%'
        or lc_value like '%fukusei%' 
        or lc_value like '%shiryokan%'  
        or lc_value like '%gurūpu%'
        or lc_value like 'shiritsuchuobyoin%'
        or lc_value like '%kenkyuukikou%'
        or lc_value like '%shiminbyoin%');"#;

        // Japan
        // metal
        // research
        // national land
        // practical business
        // integrated
        // information center
        // group
        // municipal hospital
        // research organization
        // high school for advanced study
        // municipal hospital

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to japanese records", total_records_affected);

    Ok(())

}


pub async fn update_chinese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'zh'
                where der_lang is null and n.name_type <> 10
                and country_code in ('CN', 'TW', 'HK')
                and (lc_value like '%dàxué%'
                or lc_value like '%daxue%'
                or lc_value like '%dàxúe%'
                or lc_value like '%zhōngyī%'
                or lc_value like '%xuéyuàn%'
                or lc_value like '%yīyuàn%'
                or lc_value like '%jīgòu%'
                or lc_value like '%yánjiū%'
                or lc_value like '%mínguó%'
                or lc_value like '%yínháng%');"#;
                 

        // dàxué, dàxúe, daxue   University
        // zhōngyī     (traditional) Chinese medicine
        // xuéyuàn     Educational institute (school - conservatory - academy)
        // yīyuàn      hospital
        // jīgòu       Mechanism (body - agency)
        // yánjiū      Study (Research)
        // mínguó      Republic
        // yínháng     Bank

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'zh'
                where der_lang is null and n.name_type <> 10
                and country_code in ('CN', 'TW', 'HK')
                and (lc_value like '%yīyún%'
                or lc_value like '%yánjiùyuàn%'
                or lc_value like '%ybówùguǎn%'
                or lc_value like '%xuéxiào%'
                or lc_value like '%shénxué%'
                or lc_value like '%gōngyè%'
                or lc_value like '%zhèngfǔ%'
                or lc_value like '%guójiā%' 
                or lc_value like '%shīfàn%' 
                );"#;
                 

        // yīyún      hospital
        // yánjiùyuàn researcher
        // bówùguǎn   museum
        // xuéxiào    school
        // shénxué    theology
        // gōngyè     industry
        // zhèngfǔ    government
        // guójiā     state, country
        // shīfàn     school

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to chinese records", total_records_affected);

    Ok(())
}


pub async fn update_french_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    // initial group seen as French whatever the country
    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and 
            (lc_value like '%école%'
            or lc_value like '%université%' 
            or lc_value like '%laboratoire%' 
            or lc_value like '%réseau%' 
            or lc_value like '%société%'
            or lc_value like '%santé%'
            or lc_value like '%publique%'
            or lc_value like '%mondiale%' 
            or lc_value like '%équipe%' 
            or lc_value like '%maison%'
            or lc_value like '%bibliothèque%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (lc_value like '%agence%'
            or lc_value like '%académie%'
            or lc_value like '%ecole%'
            or lc_value like '%environnement%'
            or lc_value like '%linstitut%'
            or lc_value like '%système%'
            or lc_value like '% et %');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (lc_value like '%canadienne%'
            or lc_value like '%banque%' 
            or lc_value like '%gouvernement%'
            or lc_value like '%informatique%' 
            or lc_value like '%unité%' 
            or lc_value like '%français%' 
            or lc_value like '%recherche%'
            or lc_value like '%développement%'
            or lc_value like '%biologie%'
            or lc_value like '%génétique%' 
            or lc_value like '%observatoire%' 
            or lc_value like '%centre de%'
            or lc_value like '%centre universitaire%'
            or lc_value like 'centre %'
            or lc_value like '%fédération%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (lc_value ilike '%hôpital%'
            or lc_value like '%musée%' 
            or lc_value like '%pôle%'
            or lc_value like '%études%'
            or lc_value like '%chimie%' 
            or lc_value like '%clinique%' 
            or lc_value like '%conseil%' 
            or lc_value like '%département%'
            or lc_value like '%faculté%'
            or lc_value like '%fondation%'
            or lc_value like '%ministère%' 
            or lc_value like '%plateforme%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (lc_value like '%collège%'
            or lc_value like '%communauté%' 
            or lc_value like '%espace%'
            or lc_value like '%archives%'
            or lc_value like '%centrale%' 
            or lc_value like '%château%' 
            or lc_value like '%délégation%' 
            or lc_value like '%génomique%'
            or lc_value like '%hôtel-dieu%'
            or lc_value like '%européenne%'
            or lc_value like '%ambassade%' 
            or lc_value like '%plateforme%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (value ilike 'institut de%' 
            or value ilike 'institut dhistoire%'
            or value ilike 'institut du %'
            or lc_value like '%chambre%' 
            or lc_value like '%comité%' 
            or lc_value like '%caisse%' 
            or lc_value like '%médecin%'
            or lc_value like '%ministre%'
            or lc_value like '%régie%' 
            or lc_value like '%région%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (value ilike 'Inserm %'
            or value like 'CH %' 
            or value like 'CHU %'
            or value like 'CIC %'
            or value like 'EA%' 
            or value like 'ERL %' 
            or value like 'GDR%' 
            or value like 'U %'
            or value like 'UAR%'
            or value like 'UMR%'
            or value like 'UMRS %' 
            or value like 'UMR_S %' 
            or value like 'UMS %'
            or value like 'UR%'
            or value like 'URP %'
            or value like 'US%');"#;

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

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'BE', 'CA', 'CG', 'LU', 'CM', 'MA', 'ML' , 'SN', 'DZ', 'PF', 'CH', 
                                 'RE', 'RW', 'MQ', 'YT', 'TN', 'CI', 'BI', 'NC', 'MU')
            and 
            (value like 'BRGM %'
            or value like 'CEA %' 
            or value like 'Cégep %'
            or value like 'CHR %'
            or value like 'CHP %' 
            or value like 'CISSS %' 
            or value like 'CIUSSS %'
            or value like 'CNRS %'
            or value like 'CRP %'
            or value like 'ESC %' 
            or value like 'ESIEE %' 
            or value like 'ESPI  %'
            or value like 'GRC %'
            or value like 'HES-SO %'
            or value like 'IMT %'
            or value like 'INSA %'
            or value like 'IUT %'
            or value ilike 'Labex %'
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to french records", total_records_affected);
    
    Ok(())

}

 
pub async fn update_german_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'de'
            where der_lang is null and n.name_type <> 10
            and country_code in ('DE', 'AT', 'CH')
            and 
            (lc_value ilike '% für %'
            or lc_value like '% und %' 
            or lc_value like '%klinische%'
            or lc_value like '%klinik%'
            or lc_value like '%bundesamt%' 
            or lc_value like '%hochschule%' 
            or lc_value like '%fraunhofer%' 
            or lc_value like '%zentrum%'
            or lc_value like '%akademie%'
            or lc_value like '%allgemeine%'
            or lc_value like '%deutsche%' 
            or lc_value like '%gesellschaft%' 
            or lc_value like 'krankenhaus%'
            or lc_value like '%wissenschaft%'
            or lc_value like '%arbeit%'
            or lc_value like '%gemeinschaft%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'de'
            where der_lang is null and n.name_type <> 10
            and country_code in ('DE', 'AT', 'CH')
            and 
            (lc_value ilike '%bundesverband%'
            or lc_value like '%europäische%' 
            or lc_value like '%forschung%'
            or lc_value like '%kantonsschule%'
            or lc_value like '%kantonsspital%' 
            or lc_value like '%katholische%' 
            or lc_value like '%österreichische%' 
            or lc_value like '%schweizerische%'
            or lc_value like '%stiftung%'
            or lc_value like '%technische%'
            or lc_value like '%universitaet%' 
            or lc_value like '%universität%' 
            or lc_value like '%vereinigung%'
            or lc_value like '%wasser%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    /* 
    let sql = r#"update rec.names n
                set der_lang = 'de'
            where der_lang is null and n.name_type <> 10
            and country_code in ('DE', 'AT', 'CH')
            and 
            (lc_value ilike '%%'
            or lc_value like '%%' 
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%' 
            or lc_value like '%%' 
            or lc_value like '%%' 
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%' 
            or lc_value like '%%' 
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    */
    info!("{} language codes added to german records", total_records_affected);
    
    Ok(())
}


pub async fn update_spanish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
            where der_lang is null and n.name_type <> 10
            and country_code in ('AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'GQ', 'GT', 
                                 'HN', 'MX', 'NI', 'PE', 'PY', 'UY', 'VE', 'PR')
            and 
            (lc_value ilike '%academia%'
            or lc_value like '%unidad%' 
            or lc_value like '%universitari%'
            or lc_value like '%universidad%'
            or lc_value like '%agencia%' 
            or lc_value like '%asociación%'
            or lc_value like '%ayuntamiento%'
            or lc_value like '%banco%' 
            or lc_value like '%benemérita%' 
            or lc_value like '%biblioteca%' 
            or lc_value like '%centro%'
            or lc_value like '%ciencia%'
            or lc_value like '%científico%'
            or lc_value like 'clínica%' 
            or lc_value like '%clínico%' 
            or lc_value like '%colegio%'
            or lc_value like '%comisión%'
            or lc_value like '%consejo%'
            or lc_value like '%consorcio%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

     let sql = r#"update rec.names n
                 set der_lang = 'es'
             where der_lang is null and n.name_type <> 10
             and country_code in ('AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'GQ', 'GT', 
                                  'HN', 'MX', 'NI', 'PE', 'PY', 'UY', 'VE', 'PR')
             and 
             (lc_value ilike '%corporación%'
             or lc_value like '%departamento%' 
             or lc_value like '%dirección%'
             or lc_value like '%escuela%'
             or lc_value like '%esperança%' 
             or lc_value like '%española%' 
             or lc_value like '%estación%' 
             or lc_value like '%facultad%'
             or lc_value like '%fundacion%'
             or lc_value like '%gobierno%'
             or lc_value like '%grupo%' 
             or lc_value like '%institución%'
             or lc_value like '%instituto%'
             or lc_value like '%laboratorio%');"#;
     let res = sqlx::raw_sql(sql).execute(pool)
         .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
     total_records_affected += res.rows_affected();

     let sql = r#"update rec.names n
                 set der_lang = 'es'
             where der_lang is null and n.name_type <> 10
             and country_code in ('AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 'ES', 'GQ', 'GT', 
                                  'HN', 'MX', 'NI', 'PE', 'PY', 'UY', 'VE', 'PR')
             and 
             (lc_value ilike '%médico%'
             or lc_value like '%milenium%' 
             or lc_value like '%ministerio%'
             or lc_value like '%museo%'
             or lc_value like '%nacional%' 
             or lc_value like '%observatorio%' 
             or lc_value like '%organización%' 
             or lc_value like '%parque%'
             or lc_value like '%pontificia%'
             or lc_value like '%salud%'
             or lc_value like '%sanitas%' 
             or lc_value like '%secretaría%' 
             or lc_value like '%servicio%' 
             or lc_value like '%sistema%'
             or lc_value like '%sociedad%'
             or lc_value like '%tecnológico%'
             or lc_value like '%tecnm%');"#;
     let res = sqlx::raw_sql(sql).execute(pool)
         .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
     total_records_affected += res.rows_affected();

     
    info!("{} language codes added to spanish records", total_records_affected);
    
    Ok(())
}

 
pub async fn update_portuguese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
     
    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'pt'
            where der_lang is null and n.name_type <> 10
            and country_code in ('PT', 'BR', 'CV', 'AO', 'MZ', 'GW', 'ST', 'TL')
            and 
            (lc_value ilike '%agência'
            or lc_value like '%associação%' 
            or lc_value like '%autoridade%'
            or lc_value like '%biblioteca%'
            or lc_value like '%comissão%' 
            or lc_value like '%ciência%' 
            or lc_value like '%conselho%' 
            or lc_value like '%departamento%'
            or lc_value like '%direção%'
            or lc_value like '%escola%'
            or lc_value like '%estudos%' 
            or lc_value like '%faculdade%' 
            or lc_value like '%federação%'
            or lc_value like '%fundação%'
            or lc_value like '%gabinete%'
            or lc_value like '%grupo%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'pt'
            where der_lang is null and n.name_type <> 10
            and country_code in ('PT', 'BR', 'CV', 'AO', 'MZ', 'GW', 'ST', 'TL')
            and 
            (lc_value like '%investigação%'
            or lc_value like '%instituto%'
            or lc_value like '%laboratório%' 
            or lc_value like '%ministério%' 
            or lc_value like '%museu%' 
            or lc_value like '%observatório%'
            or lc_value like '%ordem%'
            or lc_value like '%parque%'
            or lc_value like '%pesquisa%' 
            or lc_value like '%sociedade%' 
            or lc_value like '%tecnologia%'
            or lc_value like '%tecnológico%'
            or lc_value like '%unidade%'
            or lc_value like '%universitário%'
            or lc_value like '%universidade%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'pt'
            where der_lang is null and n.name_type <> 10
            and country_code in ('PT', 'BR', 'CV', 'AO', 'MZ', 'GW', 'ST', 'TL')
            and 
            (value ilike 'INCT de%' 
             or lc_value like  '%centro%'
             or lc_value like  '%nacional%'   
             or lc_value like  '%esperança%' 
             or lc_value like  '%ciencia%' 
             or lc_value like  '%academia%'
             or lc_value like  '%secretaria%' 
             or lc_value like  '%governo%'
             or lc_value like  '%prefeitura%'   
             or lc_value like  '%companhia%' 
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to portuguese records", total_records_affected);
    
    Ok(())
}


pub async fn update_italian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;
       
    let sql = r#"update rec.names n
                set der_lang = 'it'
            where der_lang is null and n.name_type <> 10
            and country_code in ('IT', 'CH')
            and 
            (lc_value ilike '%accademia%'
            or lc_value like '%agenzia%' 
            or lc_value like '%archivio%'
            or lc_value like '%associazione%'
            or lc_value like '%azienda%' 
            or lc_value like '%centro di %' 
            or lc_value like '%conservatorio%' 
            or lc_value like '%consorzio%'
            or lc_value like '%dipartimento%'
            or lc_value like '%federazione%'
            or lc_value like '%fondazione%' 
            or lc_value like '%gruppo%' 
            or lc_value like '%istituto%'
            or lc_value like '%liceo%'
            or lc_value like '%ministero%'
            or lc_value like '%museo%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'it'
            where der_lang is null and n.name_type <> 10
            and country_code in ('IT', 'CH')
            and 
            (lc_value ilike '%organizzazione%'
            or lc_value like '%ospedale%' 
            or lc_value like '%osservatorio%'
            or lc_value like '%pontificia%'
            or lc_value like '%regione%' 
            or lc_value like '%scuola%' 
            or lc_value like '%sistema%' 
            or lc_value like '%societa%'
            or lc_value like '%ufficio%'
            or lc_value like '%università%'
            or lc_value like '%unità%' 
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    /* 
    let sql = r#"update rec.names n
                set der_lang = 'it'
            where der_lang is null and n.name_type <> 10
            and country_code in ('IT', 'CH')
            and 
            (lc_value ilike '%%'
            or lc_value like '%%' 
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%' 
            or lc_value like '%%' 
            or lc_value like '%%' 
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%' 
            or lc_value like '%%' 
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%'
            or lc_value like '%%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    */

    info!("{} language codes added to italian records", total_records_affected);
    
    Ok(())
}


pub async fn update_dutch_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
 
    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'nl'
            where der_lang is null and n.name_type <> 10
            and country_code in ('NL', 'BE')
            and 
            (lc_value ilike '%academisch%'
            or lc_value like '%gemeentelijke%' 
            or lc_value like '%gezondheidsdienst%'
            or lc_value like '%koninklijke%'
            or lc_value like '% voor %' 
            or lc_value like '%ziekenhuis%' 
            or lc_value like '%ziekenhuizen%' 
            or lc_value like '%ministerie%' 
            or lc_value like '%nationaal%'
            or lc_value like '%nederlandse%'
            or lc_value like '%instituut%'
            or lc_value like '%stichting%' 
            or lc_value like '%universiteit%' 
            or lc_value like '%vereniging%'
            or lc_value like '%zorg%'
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'nl'
            where der_lang is null and n.name_type <> 10
            and country_code in ('NL', 'BE')
            and 
            (lc_value ilike '%kliniek%'
            or lc_value like '%medisch%'
            or lc_value like '%meenschap%'
            or lc_value like '%centrum%' 
            or lc_value like '%groep%'
            or lc_value like '%nederlandsche%'
            or lc_value like '%specialisten%' 
            or lc_value like '%fonds%' 
            or lc_value like '%gemeente%' 
            or lc_value like '%kundige%' 
            or lc_value like '%hogeschool%'
            or lc_value like '%huisarts%'
            or lc_value like '%maatschap%'
            or lc_value like '%gasthuis%' 
            or lc_value like '%gezondheid%' 
            or lc_value like '%groot%'
            or lc_value like '%stedelijk%'
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
  
    info!("{} language codes added to dutch records", total_records_affected);
    
    Ok(())
}


pub async fn update_danish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {
 
    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'da'
            where der_lang is null and n.name_type <> 10
            and country_code in ('DK')
            and 
            (lc_value ilike '% fonde%'
            or lc_value like '%fonden%' 
            or lc_value like '%fondet%' 
            or lc_value like '%kommune%'
            or lc_value like '%sygehus%'
            or lc_value like '%dansk%' 
            or lc_value like '%foreningen%' 
            or lc_value like '%danmarks%' 
            or lc_value like '%klinik%'
            or lc_value like '% og %'
            or lc_value like '%skole%'
            or lc_value like '%regionshospita%' 
            or lc_value like '%rigshospitalet%' 
            or lc_value like '%universitet%'
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'da'
            where der_lang is null and n.name_type <> 10
            and country_code in ('DK')
            and 
            (lc_value ilike '%teknolog%'
            or lc_value like '%arkiv%' 
            or lc_value like '%hospitaler%' 
            or lc_value like '%tekniske%'
            or lc_value like '%privathospital%'
            or lc_value like '%midt%' 
            or lc_value like '%nordvest%' 
            or lc_value like '%biblioteket%' 
            or lc_value like '%gigthospital%'
            or lc_value like '%hospitalsenheden%'
            or lc_value like '%styrelsen%'
            or lc_value like '%nationalbanken%' 
            or lc_value like '%kræftens%' 
            or lc_value like '%vaern%'
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    info!("{} language codes added to danish records", total_records_affected);
    
    Ok(())
}

 
pub async fn update_swedish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    
/*
 -- ALSO
 --Aleris Hälsocentral Bollnäs, --Avfallshantering Östra Skaraborg
 --Centrallasarettet Växjö, --Göteborgs Tandläkare Sällskap
 --Hälsans Nya Verktyg, --Industriella UtvecklingsCentra
 --Länsstyrelsen Västra Götalands län, --Praktikertjänst
 --Rinkebyakademien, --Skandinaviska Kiropraktorhögskolan
 --Skånes Livsmedelsakademi, --Specialpedagogiska Skolmyndigheten
 --TransportForsK, --Tunga Fordon, 
 --Wienerbageriet
*/

    
    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'sv'
            where der_lang is null and n.name_type <> 10
            and country_code in ('SE', 'FI')
            and 
            (lc_value ilike '%konstmuseum%'
            or lc_value like '%institutet%' 
            or lc_value like '%ningen%'
            or lc_value like '%huset%' 
            or lc_value like '%ringen%'
            or lc_value like '%universitetet%'
            or lc_value like '%förbundet%' 
            or lc_value like '%telsern%' 
            or lc_value like '%landsting%'
            or lc_value like '%lasarett%'
            or lc_value like '%minnesfond%'
            or lc_value like '%stiftelse%' 
            or lc_value like '%kliniken%' 
            or lc_value like '%fonden%'
            or lc_value like '%forskning%'
            or lc_value like '%centrum%'
            or lc_value like '%vägen%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'sv'
            where der_lang is null and n.name_type <> 10
            and country_code in ('SE', 'FI')
            and 
            (lc_value ilike '% för %'
            or lc_value like '%sjukhus%' 
            or lc_value like '%västra%'
            or lc_value like '%akademin%' 
            or lc_value like '%finlands%'
            or lc_value like '%finska%'
            or lc_value like '%folktandvården%' 
            or lc_value like '%göteborgs%' 
            or lc_value like '%högskolan%'
            or lc_value like '% i %'
            or lc_value like '%kungliga%'
            or lc_value like '%landstinget%' 
            or lc_value like '%länsstyrelsen%' 
            or lc_value like '%stiftelsen%'
            or lc_value like '%svenska%'
            or lc_value like '%sverige%'
            or lc_value like '%trafikverket%'
            or lc_value like '%kommun%' );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to swedish records", total_records_affected);
    
    Ok(())
}


pub async fn update_finnish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    
    /*
     -- ALSO
     --Ab Lukko Oy, --Amer-Yhtymä, --Keski-Satakunnan Terveydenhuollon kuntayhtymä
     --Lounais-Suomen Syöpäyhdistys, --Osakeyhtiö Kone Aktiebolag
     --Pellervo Taloustutkimus, --Puolustusvoimat, --Satakunnan Sairaanhoitopiirin
     --Suomalainen Tiedeakatemia, --Tekonivelsairaala Coxa, --Trafikverket
     --Työsuojelurahasto, --Ymparistoministerio Milijoministeriet
     */
     
    let mut total_records_affected = 0;
    
    let sql = r#"update rec.names n
                set der_lang = 'fi'
            where der_lang is null and n.name_type <> 10
            and country_code in ('SE', 'FI')
            and 
            (lc_value ilike '%yliopisto%'
            or lc_value like '%säätiö%' 
            or lc_value like '%suomi%'
            or lc_value like '%etelä%'
            or lc_value like '%helsingin%' 
            or lc_value like '%juhani%' 
            or lc_value like '%kansainvälisen%'
            or lc_value like '%instituutti%'
            or lc_value like '%korkeakoulu%'
            or lc_value like '%lääketieteellisen%' 
            or lc_value like '% ja %' 
            or lc_value like '%norjan%'
            or lc_value like '%pohjois%'
            or lc_value like '%ruotsin%'
            or lc_value like 'satakunnan%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'fi'
            where der_lang is null and n.name_type <> 10
            and country_code in ('SE', 'FI')
            and 
            (lc_value ilike '%suomalainen%'
            or lc_value like '%suomen%' 
            or lc_value like '%tampereen%'
            or lc_value like '%turun%'
            or lc_value like '%vaasan%' 
            or lc_value like '%yhteis%' 
            or lc_value like '%sairaala%' 
            or lc_value like '%liitto%'
            or lc_value like '% och %'
            or lc_value like '%laitos%' 
            or lc_value like '%musseura%' 
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update rec.names n
                set der_lang = 'fi'
            where der_lang is null and n.name_type <> 10
            and country_code in ('SE', 'FI')
            and 
            (lc_value like '%institutet%'  
            or lc_value like '%akademi%'
            or lc_value like '%säätiö%'
            or lc_value like '%sjukhus%'
            or lc_value like '%forsknings%'
            or lc_value like '%stiftelsen%'
            or lc_value like '%centrum%'
            or lc_value like '%keskus%'
            or lc_value like '%centralen%'
            or lc_value like '%räjät%'
            or lc_value like '%topiiri%'
            or lc_value like '%korkeakoul%'
            );"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    info!("{} language codes added to finnish records", total_records_affected);
    
    Ok(())
}


pub async fn update_norwegian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'no'
            where der_lang is null and n.name_type <> 10
            and country_code = 'NO'
            and (lc_value like '%ø%' 
            or lc_value like '%sykehus%' 
            or lc_value like '%skole%' 
            or lc_value like '%skule%' 
            or lc_value like '%universitet%' 
            or lc_value like '% i %'
            or lc_value like '%ø%'
            or lc_value like '%direktoratet%'
            or lc_value like '%registeret%'
            or lc_value like '%kommun%'
            or lc_value like '%instituut%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'no'
            where der_lang is null and n.name_type <> 10
            and country_code = 'NO'
            and (lc_value like '%kunnskaps%' 
            or lc_value like '%bibliotek%' 
            or lc_value like '%musea%' 
            or lc_value like '%havn%' 
            or lc_value like '%regionen%'
            or lc_value like '%teknolog%'
            or lc_value like '%svaret%'
            or lc_value like '%klinikken%'
            or lc_value like '%dyrehospital%'
            or lc_value like '%sverk%'
            or lc_value like '%sijte%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'no'
            where der_lang is null and n.name_type <> 10
            and country_code = 'NO'
            and (lc_value like '%norge%' 
            or lc_value like '%det %' 
            or lc_value like '%forskning%' 
            or lc_value like '%institutt%' 
            or lc_value like '%heise %'
            or lc_value like '%senter%'
            or lc_value like '%forening%'
            or lc_value like '%kunnskaps%'
            or lc_value like '%råd%'
            or lc_value like '%departementet%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    
    info!("{} language codes added to norwegian records", total_records_affected);
    
    Ok(())
}


pub async fn update_indian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'en'
            where der_lang is null and n.name_type <> 10
            and country_code = 'IN'
            and 
            (value like 'AIIMS%'
            or value like 'GCE%'
            or value like 'GMC%'
            or value like 'IIIT%'
            or value like 'IIM%'
            or value like 'IISER%'
            or value like 'IIT%' 
            or value like 'NIPER%'
            or value like 'NIT%'
            or value like 'RDC%'
            or value like 'REC%'
            or value like 'SKUAST%'
            or value like 'JNT%'
            or value like '%centre%'
);"#;

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

    let res = sqlx::raw_sql(sql).execute(pool)
       .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'hi'
    where der_lang is null and n.name_type <> 10
    and country_code = 'IN'
    and 
    (value like 'KVK %'
    or value like 'GCE%'
    or lc_value like '% vigyan%'
    or lc_value like '% vishwavidyalaya%'
    or lc_value like '% sanstha%'
    or lc_value like '% sansthā%'
    or lc_value like '% vidyālaya%'
    or lc_value like '%krishi%'
    or lc_value like '%samsthana%');"#;

       // KVK     Krishi Vigyan Kendra  Farm Science Center
       // vigyan           science
       // vishwavidyalaya  university school
       // sanstha          organization
       // sansthā
       // vidyālaya        school
       // krishi           agriculture
       // samsthana        institution
       
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to indian records", total_records_affected);
    
    Ok(())
}


pub async fn update_iranian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'fa'
            where der_lang is null and n.name_type <> 10
            and country_code = 'IR'
            and lc_value like '%dāneshgāh%';"#;

        // dāneshgāh    university

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to iranian records", total_records_affected);
    
    Ok(())
}


pub async fn update_russian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;
    

    let sql = r#"update rec.names n
                set der_lang = 'ru'
            where der_lang is null and n.name_type <> 10
            and country_code = 'RU'
            and (lc_value like '%institut %'
            or lc_value like '%universitet%'
            or lc_value like '%akademiya%'
            or lc_value like '%akadémiya%'
            or lc_value like '%oblastnoy%'
            or value like 'JSC %');"#;

            // JSC  Scientific research institute

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'ru'
            where der_lang is null and n.name_type <> 10
            and country_code = 'RU'
            and (lc_value like '%federalnyy%'
            or lc_value like '%patologii%'
            or lc_value like '%khirurgii%'
            or lc_value like '%shkola%'
            or lc_value like '%kombinat%'
            or lc_value like '%tsentr%');"#;

     let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to russian records", total_records_affected);
    
    Ok(())
}


pub async fn update_ukrainian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;
   
    let sql = r#"update rec.names n
                set der_lang = 'uk'
            where der_lang is null and n.name_type <> 10
            and country_code = 'UA'
            and (lc_value like '%universitét %'
            or lc_value like '%universytet%'
            or lc_value like '%ukrainsky%'
            or lc_value like '%ukrayinska%'
            or lc_value like '%ukrayiny%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to ukranian records", total_records_affected);
    
    Ok(())
}


pub async fn update_serbian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'sr'
            where der_lang is null and n.name_type <> 10
            and country_code = 'RS'
            and (lc_value like '%institut%' 
            or lc_value like '%univerzitet%' 
            or lc_value like '%zvezdara%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to serbian records", total_records_affected);
    
    Ok(())
}


pub async fn update_bulgarian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'bg'
            where der_lang is null and n.name_type <> 10
            and country_code = 'BG'
            and (lc_value like '%institut%' 
            or lc_value like '%akademiya%' 
            or lc_value like '%universitet%'
            or lc_value like '%ministerstvo%' 
            or lc_value like '%obshtina%'
            or lc_value like '%muzei%'
            or lc_value like '%medicinska%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to bulgarian records", total_records_affected);
    
    Ok(())
}


pub async fn update_israeli_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'he'
            where der_lang is null and n.name_type <> 10
            and country_code = 'IL'
            and (lc_value like '%ha-universita%' 
            or lc_value like '%hauniversita%' 
            or lc_value like '%machon %'
            or lc_value like '%merkaz %' 
            or lc_value like '%misrad %'
            or lc_value like '%misgav %'
            or lc_value like '%mikhlelet%'
            or lc_value like '%miklelet%');"#;

            // machon   institution or foundation
            // merkaz   centre
            // misrad   office
            // misgav   refuge (hospital here)
            // mikhlelet college
            // miklelet  (law) school
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to israeli records", total_records_affected);
    
    Ok(())
}


pub async fn update_korean_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'ko'
            where der_lang is null and n.name_type <> 10
            and country_code = 'KR'
            and (lc_value like '%daehak%' 
            or lc_value like '%hakkyo%'
            or lc_value like '%taehak%');"#;

            // daehak  
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to korean records", total_records_affected);
    
    Ok(())
}


pub async fn update_greek_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'el'
            where der_lang is null and n.name_type <> 10
            and country_code = 'GR'
            and (lc_value like 'tei %');"#;

            // tei     Technological Educational Institute
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update rec.names n
                set der_lang = 'el'
            where der_lang is null and n.name_type <> 10
            and country_code = 'GR'
            and (lc_value like '%panepistimio%'
            or lc_value like '%panepistimiako%'
            or lc_value like '%ellinikon%'
            or lc_value like '%institouto%'
            );"#;

            // panepistimio    university
            // panepistimiako  university
            // ellinikon       greek
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    info!("{} language codes added to greek records", total_records_affected);
    
    Ok(())
}
*/

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
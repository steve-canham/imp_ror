use sqlx::{Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn assign_lang(names: Vec<&str>, lang_code: &str, countries: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let name_crit = if names == vec!["the rest"] {
        "".to_string()
    }
    else
    {
        let mut word_list = "".to_string();
        for i in 0..names.len() {
            let comparator = if names[i].starts_with("^") { 
                format!("lang_name like '{}%'", &names[i][1..]) 
            }
            else if names[i].ends_with("$") { 
                format!("lang_name like '%{}'", &names[i][..names[i].len() - 1]) 
            }
            else { 
                format!("lang_name like '%{}%'", names[i]) 
            };
            let comparison = format!(" {}{comparator}", if i > 0 {"or "} else {""});
            word_list += comparison.as_str();
        }
        format!("and ({word_list})")
    };

    let sql = if countries == "" {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null 
                {name_crit};"#)
    } 
    else {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null 
                and country_code in ({countries})
                {name_crit};"#)
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
        else if names[i].ends_with("$") { 
            format!("display_name like '%{}'", &names[i][..names[i].len() - 1]) 
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
                where der_lang is null 
                and ({word_list});"#)
    } 
    else {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null 
                and country_code in ({countries})
                and ({word_list});"#)
    };
    
    let res = sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql))?;
    Ok(res.rows_affected())
}


pub async fn assign_lang_using_acronym(names: Vec<&str>, lang_code: &str, countries: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let mut word_list = "".to_string();
    for i in 0..names.len() {
        let comparator =  format!("display_name = '{}'", names[i]);
        let comparison = format!(" {}{comparator}", if i > 0 {"or "} else {""});
        word_list += comparison.as_str();
    }

    let sql = if countries == "" {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null 
                and ({word_list});"#)
    } 
    else {
        format!(r#"update rec.names
                set der_lang = '{lang_code}'
                where der_lang is null 
                and country_code in ({countries})
                and ({word_list});"#)
    };
    
    let res = sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql))?;
    Ok(res.rows_affected())
}


pub async fn update_greenland_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    // NB Order important
    
    records_affected += assign_lang(vec!["perorsaanermik ilinniarfik college of social education"], 
        "ki, en", "'GL'", pool).await?;
    records_affected += assign_lang(vec!["eriffik", "tusarfik", "simmavissua","nunatsinni", "nunatta"], 
        "ki", "'GL'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["NKA"], "ki", "'GL'", pool).await?;
    records_affected += assign_lang(vec!["grønlands", "dronning"], "da", "'GL'", pool).await?;
    records_affected += assign_lang(vec!["the rest"], "en", "'GL'", pool).await?;
     
    info!("{} language codes added to Greenland records", records_affected);
    Ok(())
}


pub async fn update_faroe_island_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;
   
    records_affected += assign_lang(vec!["marbejde", "færøernes"], "da", "'FO'", pool).await?;
    records_affected += assign_lang(vec!["landsbókasavnið - national library of the faroe islands"], 
        "fo, en", "'FO'", pool).await?;
    records_affected += assign_lang(vec!["føroya", "savnið", "starfið", "tjóðsavn", "stovan", "samstarv", 
        "garráðið"], "fo", "'FO'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["US"], "fo", "'FO'", pool).await?;
    records_affected += assign_lang(vec!["avannaani"], "ki", "'FO'", pool).await?;
    records_affected += assign_lang(vec!["the rest"], "en", "'FO'", pool).await?;
     
    info!("{} language codes added to Faroe Island records", records_affected);

    //  fróðskaparsetur føroya, granskingarráðið
    // "tjóðsavn føroya"
    // "tjóðsavnið"
    
    Ok(())
}


pub async fn update_iceland_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["hochschule"], "de", "'IS'", pool).await?;
    records_affected += assign_lang(vec!["ð", "stofnun", "kóli", "spítali", "veit", "hjart", 
        "skógur", "bók", "sók", "læknis", "stofa", "knisetur", "félag", "virkjun"], 
        "is", "'IS'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["PFS", "SAk", "LFÍ", "LOGS", "ÍSOR", 
        "PSSÍ", "RANNÍS", "LMFÍ", "FS", "RMFS"], "is", "'IS'", pool).await?;
    records_affected += assign_lang(vec!["icetec", "matís", "össur", "origo", "kerecis", 
        "star-oddi", "oculis", "decode", "marel", "prokazyme"], "bd", "'IS'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["GRO"], "bd", "'IS'", pool).await?;
    records_affected += assign_lang(vec!["the rest"], "en", "'IS'", pool).await?;

    // ehf and hf after Icelandic company names indicate (private ehf) limited company (?hf)
     
    info!("{} language codes added to Iceland records", records_affected);
    Ok(())
}


pub async fn update_malta_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["primo ministro"], "it", "'MT'", pool).await?;
    records_affected += assign_lang(vec![" ta ", "fondazzjoni", "isptar", "xjenza"], 
        "mt", "'MT'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["FTZ"], "mt", "'MT'", pool).await?;
    records_affected += assign_lang(vec!["triagon$", "visuray", "paragon", "ateknea", 
        "stmicroelectronics", "acrosslimits", "aquabiotech"], "bd", "'MT'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["IDEA", "ABT"], "bd", "'MT'", pool).await?;
    records_affected += assign_lang(vec!["the rest"], "en", "'MT'", pool).await?;

    // ehf and hf after Icelandic company names indicate (private ehf) limited company (?hf)
     
    info!("{} language codes added to Malta records", records_affected);
    Ok(())
}


pub async fn update_cyprus_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["университет"], "ru", "'CY'", pool).await?;
    records_affected += assign_lang(vec!["üniversitesi", "universitesi", "akademisi", "yerleşkesi̇", 
        "kıbrıs", "kibris", "istatistiki"], 
        "tr", "'CY'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["AÖA", "KIBÜ", "DAU", "RDÜ", "YDÜ"], 
        "tr", "'CY'", pool).await?;
    records_affected += assign_lang_using_display_name(vec!["GAU$"], "en, tr", "'CY'", pool).await?;

    records_affected += assign_lang(vec!["engitech", "nipd genetics", "medicover", "axon neuroscience",
        "primetel", "medsonic", "sigint", "interfusion", "amdocs", "ebos technologies", "intelen"],
        "bd", "'CY'", pool).await?;
    records_affected += assign_lang(vec!["rominster", "signalgenerix", "rinnoco", "danaos", "lumoscribe", 
        "archimedes int", "geoimaging", "rtd talos", "xeus", "citard", 
        "ssm computer", "novamechanics"], "bd", "'CY'", pool).await?;
    records_affected += assign_lang_using_display_name(vec!["BAU$, WLB, CyRIC, ADITESS, ITRB"], 
        "bd", "'CY'", pool).await?;
    records_affected += assign_lang(vec!["the rest"], "en", "'CY'", pool).await?;

    info!("{} language codes added to Cyprus records", records_affected);
    Ok(())
}


pub async fn update_turkish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut records_affected = 0;

    records_affected += assign_lang(vec!["больница"], "ru", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["zanîngeha"], "ku", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["deutsche"], "de", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["institutu", "universiteti"], "az", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["français", "université"], "fr", "'TR'", pool).await?;

    records_affected += assign_lang(vec!["niversitesi", "akademi", "yerleşkesi̇", "hastanesi%", " ve ", 
        "dernegi", "eği", "enstitüsü", "ığı", "iği", "kanligi", "merkezi", "okulu"],
        "tr", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["yüksekokulu", "köğr", "ziraat", "silahli", "devlet", "teknoloji", 
        "nobel tip", "diyanet", "sbü gülhane", "anadolu", "yayıncılık", "hakkı"], 
        "tr", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["ulusal", "belediyesi"], 
        "tr", "'TR'", pool).await?;
   // 
   
    records_affected += assign_lang_using_acronym(vec!["ÇASGEM", "TTMD", "PRBL", "THD", "AYBU YEAH",
        "LÖSEV", "DKM", "ISTUN", "İSTÜN", "DAGTEM", "TÜBA", "GATA", "DEÜ", "TTGV", "AFAD", "TKASK"], 
        "tr", "'TR'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["UNAM", "TPD", "TSK", "TAGEM", "TAÜ", "ULAKBIM",
        "İZBÜ", "TAE", "İGÜ", "BTÖ", "TNKÜ", "TUSEB", "TÜSEB", "AY", "TCMB", "TKD", "EDH", "KGM"], 
        "tr", "'TR'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["GEAH", "TUG", "BBB", "MSÜ", "YÖKAK", "OBB",
        "Adana BTU"], 
        "tr", "'TR'", pool).await?;
    // 
    // 
    records_affected += assign_lang_using_acronym(vec!["FBU", "AGU", "MSU", "ABU"], 
        "tr, en", "'TR'", pool).await?;
    
    records_affected += assign_lang(vec!["nobel$", "akyüz", "humanis", "onder$", "önder$", "probel", 
        "fibabanka", "fcr yayin", "borsa istanbul"],"bd", "'TR'", pool).await?;
    records_affected += assign_lang_using_acronym(vec!["SUASIS", "FIBA", "BAU", "BIST", "TÜRKPATENT"], 
        "bd", "'TR'", pool).await?;

    records_affected += assign_lang(vec!["university", "hospital", "college", "ministry", "association", 
        "municipality", "institute", " and ", " of ", " for "],"en", "'TR'", pool).await?;
    records_affected += assign_lang(vec!["national", "education", "observatory"],"en", "'TR'", pool).await?;
    // 

    records_affected += assign_lang_using_acronym(vec![ "EUL", "TACRC", "KVCI", "THEQC", "FSVSOD", "KOERI",
        "Adana STU", "ASTU", "TSH" ],"en", "'TR'", pool).await?;
    // 

    //records_affected += assign_lang(vec!["the rest"], "en", "'CY'", pool).await?;

    //  -- Anonim Şirket, (A.Ş.)  or joint-stock company
    //  --A Limited Şirket, or limited liability company, (Ltd. Şti.)
    
    info!("{} language codes added to Turkey records", records_affected);
    Ok(())
}


/*
"TÜBİTAK National Observatory"
"TÜBİTAK ulusal Gözlemevi"
"TUG"
 
 en --- "university", "hospital", "college", "ministry", "association", "municipality", "institute", " and ", " of ", " for "
 
  
 ? erka group
 
 eskişehir  - just a place
 
 -- Anonim Şirket, (A.Ş.)  or joint-stock company
 --A Limited Şirket, or limited liability company, (Ltd. Şti.)
 */

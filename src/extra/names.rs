use sqlx::{Pool, Postgres};
use crate::AppError;
use log::info;



pub async fn add_langs_for_nonlatin_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let mut nonlatin_names = 0;

    nonlatin_names += update_lang_code_by_country("ru", "('RU')", pool).await?;
    nonlatin_names += update_lang_code_by_country("uk", "('UA')", pool).await?;
    nonlatin_names += update_lang_code_by_country("el", "('GR', 'CY')", pool).await?;
    nonlatin_names += update_lang_code_by_country("ja", "('JP')", pool).await?;
    nonlatin_names += update_lang_code_by_country("zh", "('CN', 'TW')", pool).await?;
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

    let sql  = r#"update ext.names n
        set lang_code = 'en'
        from ext.orgs c
        where n.id = c.id
        and n.lang_code is null 
        and n.script_code = 'Grek'
        and c.country_code = 'US';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_names += res.rows_affected();

    info!("{} Non-latin language codes applied", nonlatin_names); 

    Ok(())
}


async fn update_lang_code_by_country(lang_code: &str, country_code: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ext.names n
        set lang_code = '{}'
        from ext.orgs c
        where n.id = c.id
        and n.lang_code is null 
        and n.script_code <> 'Latn'
        and c.country_code in {} ;"#, lang_code, country_code);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


async fn update_lang_code_by_script(lang_code: &str, script_code: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql  = format!(r#"update ext.names n
        set lang_code = '{}'
        where n.lang_code is null 
        and n.script_code in {} ;"#, lang_code, script_code);

    let res = sqlx::query(&sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql))?;

    Ok(res.rows_affected())
}


pub async fn add_cm_lang_code_to_comm_orgs(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql = r#"update ext.names n
                set lang_code = 'cm',
                lang_source = 'cm_brand'
                from ext.type t
                where n.id = t.id
                and t.org_type = 400"#;

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} names of commercial organisations given 'cm' language code", res.rows_affected());
  
    Ok(())
}


pub async fn update_english_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
        set lang_code = 'en'
        where n.lang_code is null
        and n.name_type <> 10
        and (name_to_match like '%university%' 
        or name_to_match like '%college%'
        or name_to_match like '%polytechnic%'
        or name_to_match like '%museum%'
        or name_to_match like '%institute%'
        or name_to_match like '%center%'
        or name_to_match like '%clinic%'
        or name_to_match like '%library%'
        or name_to_match like '%society%');"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update ext.names n
        set lang_code = 'en'
        where n.lang_code is null
        and n.name_type <> 10
        and (name_to_match like '%foundation%'
        or name_to_match like '% trust%'
        or name_to_match like '%laboratory%'
        or name_to_match like '%laboratories%'
        or name_to_match like '%bureau%'
        or name_to_match like '%academy%'
        or name_to_match like '% zoo%'
        or name_to_match like '% park%'
        or name_to_match like '% garden%'
        or name_to_match like '%wikimedia%');"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update ext.names n
        set lang_code = 'en'
        where n.lang_code is null
        and n.name_type <> 10
        and (name_to_match like '%forum%'
        or name_to_match like '%municipal%'
        or name_to_match like '%medical%'
        or name_to_match like '%health%'
        or name_to_match like '%sanitorium%'
        or name_to_match like '%genebank%');"#;

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update ext.names n
        set lang_code = 'en'
        where n.lang_code is null
        and n.name_type <> 10
        and (n.name_to_match like '%observatory%'
        or n.name_to_match like '%observatories%')
        and n.name not like '%ПМФ%'
    "#;

    let res = sqlx::raw_sql(sql).execute(pool)
    .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update ext.names n
            set lang_code = 'en'
            where n.lang_code is null
            and n.name_type <> 10
            and n.name_to_match like '%school%'
            and n.name_to_match not like '%hochshule%'
        "#;
    
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update ext.names n
    set lang_code = 'en'
    from ext.org_countries c
    where n.id = c.id
    and n.lang_code is null
    and n.name_type <> 10
    and name_to_match like '%hospital%'
    and c.country_code not in ('AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 
    'ES', 'GQ', 'GT', 'HN', 'MX', 'NI', 'PE', 'PY', 'UY', 'VE', 
    'PT', 'BR', 'CV', 'AO', 'MZ', 'GW', 'ST', 'TL' );"#;
    
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

   
    let sql = r#"update ext.names n
    set lang_code = 'en'
    where n.lang_code is null
    and n.name_type <> 10
    and name_to_match like '%network%'  
    and name_to_match not like '%researcherenye%'"#;
    
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to english names", total_records_affected);

    Ok(())

    // institite and centre??? - soplit between anglophone and francophone...
}


pub async fn update_japanese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'ja'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'JP'
            and 
            (name_to_match like '%daigaku%'
            or name_to_match like '%daigakkō%'
            or name_to_match like '%kabushiki%'
            or name_to_match like '%nippon%' 
            or name_to_match like '%kaihatsu%' 
            or name_to_match like '%bijutsukan%');"#;   
            
            // university
            // college
            // corporation
            // Japan
            // development
            // art museum
                        
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();
    
    let sql = r#"update ext.names n
            set lang_code = 'ja'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'JP'
            and 
            (name_to_match like '%kenritsu%' 
            or name_to_match like '%dokuritsu%'  
            or name_to_match like '% kikō%'
            or name_to_match like '%gakkō%'
            or name_to_match like '%gakko%'
            or name_to_match like '%gakkou%'
            or name_to_match like '%kaihatsu%'
            or name_to_match like '%-shō%'
            or name_to_match like '%bunka senta%' 
            or name_to_match like '%denryoku%'  
            or name_to_match like '%gakuen%'
            or name_to_match like '%kagaku-kan%'
            or name_to_match like '%bungaku-kan%'
            or name_to_match like '%-chō%');"#;

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

    let sql = r#"update ext.names n
            set lang_code = 'ja'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'JP'
            and (name_to_match like '%chuobyoin%'
            or name_to_match like '%shiritsu%'  
            or name_to_match like '%kenkyūjo%'
            or name_to_match like '%kenkyujo%'
            or name_to_match like '%kenkyūsho%'
            or name_to_match like '%kenkei%'
            or name_to_match like '%kyōdō%');"#;
            
            // medical center
            // municipal
            // research institute (3)
            // survey
            // collaboration
            
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update ext.names n
        set lang_code = 'ja'
        from ext.org_countries c
        where n.id = c.id
        and n.lang_code is null
        and n.name_type <> 10
        and c.country_code = 'JP'
        and (name_to_match like '%tankyu%'
        or name_to_match like '%kenkyusho%'
        or name_to_match like '%kenkyuu%'
        or name_to_match like '%kokusai%'
        or name_to_match like '%hakubutsukan%'
        or name_to_match like '%toshoken%'
        or name_to_match like '%byoin%'
        or name_to_match like '%byōin%');"#;
        
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

    let sql = r#"update ext.names n
        set lang_code = 'ja'
        from ext.org_countries c
        where n.id = c.id
        and n.lang_code is null
        and n.name_type <> 10
        and c.country_code = 'JP'
        and (name_to_match like '%nihon%' 
        or name_to_match like '%kinzoku%'  
        or name_to_match like '%kenkyū%'
        or name_to_match like '%kokudo%'
        or name_to_match like '%jitsugyo%'
        or name_to_match like '%fukusei%' 
        or name_to_match like '%shiryokan%'  
        or name_to_match like '%gurūpu%'
        or name_to_match like 'shiritsuchuobyoin%'
        or name_to_match like '%kenkyuukikou%'
        or name_to_match like '%shiminbyoin%');"#;

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

    let sql = r#"update ext.names n
                set lang_code = 'zh'
                from ext.org_countries c
                where n.id = c.id
                and n.lang_code is null
                and n.name_type <> 10
                and c.country_code in ('CN', 'TW', 'HK')
                and (name_to_match like '%dàxué%'
                or name_to_match like '%daxue%'
                or name_to_match like '%dàxúe%'
                or name_to_match like '%zhōngyī%'
                or name_to_match like '%xuéyuàn%'
                or name_to_match like '%yīyuàn%'
                or name_to_match like '%jīgòu%'
                or name_to_match like '%yánjiū%'
                or name_to_match like '%mínguó%'
                or name_to_match like '%yínháng%');"#;
                 

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


    let sql = r#"update ext.names n
                set lang_code = 'zh'
                from ext.org_countries c
                where n.id = c.id
                and n.lang_code is null
                and n.name_type <> 10
                and c.country_code in ('CN', 'TW', 'HK')
                and (name_to_match like '%yīyún%'
                or name_to_match like '%yánjiùyuàn%'
                or name_to_match like '%ybówùguǎn%'
                or name_to_match like '%xuéxiào%'
                or name_to_match like '%shénxué%'
                or name_to_match like '%gōngyè%'
                or name_to_match like '%zhèngfǔ%'
                or name_to_match like '%guójiā%' 
                or name_to_match like '%shīfàn%' 
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

    let sql = r#"update ext.names n
            set lang_code = 'fr'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code in ('FR', 'PF')
            and 
            (name ilike 'Inserm %'
            or name like 'CH %' 
            or name like 'CHU %'
            or name like 'CIC %'
            or name like 'EA%' 
            or name like 'ERL %' 
            or name like 'GDR%' 
            or name like 'U %'
            or name like 'UAR%'
            or name like 'UMR%'
            or name like 'UMRS %' 
            or name like 'UMR_S %' 
            or name like 'UMS %'
            or name like 'UR%'
            or name like 'URP %'
            or name like 'US%');"#;

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

    info!("{} language codes added to french records", total_records_affected);
    
    Ok(())
}


pub async fn update_indian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'en'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'IN'
            and 
            (name like 'AIIMS%'
            or name like 'GCE%'
            or name like 'GMC%'
            or name like 'IIIT%'
            or name like 'IIM%'
            or name like 'IISER%'
            or name like 'IIT%' 
            or name like 'NIPER%'
            or name like 'NIT%'
            or name like 'RDC%'
            or name like 'REC%'
            or name like 'SKUAST%'
            or name like 'JNT%'
            or name_to_match like '%centre%'
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


    let sql = r#"update ext.names n
    set lang_code = 'hi'
    from ext.org_countries c
    where n.id = c.id
    and n.lang_code is null
    and n.name_type <> 10
    and c.country_code = 'IN'
    and 
    (name like 'KVK %'
    or name_to_match like 'GCE%'
    or name_to_match like '% vigyan%'
    or name_to_match like '% vishwavidyalaya%'
    or name_to_match like '% sanstha%'
    or name_to_match like '% sansthā%'
    or name_to_match like '% vidyālaya%'
    or name_to_match like '%krishi%'
    or name_to_match like '%samsthana%');"#;

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

    let sql = r#"update ext.names n
            set lang_code = 'fa'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'IR'
            and name_to_match like '%dāneshgāh%';"#;

        // dāneshgāh    university

    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to iranian records", total_records_affected);
    
    Ok(())
}


pub async fn update_russian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;
    

    let sql = r#"update ext.names n
            set lang_code = 'ru'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'RU'
            and (name_to_match like '%institut %'
            or name_to_match like '%universitet%'
            or name_to_match like '%akademiya%'
            or name_to_match like '%akadémiya%'
            or name_to_match like '%oblastnoy%'
            or name like 'JSC %');"#;

            // JSC  Scientific research institute

    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update ext.names n
            set lang_code = 'ru'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'RU'
            and (name_to_match like '%federalnyy%'
            or name_to_match like '%patologii%'
            or name_to_match like '%khirurgii%'
            or name_to_match like '%shkola%'
            or name_to_match like '%kombinat%'
            or name_to_match like '%tsentr%');"#;

     let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to russian records", total_records_affected);
    
    Ok(())
}


pub async fn update_ukrainian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;
   
    let sql = r#"update ext.names n
            set lang_code = 'uk'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'UA'
            and (name_to_match like '%universitét %'
            or name_to_match like '%universytet%'
            or name_to_match like '%ukrainsky%'
            or name_to_match like '%ukrayinska%'
            or name_to_match like '%ukrayiny%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to ukranian records", total_records_affected);
    
    Ok(())
}


pub async fn update_norwegian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'no'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'NO'
            and (name_to_match like '%sykehus%' 
            or name_to_match like '%skole%' 
            or name_to_match like '%skule%' 
            or name_to_match like '%universitet%' 
            or name_to_match like '% i %'
            or name_to_match like '%ø%'
            or name_to_match like '%direktoratet%'
            or name_to_match like '%registeret%'
            or name_to_match like '%kommune%'
            or name_to_match like '%instituut%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to norwegian records", total_records_affected);
    
    Ok(())
}


pub async fn update_serbian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'sr'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'RS'
            and (name_to_match like '%institut%' 
            or name_to_match like '%univerzitet%' 
            or name_to_match like '%zvezdara%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to serbian records", total_records_affected);
    
    Ok(())
}


pub async fn update_bulgarian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'bg'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'BG'
            and (name_to_match like '%institut%' 
            or name_to_match like '%akademiya%' 
            or name_to_match like '%universitet%'
            or name_to_match like '%ministerstvo%' 
            or name_to_match like '%obshtina%'
            or name_to_match like '%muzei%'
            or name_to_match like '%medicinska%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to bulgarian records", total_records_affected);
    
    Ok(())
}


pub async fn update_israeli_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'he'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'IL'
            and (name_to_match like '%ha-universita%' 
            or name_to_match like '%hauniversita%' 
            or name_to_match like '%machon %'
            or name_to_match like '%merkaz %' 
            or name_to_match like '%misrad %'
            or name_to_match like '%misgav %'
            or name_to_match like '%mikhlelet%'
            or name_to_match like '%miklelet%');"#;

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

    let sql = r#"update ext.names n
            set lang_code = 'ko'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'KR'
            and (name_to_match like '%daehak%' 
            or name_to_match like '%hakkyo%'
            or name_to_match like '%taehak%');"#;

            // daehak  
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to korean records", total_records_affected);
    
    Ok(())
}


pub async fn update_greek_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update ext.names n
            set lang_code = 'en'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'GR'
            and (name_to_match like 'tei %');"#;

            // tei     Technological Educational Institute
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    let sql = r#"update ext.names n
            set lang_code = 'el'
            from ext.org_countries c
            where n.id = c.id
            and n.lang_code is null
            and n.name_type <> 10
            and c.country_code = 'GR'
            and (name_to_match like '%panepistimio%'
            or name_to_match like '%panepistimiako%'
            or name_to_match like '%ellinikon%'
            or name_to_match like '%institouto%'
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

/* 
pub async fn obtain_manual_coding_list(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql  = r#"drop table if exists ext.manual_codes;
                CREATE TABLE ext.manual_codes (
                    id varchar NOT NULL,
                    name varchar NULL,
                    name_to_match varchar NULL,
                    name_type varchar NULL,
                    lang_code varchar NULL,
                    notes varchar NULL
                );
                CREATE INDEX manual_codes_id ON ext.manual_codes USING btree (id);
                CREATE INDEX manual_codes_name ON ext.manual_codes USING btree (name_to_match); "#;

    sqlx::raw_sql(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql  = r#"copy ext.manual_codes FROM 'E:\Resources - Data\ROR\manual_coding.csv' DELIMITER ',' CSV HEADER; "#;

    let res = sqlx::raw_sql(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} manual coding records imported from file", res.rows_affected());

    Ok(())
}


pub async fn apply_manual_coding_list(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let sql  = r#"update ext.names n
                set lang_code = m.lang_code
                from ext.manual_codes m
                where n.id = m.id
                and n.name_to_match = m.name_to_match
                and n.lang_code is null; "#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} language codes applied from manual coding data", res.rows_affected());

    Ok(())
}

*/





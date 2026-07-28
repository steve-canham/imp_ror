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
  
    update_english_names(pool).await?;
    update_japanese_names(pool).await?;
    update_chinese_names(pool).await?;
    update_french_names(pool).await?;
    update_german_names(pool).await?;
    update_spanish_names(pool).await?;
    update_portuguese_names(pool).await?;
    update_italian_names(pool).await?;
    update_dutch_names(pool).await?;
    update_swedish_names(pool).await?;
    update_finnish_names(pool).await?;
    update_norwegian_names(pool).await?;
    update_indian_names(pool).await?;
    update_iranian_names(pool).await?;
    update_russian_names(pool).await?;
    update_ukrainian_names(pool).await?;
    update_norwegian_names(pool).await?;
    update_serbian_names(pool).await?;
    update_bulgarian_names(pool).await?;
    update_israeli_names(pool).await?;
    update_korean_names(pool).await?;
    update_greek_names(pool).await?;

    // Do language of acronyms where all other names have the same language
    // See what are left
    
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
        or lc_value like '%data%'
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
        or lc_value like '%centre for%'
        or lc_value like '%society%');"#;

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
        or lc_value like '%bureau%'
        or lc_value like '%academy%'
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
        and (lc_value like '%forum%'
        or lc_value like '%municipal%'
        or lc_value like '%medical%'
        or lc_value like '%health%'
        or lc_value like '%sanitorium%'
        or lc_value like '%genebank%');"#;

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
            and n.lc_value like '%school%'
            and n.lc_value not like '%hochshule%'
        "#;
    
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'en'
    where der_lang is null and n.name_type <> 10
    and lc_value like '%hospital%'
    and country_code not in ('AR', 'BO', 'CL', 'CO', 'CR', 'CU', 'DO', 'EC', 
    'ES', 'GQ', 'GT', 'HN', 'MX', 'NI', 'PE', 'PY', 'UY', 'VE', 
    'PT', 'BR', 'CV', 'AO', 'MZ', 'GW', 'ST', 'TL' );"#;
    
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

/*
  
 
 für und klinische klinik  klinikum bundesamt fachhochschule fraunhofer zentrum
 akademie allgemeine deutsche gesellschaft krankenhaus hochschule wissenschaft
 arbeitsgemeinschaft  zentrum bundesverband europäische forschungsinstitut
 'institut für ' kantonsschule kantonsspital katholische forschung österreichische schweizerische
  stiftung technische universitaet universität vereinigung wasser
 
 --es
 academia** agencia asociación ayuntamiento banco
 benemérita biblioteca centro** científico clínica clínico colegio comisión 
 consejo consorcio corporación departamento**  dirección 
 escuela española  estación facultad fundación 
 gobierno grupo** hospital??? hospitalario institución instituto**
 laboratorio** médico  milenium  ministerio museo** observatorio** organización
 parque** pontificia** salud sanitas secretaría sistema sociedad
 tecnológico** tecnm unidad universitario universidad
 
 --pt
 academia** agência associação autoridade biblioteca centro** ciência
 comissão conselho departamento** direção escola estudos  faculdade federação
 fundação gabinete grupo** hospitalar** hospital??? investigação  instituto**
 laboratório** ministério** museu observatório** ordem parque** pesquisa
 sociedade tecnologia tecnológico** unidade
 universitário universidade
 
 --it
 accademia  agenzia archivio associazione azienda
 'centro di '  conservatorio  consorzio dipartimento federazione
 fondazione gruppo istituto liceo ministero  museo**
 organizzazione ospedale osservatorio pontificia** regione
 scuola  sistema societa ufficio università
 università 
 
 --nl
 academisch gemeentelijke gezondheidsdienst koninklijke
  voor ziekenhuis ministerie nationaal nederlandse instituut stichting
 universiteit  vereniging zorg
 
 --sv
 akademin för finlands finländska finska folktandvården föreningen göteborgs
 högskolan  i**  kungliga landstinget länsstyrelsen stiftelsen svenska sveriges
 trafikverket sjukhus västra
 
 --fi
 yliopisto  säätiö  suomi etelä föreningen helsingin juhani kansainvälisen 
 instituutti korkeakoulu lääketieteellisen  maa- ja  norjan  pohjois-karjalan pohjoismainen
 pohjois-suomen ruotsin satakunnan suomalainen suomen tampereen turun vaasan yhteis
 

 * 
 * 
 */

pub async fn update_french_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'PF', 'CH')
            and 
            (lc_value ilike '%académie%'
            or lc_value like '%agence%' 
            or lc_value like '%école%'
            or lc_value like '%environnement%'
            or lc_value like '%université%' 
            or lc_value like '%laboratoire%' 
            or lc_value like '%réseau%' 
            or lc_value like '%société%'
            or lc_value like '%santé%'
            or lc_value like '%publique%'
            or lc_value like '%mondiale%' 
            or lc_value like '%équipe%' 
            or lc_value like '%linstitut%'
            or lc_value like '%maison%'
            or lc_value like '%système%'
            or lc_value like '% et %');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();


    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'PF', 'CH')
            and 
            (lc_value ilike '%canadienne%'
            or lc_value like '%banque%' 
            or lc_value like '%bibliothèque%'
            or lc_value like '%gouvernement%'
            or lc_value like '%informatique%' 
            or lc_value like '%unité%' 
            or lc_value like '%française%' 
            or lc_value like '%recherche%'
            or lc_value like '%développement%'
            or lc_value like '%biologie%'
            or lc_value like '%génétique%' 
            or lc_value like '%observatoire%' 
            or lc_value like '%centre de%'
            or lc_value like '%centre universitaire%'
            or lc_value like '%générale%'
            or lc_value like '%fédération%');"#;
    let res = sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'PF', 'CH')
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
     
    
    info!("{} language codes added to french records", total_records_affected);
    
    let sql = r#"update rec.names n
                set der_lang = 'fr'
            where der_lang is null and n.name_type <> 10
            and country_code in ('FR', 'PF')
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

    info!("{} language codes added to german records", total_records_affected);
    
    Ok(())
}


pub async fn update_spanish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
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

    info!("{} language codes added to spanish records", total_records_affected);
    
    Ok(())
}


pub async fn update_portuguese_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
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

    info!("{} language codes added to spanish records", total_records_affected);
    
    Ok(())
}


pub async fn update_italian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
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

    info!("{} language codes added to spanish records", total_records_affected);
    
    Ok(())
}


pub async fn update_dutch_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
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

    info!("{} language codes added to spanish records", total_records_affected);
    
    Ok(())
}

pub async fn update_swedish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
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

    info!("{} language codes added to spanish records", total_records_affected);
    
    Ok(())
}


pub async fn update_finnish_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'es'
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

    info!("{} language codes added to spanish records", total_records_affected);
    
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


pub async fn update_norwegian_names(pool: &Pool<Postgres>) -> Result<(), AppError> {

    let mut total_records_affected = 0;

    let sql = r#"update rec.names n
                set der_lang = 'no'
            where der_lang is null and n.name_type <> 10
            and country_code = 'NO'
            and (lc_value like '%sykehus%' 
            or lc_value like '%skole%' 
            or lc_value like '%skule%' 
            or lc_value like '%universitet%' 
            or lc_value like '% i %'
            or lc_value like '%ø%'
            or lc_value like '%direktoratet%'
            or lc_value like '%registeret%'
            or lc_value like '%kommune%'
            or lc_value like '%instituut%');"#;
 
    let res = sqlx::raw_sql(sql).execute(pool)
            .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    total_records_affected += res.rows_affected();

    info!("{} language codes added to norwegian records", total_records_affected);
    
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
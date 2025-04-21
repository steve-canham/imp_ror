use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::AppError;
use log::info;

pub async fn store_org_attribute_numbers (pool: &Pool<Postgres>) -> Result<(), AppError> {
        
    execute_sql(get_name_data_sql(), pool).await?;
    execute_sql(get_label_data_sql(), pool).await?;
    execute_sql(get_alias_data_sql(), pool).await?;
    execute_sql(get_acronym_data_sql(), pool).await?;

    info!("Basic name data summarised in admin data table");

    execute_sql(get_nacro_data_sql(), pool).await?;
    execute_sql(get_names_wolc_sql(), pool).await?;
    execute_sql(get_nacro_wolc_sql(), pool).await?;

    info!("Name language code data summarised in admin data table");

    execute_sql(get_companies_sql(), pool).await?;
    execute_sql(get_types_data_sql(), pool).await?;

    info!("Types data summarised in admin data table");
    
    execute_sql(get_isni_data_sql(), pool).await?;
    execute_sql(get_grid_data_sql(), pool).await?;
    execute_sql(get_fundref_data_sql(), pool).await?;
    execute_sql(get_wikidata_data_sql(), pool).await?;
    execute_sql(get_ext_ids_data_sql(), pool).await?;

    info!("External ID summarised in admin data table");

    execute_sql(get_wikipedia_data_sql(), pool).await?;
    execute_sql(get_website_data_sql(), pool).await?;
    execute_sql(get_links_data_sql(), pool).await?;

    info!("Links data summarised in admin data table");
    
    execute_sql(get_parrels_data_sql(), pool).await?;
    execute_sql(get_chrels_data_sql(), pool).await?;
    execute_sql(get_relrels_data_sql(), pool).await?;
    execute_sql(get_predrels_data_sql(), pool).await?;
    execute_sql(get_succrels_data_sql(), pool).await?;
    execute_sql(get_domains_data_sql(), pool).await?;

    info!("Relationship, domain data summarised in admin data table");

    execute_sql(get_locations_data_sql(), pool).await?;
    execute_sql(get_subdivs_data_sql(), pool).await?;
    execute_sql(get_countries_data_sql(), pool).await?;

    info!("Location data summarised in admin data table");

    execute_sql(update_core_data_sql1(), pool).await?;
    execute_sql(update_core_data_sql2(), pool).await?;
    execute_sql(update_core_data_sql3(), pool).await?;
    execute_sql(update_core_data_sql4(), pool).await?;
 
    info!("Location data added to core data table");

    Ok(())
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}

fn get_name_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_names = n
    from (
        select id, count(id) as n
        from src.names 
        group by id) c
    where ad.id = c.id;"#
}

fn get_label_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_labels = n
    from (
        select id, count(id) as n
        from src.names where name_type = 5
        group by id) c
    where ad.id = c.id;"#
}

fn get_alias_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_aliases = n
    from (
        select id, count(id) as n
        from src.names where name_type = 7
        group by id) c
    where ad.id = c.id;"#
}

fn get_acronym_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_acronyms = n
    from (
        select id, count(id) as n
        from src.names where name_type = 10
        group by id) c
    where ad.id = c.id;"#
}

fn get_nacro_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_nacro = n_names - n_acronyms;"#
}

fn get_names_wolc_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_names_wolc = n
    from (
        select id, count(id) as n
        from src.names 
        where lang_code is null
        group by id) c
    where ad.id = c.id;"#
}

fn get_nacro_wolc_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_nacro_wolc = n
    from (
        select id, count(id) as n
        from src.names 
        where lang_code is null and name_type <> 10
        group by id) c
    where ad.id = c.id;"#
}

fn get_companies_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set is_company = true
    from src.type t
    where ad.id = t.id
    and t.org_type = 400;"#
}

fn get_types_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_types = n
    from (
        select id, count(id) as n
        from src.type 
        group by id) c
    where ad.id = c.id;"#
}

fn get_isni_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_isni = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 11
        group by id) c
    where ad.id = c.id;"#
}

fn get_grid_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_grid = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 13
        group by id) c
    where ad.id = c.id;"#
}

fn get_fundref_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_fundref = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 14
        group by id) c
    where ad.id = c.id;"#
}

fn get_wikidata_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_wikidata = n
    from (
        select id, count(id) as n
        from src.external_ids 
        where id_type = 12
        group by id) c
    where ad.id = c.id;"#
}

fn get_ext_ids_data_sql <'a>() -> &'a str {
    r#"update src.admin_data
    set n_ext_ids = n_isni + n_grid + n_fundref + n_wikidata;"#
}

fn get_wikipedia_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_wikipedia = n
    from (
        select id, count(id) as n
        from src.links 
        where link_type = 21
        group by id) c
    where ad.id = c.id;"#
}

fn get_website_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_website = n
    from (
        select id, count(id) as n
        from src.links 
        where link_type = 22
        group by id) c
        where ad.id = c.id;"#
}

fn get_links_data_sql <'a>() -> &'a str {
    r#"update src.admin_data
    set n_links = n_wikipedia + n_website"#
}


fn get_locations_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_locs = n
    from (
        select id, count(id) as n
        from src.locations 
        group by id) c
    where ad.id = c.id;"#
}

fn get_subdivs_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_subdivs = n
    from (
        select id, count(country_subdivision_code) as n
        from
            (select distinct id, country_subdivision_code
            from ror.locations t) t
        group by t.id) c
    where ad.id = c.id;"#
}

fn get_countries_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_countries = n
    from (
        select id, count(country_code) as n
        from
            (select distinct id, country_code
            from ror.locations t) t
        group by t.id) c
    where ad.id = c.id;"#
}

fn get_parrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_parrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 1
        group by id) c
    where ad.id = c.id;"#
}

fn get_chrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_chrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 2
        group by id) c
    where ad.id = c.id;"#
}

fn get_relrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_relrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 3
        group by id) c
    where ad.id = c.id;"#
}

fn get_predrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_predrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 4
        group by id) c
    where ad.id = c.id;"#
}

fn get_succrels_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_sucrels = n
    from (
        select id, count(id) as n
        from src.relationships
        where rel_type = 5
        group by id) c
    where ad.id = c.id;"#
}

fn get_domains_data_sql <'a>() -> &'a str {
    r#"update src.admin_data ad
    set n_doms = n
    from (
        select id, count(id) as n
        from src.domains 
    group by id) c
    where ad.id = c.id;"#
}

// update core table with location data - for single location orgs

fn update_core_data_sql1 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = t.name,
    csubdiv_code = t.country_subdivision_code,
    country_code = t.country_code
    from  
       (select a.id, name, country_subdivision_code, country_code
        from ror.locations loc
        inner join src.admin_data a
        on loc.id = a.id
        where a.n_locs = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, single subdiv

fn update_core_data_sql2 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = '**multi**',
    csubdiv_code = t.country_subdivision_code,
    country_code = t.country_code
    from 
        (select distinct a.id, country_subdivision_code, country_code
        from ror.locations loc
        inner join src.admin_data a
        on loc.id = a.id
        where a.n_locs > 1
        and a.n_subdivs = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, multi subdivs, single country

fn update_core_data_sql3 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = '**multi**',
    csubdiv_code = '**',
    country_code = t.country_code
    from 
        (select distinct a.id, country_code
        from ror.locations loc
        inner join src.admin_data a
        on loc.id = a.id
        where a.n_subdivs > 1
        and a.n_countries = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, multi countries

fn update_core_data_sql4 <'a>() -> &'a str {
    r#"update src.core_data c
    set location = '**multi**',
    csubdiv_code = '**',
    country_code = '**'
    from src.admin_data a
    where c.id = a.id 
    and a.n_countries > 1;"#
}


pub async fn tidy_names (pool: &Pool<Postgres>) -> Result<(), AppError> {

    // remove any of the set of zero width characters 

    let mut no_width_chars = 0;

    // zero width space

    let sql = r#"update src.names
              set value = replace(value, U&'\200B', '')
              where value like U&'%\200B%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    no_width_chars  += res.rows_affected();

    // zero width no join

    let sql = r#"update src.names
              set value = replace(value, U&'\200C', '')
              where value like U&'%\200C%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    no_width_chars  += res.rows_affected();

    // zero width join

    let sql = r#"update src.names
            set value = replace(value, U&'\200D', '')
            where value like U&'%\200D%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    no_width_chars  += res.rows_affected();

    // zero width word joiner

    let sql = r#"update src.names
            set value = replace(value, U&'\2060', '')
            where value like U&'%\2060%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    no_width_chars  += res.rows_affected();

    // zero width no-break space

    let sql = r#"update src.names
            set value = replace(value, U&'\FEFF', '')
            where value like U&'%\FEFF%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    no_width_chars  += res.rows_affected();

    info!("{} no width characters removed from names", no_width_chars);

    // remove any bullet characters
    // Only one seems to be present at the monment

    let sql = r#"update src.names
            set value = replace(value, U&'\2022', '')
            where value like U&'%\2022%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} bullet characters removed from names", res.rows_affected());
             
    Ok(())
}


pub async fn prepare_names_for_script_codes(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // set up the 'names_pad' table as a copy of the trimmed value (name) column

    let sql = r#"Insert into src.names_pad (id, name, country_code, lang_code)
            select n.id, n.value, c.country_code, n.lang_code
            from src.names n
            inner join 
            src.core_data c
            on n.id = c.id"#;

    sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("Names copied for processing porior to script coding");

    // remove 'latin' characters that could be at the start or end of non latin names

    let mut latin_punctuation = 0;

    // commas, semi-colons and full stops

    let sql = r#"update src.names_pad
            set name = replace(name, '.', '')
            where name like '%.%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
    set name = replace(name, ',', '')
    where name like '%,%'; "#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
                set name = replace(name, ';', '')
                where name like '%;%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    info!("{} commas, full stops and semi-colons removed from name copies", latin_punctuation);

    // parentheses

    let mut latin_punctuation = 0;

    let sql = r#"update src.names_pad
            set name = replace(name, '(', '')
            where name like '%(%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
            set name = replace(name, ')', '')
            where name like '%)%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    info!("{} parantheses characters removed from name copies", latin_punctuation);

    // brackets

    let sql = r#"update src.names_pad
            set name = replace(name, '[', '')
            where name like '%[%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
            set name = replace(name, ']', '')
            where name like '%]%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    info!("{} bracket characters removed from name copies", latin_punctuation);

    // double quotes, apostrophes

    let sql = r#"update src.names_pad
            set name = replace(name, '"', '')
            where name like '%"%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} double quotes removed from name copies", res.rows_affected());

    let sql = r#"update src.names_pad
            set name = replace(name, '''', '')
            where name like '%''%'; "#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("{} apostrophes removed from name copies", res.rows_affected());

    // guillemets U+00AB, U+00BB

    let mut latin_punctuation = 0;

    let sql = r#"update src.names_pad
            set name = replace(name, U&'\00AB', '')
            where name like U&'%\00AB%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
            set name = replace(name, U&'\00BB', '')
            where name like U&'%\00BB%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    latin_punctuation += res.rows_affected();

    info!("{} guillemets characters removed from name copies", latin_punctuation);


    // remove 'non-latin' characters that could be at the start or end of latin names

    let mut nonlatin_punctuation = 0;

    // single quotes

    let sql = r#"update src.names_pad
            set name = replace(name, U&'\2018', '')
            where name like U&'%\2018%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
            set name = replace(name, U&'\2019', '')
            where name like U&'%\2019%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();
    
    let sql = r#"update src.names_pad
            set name = replace(name, U&'\201A', '')
            where name like U&'%\201A%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
            set name = replace(name, U&'\201B', '')
            where name like U&'%\201B%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    info!("{} non latin single quote characters removed from name copies", nonlatin_punctuation);
    let mut nonlatin_punctuation = 0;

    // double quotes
        
    let sql = r#"update src.names_pad
        set name = replace(name, U&'\201C', '')
        where name like U&'%\201C%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
        set name = replace(name, U&'\201D', '')
        where name like U&'%\201D%'"#;

    let res = sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
        set name = replace(name, U&'\201E', '')
        where name like U&'%\201E%'"#;
    
    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    let sql = r#"update src.names_pad
        set name = replace(name, U&'\201F', '')
        where name like U&'%\201F%'"#;

    let res = sqlx::query(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    nonlatin_punctuation += res.rows_affected();

    info!("{} non latin double quote characters removed from name copies", nonlatin_punctuation);


    // do a final trim as removals above may have generated trailing spaces

    let sql = r#"update src.names_pad
            set name = trim(name)"#;

    sqlx::query(sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

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
    }

    // Get the Unicode scripts with their ascii code boundaries.

    let sql  = r#"select code, ascii_start, ascii_end
    from lup.lang_scripts
    where ascii_end <> 0
    order by ascii_start;"#;
    let rows: Vec<Script> = sqlx::query_as(sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("Unicode script characteristics obtained");
    
    // Update names records by testing against each unicode entry.

    let mut n = 0;
    for r in rows {

        let sql  = r#"update src.names_pad
        set script_code = $1 
        where ascii(substr(name, 1, 1)) >= $2
        and   ascii(substr(name, 1, 1)) <= $3"#;

        sqlx::query(sql)
        .bind(r.code.clone()).bind(r.ascii_start).bind(r.ascii_end)
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

        let sql = r#"update src.names_pad
        set script_code_end = $1 
        where ascii(substr(name, length(name), 1)) >= $2
        and   ascii(substr(name, length(name), 1)) <= $3"#;

        sqlx::query(sql)
        .bind(r.code.clone()).bind(r.ascii_start).bind(r.ascii_end)
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        
        n +=1;
        if n % 20 == 0 {
            info!("{} scripts processed...", n.to_string());
        }
    }

    let sql  = r#"update src.names_pad
        set script_code = script_code||', '||script_code_end
        where script_code <> script_code_end"#;

    let res = sqlx::query(sql).execute(pool).await
         .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    info!("{} names found using two scripts", res.rows_affected()); 

    Ok(())
}


pub async fn add_langs_for_nonlatin_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let mut nonlatin_names = 0;

    let sql  = r#"update src.names_pad
    set lang_code = 'ru'
    where lang_code is null
    and script_code <> 'Latn'
    and country_code = 'RU';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    nonlatin_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set lang_code = 'uk'
    where lang_code is null
    and script_code <> 'Latn'
    and country_code = 'UA';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    nonlatin_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set lang_code = 'el'
    where lang_code is null
    and script_code <> 'Latn'
    and country_code = 'GR';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    nonlatin_names += res.rows_affected();


    let sql  = r#"update src.names_pad
    set lang_code = 'ja'
    where lang_code is null
    and script_code <> 'Latn'
    and country_code = 'JP';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    nonlatin_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set lang_code = 'zh'
    where lang_code is null
    and script_code <> 'Latn'
    and country_code = 'CN';"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    nonlatin_names += res.rows_affected();

    info!("{} Non-latin language codes applied", nonlatin_names); 

    Ok(())
}


pub async fn clean_japanese_script_codes (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let mut japanese_nonlatin_names = 0;

    let sql  = r#"update src.names_pad
    set script_code = 'Jpan'
    where lang_code = 'ja' 
    and script_code in ('Kana', 'Hani', 'Hira')"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set script_code = 'Jpan'
    where lang_code = 'ja' 
    and script_code in ('Kana, Hani', 'Hani, Kana', 'Hira, Hani', 'Hani, Hira', 'Kana, Hira', 'Hira, Kana')"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    japanese_nonlatin_names += res.rows_affected();

    info!("{} japanese non-latin scripts recoded to 'Jpan'", japanese_nonlatin_names); 

    Ok(())
}


pub async fn clean_script_codes_with_numbers (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let mut rga_names = 0;

    let sql  = r#"update src.names_pad
    set script_code = 'Cyrl'
    where script_code = 'Cyrl, Latn'
    and ascii(substr(name, length(name), 1)) >= 48
    and ascii(substr(name, length(name), 1)) <= 57;"#;
    
    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set script_code = 'Cyrl'
    where script_code =  'Latn, Cyrl'
    and ascii(substr(name, 1, 1)) >= 48
    and ascii(substr(name, 1, 1)) <= 57"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set script_code = 'Grek'
    where script_code = 'Grek, Latn' 
    and ascii(substr(name, length(name), 1)) >= 48
    and ascii(substr(name, length(name), 1)) <= 57;"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set script_code = 'Grek'
    where script_code = 'Latn, Grek' 
    and ascii(substr(name, 1, 1)) >= 48
    and ascii(substr(name, 1, 1)) <= 57"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set script_code = 'Arab'
    where script_code = 'Arab, Latn'
    and ascii(substr(name, length(name), 1)) >= 48
    and ascii(substr(name, length(name), 1)) <= 57;"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    let sql  = r#"update src.names_pad
    set script_code = 'Arab'
    where script_code =  'Latn, Arab'
    and ascii(substr(name, 1, 1)) >= 48
    and ascii(substr(name, 1, 1)) <= 57"#;

    let res = sqlx::query(sql).execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    rga_names += res.rows_affected();

    info!("{} Russian, Greek and Arabic names with numbers recoded", rga_names); 

    Ok(())
}


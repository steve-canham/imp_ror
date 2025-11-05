use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn remove_dups (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Before further processing the duplicate names need to be removed from ppr.names. 
    // If this is not done the import to the core data, that follows, will fail, 
    // as some organisations have more than one name marked as the 'ror name' (the 
    // import therefore fails because of a duplicated PK). 

    // The duplicates, as found at the beginning of the process, are stored in src.dup_names

    // No organisation seems to have - in the source file - two names that are exactly the same in 
    // all respects  - i.e. have the same value, name type, language code and 'is_ror_name' status.
    // This is possible and should be guarded against but does not seem to occur.
    
    // More commonly, duplicates exist where an organisation has names with the same (lower case) name values
    // and lang code, but differ on the name type, or have the same value and name type, but differ 
    // on the language code. Both groups include pairs with the same 'is_ror_name' value, and 
    // pairs with the opposite 'is_ror_name' values.

    // First set up the table of names that are duplicated (same ror id and name value).
        
    let res =  execute_sql(make_duplicates_table(), pool).await?.rows_affected();
    info!("{} Duplicate name pairs identified", res / 2) ;
  
    // Use a 'scratch pad' table, src.dups, to hold duplicate pairs - reduces in size as
    // process driops duplicates and the table is reformed.

    recreate_dups(pool).await?;  

    // Drop names that are the non-ror equivalents of ror names.

    let res = drop_non_ror_name_dups(pool).await?;
    info!("{} names dropped that are the non-ror equivalents of ror names", res);

    recreate_dups(pool).await?;  

    // Drop names that are the alias equivalents of labels

    let res = drop_alias_dups(pool).await?;
    info!("{} names dropped that are the alias equivalents of labels", res);

    // Drop names that are one of an acronym - other name pair

    recreate_dups(pool).await?;  

    let res1 = drop_long_acros_dups(pool).await?;
    let res2 = drop_short_alias_dups(pool).await?; 
    info!("{} names dropped from acronym - other name pairs", res1 + res2);
    
    // Drop some specific errors using code for indivdual names

    let res = drop_specific_dups(pool).await?;
    info!("{} names dropped using name specific code to target them", res);

    recreate_dups(pool).await?;  

    // Drop the names with the lowest id in the remainder that are left

    let res = drop_lowest_ident_dups(pool).await?;
    info!("{} names dropped using the lowest Ident in the remaining duplicates", res);

    execute_sql(replace_deprecated_lang_code_sql(), pool).await?;  
    
    info!(""); 
    Ok(())
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
   
}

fn make_duplicates_table <'a>() -> &'a str {
    
    r#"insert into src.dup_names (ident, id, value, name_type, is_ror_name, lang_code)
    select n.ident, d.id, n.value, n.name_type, n.is_ror_name, n.lang
    from (
        select id, lower(value) as lvalue from src.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join src.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;"#
}


async fn recreate_dups(pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let sql = r#"drop table if exists src.dups;
        create table src.dups 
        (  
              ident             int         not null
            , id                varchar     not null
            , value             varchar     not null  
            , name_type         varchar     null 
            , is_ror_name       bool        null
            , lang_code         varchar     null
        );
        create index dup_names_full_idx on src.dups(id);"#;
        sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = r#"insert into src.dups (ident, id, value, name_type, is_ror_name, lang_code)
        select n.ident, d.id, n.value, n.name_type, n.is_ror_name, n.lang
        from (
            select id, lower(value) as lvalue from src.names
            group by id, lower(value) having count(id) > 1
        ) d
        inner join src.names n
        on d.id = n.id
        and d.lvalue = lower(n.value)
        order by d.id;"#;
        sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

        Ok(())

}


async fn drop_non_ror_name_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let sql = r#"update src.dup_names d
           set fate = 'Dropped because of non-ror status when ror equivalent present'
           from 
            (select f.* from 
                    (select * from src.dups
                    where is_ror_name = true) t
                inner join
                    (select * from src.dups
                    where is_ror_name = false) f
                on t.id = f.id
                and lower(t.value) = lower(f.value)) r
            where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"delete from src.names n
    using
        (select f.* from 
            (select * from src.dups
            where is_ror_name = true) t
        inner join
            (select * from src.dups
            where is_ror_name = false) f
        on t.id = f.id
        and lower(t.value) = lower(f.value)) r
    where n.id = r.id
    and n.value = r.value
    and n.is_ror_name = false;"#;

    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}


async fn drop_alias_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let sql = r#"update src.dup_names d
        set fate = 'Dropped because an alias when equivalent label present'
        from 
            (select s.* from 
                (select * from src.dups
                where name_type = 'label') f
            inner join
                (select * from src.dups
                where name_type = 'alias') s
            on f.id = s.id
            and lower(f.value) = lower(s.value)) r
        where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;


    let sql = r#"delete from src.names n
    using
        (select s.* from 
            (select * from src.dups
            where name_type = 'label') f
        inner join
            (select * from src.dups
            where name_type = 'alias') s
        on f.id = s.id
        and lower(f.value) = lower(s.value)) r
    where n.id = r.id
    and n.value = r.value
    and n.name_type = 'alias';"#;

    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}

async fn drop_long_acros_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let sql = r#"update src.dup_names d
        set fate = 'Dropped because an acronym when equivalent alias or label present'
        from 
            (select a.* from 
                (select * from src.dups
                where name_type <> 'acronym') f
            inner join
                (select * from src.dups
                where name_type = 'acronym') a
            on a.id = f.id
            and lower(a.value) = lower(f.value)
            and length(a.value) > 5) r
        where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"delete from src.names n
    using
        (select a.* from 
           (select * from src.dups
            where name_type <> 'acronym') f
        inner join
            (select * from src.dups
            where name_type = 'acronym') a
        on a.id = f.id
        and lower(a.value) = lower(f.value)
        where length(a.value) > 5) r
    where n.id = r.id
    and n.value = r.value
    and n.name_type = 'acronym';"#;

    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}

async fn drop_short_alias_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql = r#"update src.dup_names d
        set fate = 'Dropped because an alias or label when equivalent acronym present'
        from 
           (select f.* from 
                (select * from src.dups
                where name_type <> 'acronym') f
            inner join
                (select * from src.dups
                where name_type = 'acronym') a
            on f.id = a.id
            and lower(f.value) = lower(a.value)
            and length(a.value) <= 5) r
        where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"delete from src.names n
    using
        (select f.* from 
            (select * from src.dups
            where name_type <> 'acronym') f
        inner join
            (select * from src.dups
            where name_type = 'acronym') a
        on f.id = a.id
        and lower(f.value) = lower(a.value)
        and length(a.value) <= 5) r
    where n.id = r.id
    and n.value = r.value
    and n.name_type <> 'acronym';"#;

    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}

async fn drop_specific_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let mut total = 0;
    total += drop_specific_dup("00bep5t26", "Biblioteca de Catalunya", "gl" , pool).await?;
    total += drop_specific_dup("00dsy9f04", "Ebsco information services", "fr" , pool).await?;
    total += drop_specific_dup("00wge5k78", "Universitetet i Tromsø – Noregs arktiske universitet", "nn" , pool).await?;
    total += drop_specific_dup("00wge5k78", "UiT Noregs arktiske universitet", "nn" , pool).await?;
    total += drop_specific_dup("00yrf4e35", "Osservatorio Astronomico di Torino", "de" , pool).await?;
    total += drop_specific_dup("00zb6nk96", "Labor Spiez", "rm" , pool).await?;
    total += drop_specific_dup("01767d733", "Erasmushogeschool brussel", "fr" , pool).await?;
    total += drop_specific_dup("01cdn3r29", "École des Beaux-Arts", "ca" , pool).await?;
    total += drop_specific_dup("01kwczx50", "Skadden, arps, slate, meagher & flom", "fr" , pool).await?;
    total += drop_specific_dup("02vc99v03", "Calbinotox", "en" , pool).await?;
    total += drop_specific_dup("03v8adn41", "Queens college, city university of new york", "fr" , pool).await?;
    total += drop_specific_dup("05c2g3729", "Ministarstvo vanjskih poslova", "hr" , pool).await?;
    total += drop_specific_dup("05e0vkr08", "Bibliothèque nationale de Luxembourg", "de" , pool).await?;
    Ok(total)

}


async fn drop_specific_dup(id: &str, name: &str, lang: &str, pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let sql = format!(r#"update src.dup_names 
           set fate = 'Dropped using a specific call for this name / language code'
           where id = '{}' and value = '{}' and lang_code = '{}';"#, id, name, lang);
    execute_sql(&sql, pool).await?;

    let sql = format!("delete from src.names n where id = '{}' and value = '{}' and lang = '{}';", id, name, lang);
    let res = execute_sql(&sql, pool).await?.rows_affected();
   
    Ok(res)

}


async fn drop_lowest_ident_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql = r#"update src.dup_names d
        set fate = 'Dropped because the lower ident, other fields being equivalent'
        from 
           (select id, min(ident) as min
            from src.dups 
            group by id) r
        where d.ident = r.min;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"update src.dup_names d
        set fate = 'Retained'
        where fate is null;"#;

    execute_sql(&sql, pool).await?;

    let sql = r#"delete from src.names n
    using
        (select id, min(ident) as min
        from src.dups 
        group by id) r
    where n.id = r.id
    and n.ident = r.min;"#;

    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}
    

fn replace_deprecated_lang_code_sql <'a>() -> &'a str {
    r#"update ppr.names 
    set lang_code = 'sr'
    where lang_code = 'sh';"#
}
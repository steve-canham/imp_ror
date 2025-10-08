use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn remove_dups (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Before further processing the duplicate names need to be removed from src.names. 
    // If this is not done the import to the core data, that follows, will fail, 
    // as some organisations have more than one name marked as the 'ror name' (the 
    // import therefore fails because of a duplicated PK). 

    // The duplicates, as found at the beginning of the process, are stored in ror.dup_names

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
  
    // Use a 'scratch pad' table, ror.dups, to hold duplicate pairs - reduces in size as
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
    
    r#"insert into ror.dup_names (ident, id, value, name_type, is_ror_name, lang_code)
    select n.ident, d.id, n.value, n.name_type, n.is_ror_name, n.lang
    from (
        select id, lower(value) as lvalue from ror.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join ror.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;"#
}


async fn recreate_dups(pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    let sql = r#"drop table if exists ror.dups;
        create table ror.dups 
        (  
              ident             int         not null
            , id                varchar     not null
            , value             varchar     not null  
            , name_type         varchar     null 
            , is_ror_name       bool        null
            , lang_code         varchar     null
        );
        create index dup_names_full_idx on ror.dups(id);"#;
        sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = r#"insert into ror.dups (ident, id, value, name_type, is_ror_name, lang_code)
        select n.ident, d.id, n.value, n.name_type, n.is_ror_name, n.lang
        from (
            select id, lower(value) as lvalue from ror.names
            group by id, lower(value) having count(id) > 1
        ) d
        inner join ror.names n
        on d.id = n.id
        and d.lvalue = lower(n.value)
        order by d.id;"#;
        sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

        Ok(())

}


async fn drop_non_ror_name_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {
    
    let sql = r#"update ror.dup_names d
           set fate = 'Dropped because of non-ror status when ror equivalent present'
           from 
            (select f.* from 
                    (select * from ror.dups
                    where is_ror_name = true) t
                inner join
                    (select * from ror.dups
                    where is_ror_name = false) f
                on t.id = f.id
                and lower(t.value) = lower(f.value)) r
            where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"delete from ror.names n
    using
        (select f.* from 
            (select * from ror.dups
            where is_ror_name = true) t
        inner join
            (select * from ror.dups
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
    
    let sql = r#"update ror.dup_names d
        set fate = 'Dropped because an alias when equivalent label present'
        from 
            (select s.* from 
                (select * from ror.dups
                where name_type = 'label') f
            inner join
                (select * from ror.dups
                where name_type = 'alias') s
            on f.id = s.id
            and lower(f.value) = lower(s.value)) r
        where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;


    let sql = r#"delete from ror.names n
    using
        (select s.* from 
            (select * from ror.dups
            where name_type = 'label') f
        inner join
            (select * from ror.dups
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
    
    let sql = r#"update ror.dup_names d
        set fate = 'Dropped because an acronym when equivalent alias or label present'
        from 
            (select a.* from 
                (select * from ror.dups
                where name_type <> 'acronym') f
            inner join
                (select * from ror.dups
                where name_type = 'acronym') a
            on a.id = f.id
            and lower(a.value) = lower(f.value)
            and length(a.value) > 5) r
        where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"delete from ror.names n
    using
        (select a.* from 
           (select * from ror.dups
            where name_type <> 'acronym') f
        inner join
            (select * from ror.dups
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

    let sql = r#"update ror.dup_names d
        set fate = 'Dropped because an alias or label when equivalent acronym present'
        from 
           (select f.* from 
                (select * from ror.dups
                where name_type <> 'acronym') f
            inner join
                (select * from ror.dups
                where name_type = 'acronym') a
            on f.id = a.id
            and lower(f.value) = lower(a.value)
            and length(a.value) <= 5) r
        where d.ident = r.ident;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"delete from ror.names n
    using
        (select f.* from 
            (select * from ror.dups
            where name_type <> 'acronym') f
        inner join
            (select * from ror.dups
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
    
    let sql = format!(r#"update ror.dup_names 
           set fate = 'Dropped using a specific call for this name / language code'
           where id = '{}' and value = '{}' and lang_code = '{}';"#, id, name, lang);
    execute_sql(&sql, pool).await?;

    let sql = format!("delete from ror.names n where id = '{}' and value = '{}' and lang = '{}';", id, name, lang);
    let res = execute_sql(&sql, pool).await?.rows_affected();
   
    Ok(res)

}


async fn drop_lowest_ident_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let sql = r#"update ror.dup_names d
        set fate = 'Dropped because the lower ident, other fields being equivalent'
        from 
           (select id, min(ident) as min
            from ror.dups 
            group by id) r
        where d.ident = r.min;"#;
    
    execute_sql(&sql, pool).await?;

    let sql = r#"update ror.dup_names d
        set fate = 'Retained'
        where fate is null;"#;

    execute_sql(&sql, pool).await?;

    let sql = r#"delete from ror.names n
    using
        (select id, min(ident) as min
        from ror.dups 
        group by id) r
    where n.id = r.id
    and n.ident = r.min;"#;

    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}
    

/* 


-- get is_ror_name = false set from the 
-- mismatched is_ror_name pairs, and use that to deletre the coirresponding record from names

delete from src.names n
using
	(select f.* from 
		(select * from src.dup_names_full
		where is_ror_name = true) t
	inner join
		(select * from src.dup_names_full
		where is_ror_name = false) f
	on t.id = f.id
	and lower(t.value) = lower(f.value)) r
where n.id = r.id
and n.value = r.value
and n.is_ror_name = false

-- redo dup table
drop table if exists src.dup_names_full;
create table src.dup_names_full 
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_full_idx on src.dup_names_full(id);
insert into src.dup_names_full (id, value, name_type, is_ror_name, lang_code)
    select d.id, n.value, n.name_type, n.is_ror_name, n.lang_code
    from (
        select id, lower(value) as lvalue from src.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join src.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;

-- get is_ror_name = 7 set from the 
-- mismatched 5 and 7 pairs, and use that to deletre the coirresponding record from names

delete from src.names n
using
	(select s.* from 
		(select * from src.dup_names_full
		 where name_type = 5) f
	inner join
		(select * from src.dup_names_full
		where name_type = 7) s
	on f.id = s.id
	and lower(f.value) = lower(s.value)) r
where n.id = r.id
and n.value = r.value
and n.name_type = 7

-- redo dup table
drop table if exists src.dup_names_full;
create table src.dup_names_full 
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_full_idx on src.dup_names_full(id);
insert into src.dup_names_full (id, value, name_type, is_ror_name, lang_code)
    select d.id, n.value, n.name_type, n.is_ror_name, n.lang_code
    from (
        select id, lower(value) as lvalue from src.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join src.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;

-- get is_ror_name = 10 set from the 
-- mismatched 5 / 7 and 10 pairs, and use that to deletre the corresponding record from names

delete from src.names n
using
	(select s.* from 
		(select * from src.dup_names_full
		 where name_type <> 10) f
	inner join
		(select * from src.dup_names_full
		where name_type = 10) s
	on f.id = s.id
	and lower(f.value) = lower(s.value)) r
where n.id = r.id
and n.value = r.value
and length(n.value) > 5
and n.name_type = 10


delete from src.names n
using
	(select f.* from 
		(select * from src.dup_names_full
		 where name_type <> 10) f
	inner join
		(select * from src.dup_names_full
		where name_type = 10) s
	on f.id = s.id
	and lower(f.value) = lower(s.value)) r
where n.id = r.id
and n.value = r.value
and length(n.value) <= 5
and n.name_type <> 10

-- redo dup table
drop table if exists src.dup_names_full;
create table src.dup_names_full 
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_full_idx on src.dup_names_full(id);
insert into src.dup_names_full (id, value, name_type, is_ror_name, lang_code)
    select d.id, n.value, n.name_type, n.is_ror_name, n.lang_code
    from (
        select id, lower(value) as lvalue from src.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join src.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;


delete from src.names n where id = '00bep5t26' and value = 'Biblioteca de Catalunya' and lang_code = 'gl';
delete from src.names n where id = '00dsy9f04' and value = 'Ebsco information services' and lang_code = 'fr';
delete from src.names n where id = '00wge5k78' and value = 'Universitetet i Tromsø – Noregs arktiske universitet' and lang_code = 'nn';
delete from src.names n where id = '00wge5k78' and value = 'UiT Noregs arktiske universitet' and lang_code = 'nn';
delete from src.names n where id = '00yrf4e35' and value = 'Osservatorio Astronomico di Torino' and lang_code = 'de';
delete from src.names n where id = '00zb6nk96' and value = 'Labor Spiez' and lang_code = 'rm';
delete from src.names n where id = '01767d733' and value = 'Erasmushogeschool brussel' and lang_code = 'fr';
delete from src.names n where id = '01cdn3r29' and value = 'École des Beaux-Arts' and lang_code = 'ca';
delete from src.names n where id = '01kwczx50' and value = 'Skadden, arps, slate, meagher & flom' and lang_code = 'fr';
delete from src.names n where id = '02vc99v03' and value = 'Calbinotox' and lang_code = 'en';
delete from src.names n where id = '03v8adn41' and value = 'Queens college, city university of new york' and lang_code = 'fr';
delete from src.names n where id = '05c2g3729' and value = 'Ministarstvo vanjskih poslova' and lang_code = 'hr';
delete from src.names n where id = '05e0vkr08' and value = 'Bibliothèque nationale de Luxembourg' and lang_code = 'de';

-- redo dup table
drop table if exists src.dup_names_full;
create table src.dup_names_full 
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_full_idx on src.dup_names_full(id);
insert into src.dup_names_full (id, value, name_type, is_ror_name, lang_code)
    select d.id, n.value, n.name_type, n.is_ror_name, n.lang_code
    from (
        select id, lower(value) as lvalue from src.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join src.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;



delete from src.names n
using
	(select id, min(value) as min
	 from src.dup_names_full 
	 group by id) r
where n.id = r.id
and n.value = r.min


-- redo dup table
drop table if exists src.dup_names_full;
create table src.dup_names_full 
    (
          id                varchar     not null
        , value             varchar     not null  
        , name_type         int         null 
        , is_ror_name       bool        null
        , lang_code         varchar     null
    );
    create index dup_names_full_idx on src.dup_names_full(id);
insert into src.dup_names_full (id, value, name_type, is_ror_name, lang_code)
    select d.id, n.value, n.name_type, n.is_ror_name, n.lang_code
    from (
        select id, lower(value) as lvalue from src.names
        group by id, lower(value) having count(id) > 1
    ) d
    inner join src.names n
    on d.id = n.id
    and d.lvalue = lower(n.value)
    order by d.id;

-- if records still save...



fn get_dup_labels_with_same_lang_codes_and_diff_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, name_type, lang_code, dup_type)
    select id, lower(value), name_type, lang_code, '2 identical labels (diff is_ror_name)' 
    from src.names where name_type = 5
    group by id, lower(value), name_type, lang_code 
    having count(id) > 1;"#
}

fn get_dup_names_with_2_name_types_and_same_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, lang_code, dup_type, is_ror_name)
    select id, lower(value), lang_code, '2 name types', is_ror_name  from src.names
    group by id, lower(value), lang_code, is_ror_name 
    having count(id) > 1 ;"#
}

fn get_dup_names_with_2_lang_codes_and_same_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, name_type, dup_type, is_ror_name)
    select id, lower(value), name_type, '2 lang codes', is_ror_name from src.names
     group by id, lower(value), name_type, is_ror_name 
     having count(id) > 1;"#
}

fn get_dup_names_with_2_name_types_and_diff_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, lang_code, dup_type)
    select g.* from
        (select id, lower(value), lang_code, '2 name types + diff is_ror_name' from src.names
         group by id, lower(value), lang_code 
         having count(id) > 1) g
    left join src.dup_names dn
    on g.id = dn.id
    where dn.id is null;"#
}

fn get_dup_names_with_2_lang_codes_and_diff_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, name_type, dup_type)
    select g.* from
        (select id, lower(value), name_type, '2 lang codes + diff is_ror_name' from src.names
         group by id, lower(value), name_type 
         having count(id) > 1) g
    left join src.dup_names dn
    on g.id = dn.id
    where dn.id is null;"#
}

*/


/* 


fn get_update_2_name_types_dup_names_with_name_type_info_sql<'a>() -> &'a str {

    // Allows both name types to be visible in the record of each of the name type pair.

    r#"update src.dup_names dn
    set dup_type = adn.dup_type
    from 
        (select dn.id, dn.value, dn.dup_type||' '||string_agg(n.name_type::text, ', ' order by n.name_type) as dup_type
        from src.names n inner join src.dup_names dn
        on  dn.id = n.id and dn.value = n.value
        where dup_type like '2 name types'
        group by dn.id, dn.value, dn.dup_type) adn
    where dn.id = adn.id and dn.value = adn.value;"#
}

fn get_dups_with_2_name_types_5_and_7_to_be_deleted_sql<'a>() -> &'a str {

    // The alias is always to be deleted, leaving the label.

    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous name_type', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 name types 5, 7'
    and n.name_type = 7;"#
}  

fn get_dups_with_2_name_types_5_and_10_to_be_deleted_sql<'a>() -> &'a str {

    // The acronym is always to be deleted, leaving the label.

    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous name_type', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 name types 5, 10'
    and n.name_type = 10;"#
}  

fn get_dups_with_2_name_types_7_and_10_to_be_deleted_sql<'a>() -> &'a str {

    // The alias is always to be deleted, leaving the acronym (these values all appear to be acronyms).
    
    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous name_type', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 name types 7, 10'
    and n.name_type = 7;"#
}  


fn get_dups_with_label_pairs_and_diff_ror_status_to_be_deleted_sql <'a>() -> &'a str {

    // The non 'ror name' is selected for deletion 
    
    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous non ror', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 identical labels (diff is_ror_name)'
    and n.is_ror_name is false;"#
}


fn get_dups_with_2_name_types_and_diff_ror_status_to_be_deleted_sql <'a>() -> &'a str {

    // The non 'ror name' is selected for deletion whatever the name type.
    
    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous name_type', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 name types + diff is_ror_name'
    and n.is_ror_name is false;"#
}

fn get_dups_with_2_lang_codes_and_same_ror_status_to_be_deleted_sql <'a>() -> &'a str {

    // The selection of codes to delete was manually determined as being the most 
    // reasonable, given organisation locations, but are arbitrary.
    
    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous lang_code', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 lang codes'
    and n.lang_code in ('gl', 'nn', 'rm', 'en', 'hr');"#
}

fn get_dups_with_2_lang_codes_and_diff_ror_status_to_be_deleted_sql <'a>() -> &'a str {

    // The non 'ror name' is selected for deletion whatever the lang code.
    
    r#"insert into src.dup_names_deleted (id, value, name_type, dup_type, is_ror_name, lang_code)
    select n.id, n.value, n.name_type, 'superfluous lang_code', n.is_ror_name, n.lang_code
    from src.dup_names dn
    inner join src.names n
    on dn.id = n.id and dn.value = n.value
    where dup_type like '2 lang codes + diff is_ror_name'
    and n.is_ror_name is false;"#
}

fn get_delete_names_with_superfluous_non_ror_sql <'a>() -> &'a str {
    r#"delete from src.names n
    using src.dup_names_deleted dnd
    where dup_type = 'superfluous non ror'
    and n.id = dnd.id
    and n.value = dnd.value
    and n.name_type = dnd.name_type
    and n.is_ror_name is false;"#
}

fn get_delete_names_with_superfluous_name_type_sql <'a>() -> &'a str {
    r#"delete from src.names n
    using src.dup_names_deleted dnd
    where dup_type = 'superfluous name_type'
    and n.id = dnd.id
    and n.value = dnd.value
    and n.name_type = dnd.name_type;"#
}

fn get_delete_names_with_superfluous_lang_code_sql <'a>() -> &'a str {
    r#"delete from src.names n
    using src.dup_names_deleted dnd
    where dup_type = 'superfluous lang_code'
    and n.id = dnd.id
    and n.value = dnd.value
    and n.lang_code = dnd.lang_code;"#
}

    */

fn replace_deprecated_lang_code_sql <'a>() -> &'a str {
    r#"update src.names 
    set lang_code = 'sr'
    where lang_code = 'sh';"#
}
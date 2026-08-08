use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn remove_dups (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Before further processing the duplicate names need to be removed from rec.names. 
    // If this is not done the import to the core data, that follows, will fail, 
    // as some organisations have more than one name marked as the 'ror name' (the 
    // import therefore fails because of a duplicated PK). 

    // The duplicates, as found at the beginning of the process, are stored in rec.dup_names

    // No organisation seems to have - in the source file - two names that are exactly the same in 
    // all respects  - i.e. have the same value, name type, language code and 'is_ror_name' status.
    // This is possible and should be guarded against but does not seem to occur.
    
    // More commonly, duplicates exist where an organisation has names with the same (lower case) name values
    // and lang code, but differ on the name type, or have the same value and name type, but differ 
    // on the language code. Both groups include pairs with the same 'is_ror_name' value, and 
    // pairs with the opposite 'is_ror_name' values.

    // First set up the table of names that are duplicated (same ror id and name value).
        
    let res =  create_duplicates_table(pool).await?;  
    info!("{} Duplicate name pairs identified", res / 2) ;
  
    // Use a 'scratch pad' table, src.dups, to hold duplicate pairs - reduces in size as
    // process drops duplicates and the table is reformed.

    //recreate_dups(pool).await?;  

    // Drop names that are the non-ror equivalents of ror names.

    let res = drop_non_ror_name_dups(pool).await?;
    info!("{} names dropped that are the non-ror equivalents of ror names", res);

    //recreate_dups(pool).await?;  

    // Drop names that are the alias equivalents of labels

    let res = drop_alias_dups(pool).await?;
    info!("{} names dropped that are the alias equivalents of labels", res);

    // Drop names that are one of an acronym - other name pair

    // recreate_dups(pool).await?;  

    let res = drop_acro_dups(pool).await?;
    info!("{} names dropped from acronym - other name pairs", res);
    
    // Drop some specific errors using code for indivdual names

    let res = drop_specific_dups(pool).await?;
    info!("{} names dropped using name specific code to target them", res);

    // recreate_dups(pool).await?;  

    // Drop the names with the lowest id in the remainder that are left

    let res = drop_lowest_ident_dups(pool).await?;
    info!("{} names dropped using the lowest Ident in the remaining duplicates", res);

    execute_sql(replace_deprecated_lang_code_sql(), pool).await?;  
    
    info!(""); 
    Ok(())
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {

    sqlx::raw_sql(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
    
        //sqlx::query(&sql).execute(pool)
        //.await
        //.map_err(|e| AppError::SqlxError(e, sql.to_string()))
   
}

async fn create_duplicates_table(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    // Table recreated after each duplicate drop operatrion -  i.e.
    // has the current (but diminishing) duplicate name orgs
    
    let sql = r#"SET client_min_messages TO WARNING; 
        drop table if exists rec.dups;
        create table rec.dups 
        (  
    	  ident             int         not null  primary key
    	, id                varchar     not null
    	, value             varchar     not null  
    	, name_type         int         null 
    	, is_ror_name       bool        null
    	, lang_code         varchar     null
    	, dealt_with        bool        default false
        );
        create index dup_ids_idx on rec.dups(id);"#;
    
        execute_sql(sql, pool).await?;
        
    let sql = r#"insert into rec.dups (ident, id, value, name_type, is_ror_name, lang_code)
        select n.ident, d.id, n.display_name, n.name_type, n.is_ror_name, n.lang
        from (
            select id, lower(display_name) as lvalue from rec.names
            group by id, lower(display_name) having count(id) > 1
        ) d
        inner join rec.names n
        on d.id = n.id
        and d.lvalue = lower(n.display_name)
        order by d.id;"#;
    
        let res = execute_sql(sql, pool).await?.rows_affected();
        Ok(res)
}


async fn drop_non_ror_name_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let drop_reason = "DROPPED because of non-ror status when ror equivalent present";

    let sql = r#"SET client_min_messages TO WARNING; 
        drop table if exists rec.ror_nonror;
        create table rec.ror_nonror as
        select f.ident as non_ror_ident, t.ident as ror_ident
        from 
       	    (select * from rec.dups
           	where is_ror_name = true) t
            inner join
           	(select * from rec.dups
           	where is_ror_name = false) f
            on t.id = f.id
            and lower(t.value) = lower(f.value);"#;
    
    execute_sql(sql, pool).await?;
    
    let sql = format!(r#"update rec.names n
        set changed = true,
        change_type_id = case when change_type_id is null then 'XX1'
    	else change_type_id||', XX1'
        end,
        change_type = 
    	case when change_type is null then '{drop_reason}'
    	else change_type||', '||'{drop_reason}'
        end 
        from rec.ror_nonror x
        where n.ident = x.non_ror_ident;"#);
    
    let res = execute_sql(&sql, pool).await?.rows_affected();
    
    let sql = r#"update rec.dups d
        set dealt_with = true
        from rec.ror_nonror x
        where d.ident = x.ror_ident
        or d.ident = x.non_ror_ident;
        drop table rec.ror_nonror;"#;

    execute_sql(sql, pool).await?;
    Ok(res)
}


async fn drop_alias_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    let drop_reason = "DROPPED because an alias when equivalent label present";

    let sql = r#"SET client_min_messages TO WARNING; 
        drop table if exists rec.alias_label;
        create table rec.alias_label as
        select s.ident as alias_ident, b.ident as label_ident
        from 
    	(select * from rec.dups
    		where name_type = 7
    		and dealt_with = false) s
    	inner join
    		(select * from rec.dups
    		where name_type = 5
    		and dealt_with = false) b
    	on s.id = b.id
    	and lower(s.value) = lower(b.value);"#;
    
    execute_sql(sql, pool).await?;

    let sql = format!(r#"update rec.names n
        set changed = true,
        change_type_id = case when change_type_id is null then 'XX2'
    	else change_type_id||', XX2'
        end,
        change_type = 
    	case when change_type is null then '{drop_reason}'
    	else change_type||', '||'{drop_reason}'
        end 
        from rec.alias_label x
        where n.ident = x.alias_ident;"#);
    
    let res = execute_sql(&sql, pool).await?.rows_affected();

    let sql = r#"update rec.dups d
        set dealt_with = true
        from rec.alias_label x
        where d.ident = x.alias_ident
        or d.ident = x.label_ident;
        drop table rec.alias_label;"#;

    execute_sql(sql, pool).await?;
    Ok(res)
}

async fn drop_acro_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    // N.B. Names >= 5 in length are viewed as non-acronyms.
    
    let sql = r#"SET client_min_messages TO WARNING; 
        drop table if exists rec.acro_nonacro;
        create table rec.acro_nonacro as
        select b.ident as nonacro_ident, a.ident as acro_ident, 
                length(a.value) as name_length
        from 
       	(select * from rec.dups
      		where name_type <> 10
      		and dealt_with = false) b
       	inner join
      		(select * from rec.dups
      		where name_type = 10
      		and dealt_with = false) a
       	on b.id = a.id
       	and lower(b.value) = lower(a.value);"#;
       
    execute_sql(sql, pool).await?;

    let drop_reason = "DROPPED because an acronym when equivalent alias or label present";
    
    let sql = format!(r#"update rec.names n
        set changed = true,
        change_type_id = case when change_type_id is null then 'XX3'
    	else change_type_id||', XX3'
        end,
        change_type = 
    	case when change_type is null then '{drop_reason}'
    	else change_type||', '||'{drop_reason}'
        end 
        from rec.acro_nonacro x
        where n.ident = x.acro_ident
        and x.name_length > 5;"#);

    let res1 = execute_sql(&sql, pool).await?.rows_affected();

    let drop_reason = "DROPPED because an alias or label when equivalent acronym present";
    
    let sql = format!(r#"update rec.names n
        set changed = true,
        change_type_id = case when change_type_id is null then 'XX4'
    	else change_type_id||', XX4'
        end,
        change_type = 
    	case when change_type is null then '{drop_reason}'
    	else change_type||', '||'{drop_reason}'
        end 
        from rec.acro_nonacro x
        where n.ident = x.nonacro_ident
        and x.name_length <= 5;"#);

    let res2 = execute_sql(&sql, pool).await?.rows_affected();

    let sql = r#"update rec.dups d
        set dealt_with = true
        from rec.acro_nonacro x
        where d.ident = x.acro_ident
        or d.ident = x.nonacro_ident;
        drop table rec.acro_nonacro;
        SET client_min_messages TO NOTICE;"#;

    execute_sql(sql, pool).await?;
    Ok(res1 + res2)
}

async fn drop_specific_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    // These specific drops included after manual inspection of pairs 
    // (not already dealt witgh above automatically).
    
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

    let drop_reason = "DROPPED using a specific call for this name / language code";
    
    let sql = format!(r#"update rec.names 
        set changed = true,
        change_type_id = case when change_type_id is null then 'XX5'
    	else change_type_id||', XX5'
        end,
        change_type = 
    	case when change_type is null then '{drop_reason}'
    	else change_type||', '||'{drop_reason}'
        end 
        where id = '{}' and display_name = '{}' and lang = '{}';"#, id, name, lang);
    execute_sql(&sql, pool).await?;

    let sql = format!("update rec.dups d
        set dealt_with = true
        where id = '{}' and value = '{}' and lang_code = '{}';", id, name, lang);
    let res = execute_sql(&sql, pool).await?.rows_affected();
   
    Ok(res)
}


async fn drop_lowest_ident_dups(pool: &Pool<Postgres>) -> Result<u64, AppError> {

    // Final 'catch all' drop mechanism. On an arbitrary basis  the duplicate with 
    // the lowest Id - all other fields being equal.
    // 
    let drop_reason = "DROPPED ecause the lower ident, other fields being equivalent";
    
    let sql = format!(r#"update rec.names d
        set changed = true,
        change_type_id = case when change_type_id is null then '995'
    	else change_type_id||', XX6'
        end,
        change_type = 
    	case when change_type is null then '{drop_reason}'
    	else change_type||', '||'{drop_reason}'
        end 
        from 
           (select id, min(ident) as min
            from rec.dups 
            where dealt_with = false
            group by id) r
        where d.ident = r.min;"#);
    
    let res = execute_sql(&sql, pool).await?.rows_affected();

    Ok(res)
}
    

fn replace_deprecated_lang_code_sql <'a>() -> &'a str {
    r#"update ppr.names 
    set lang_code = 'sr'
    where lang_code = 'sh';"#
}
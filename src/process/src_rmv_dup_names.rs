use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use log::info;
use crate::AppError;

pub async fn remove_dups (pool: &Pool<Postgres>) -> Result<(), AppError> {
    
    // Before further processing the duplicate names need to be removed from src.names. 
    // If this is not done the import to the core data, that follows, will fail, 
    // as some organisations have more than one name marked as the 'ror name' (the 
    // import therefore fails because of a duplicated PK). 
    // The duplicates are stored separately in src.dup_names, which 
    // also indicates the nature of the duplication, and the records deleted in 
    // src.dup_names_deleted.

    // No organisation seems to have - in the source file - two names that have the same value, 
    // name type and language code, though in at least one case the fact that 'naked' ror names 
    // (i.e. without any associated type) are given a 'label' type means that an organisation
    // can have two labels with the same language if one of the original names was a label
    // with the same value as the 'naked' ror name.

    // More commonly, duplicates exist where an organisation has names with the same value 
    // and lang code, but differ on the name type, or have the same value and name type, but differ 
    // on the language code. Both groups include pairs with the same 'is_ror_name' value, and 
    // pairs with the opposite 'is_ror_name' values.

    // First set up the table of names that are duplicated (same ror id and name value).
    // Four different sql statements are needed, given the different types of duplication.
    
    execute_sql(get_dup_labels_with_same_lang_codes_and_diff_ror_status_sql(), pool).await?;                           // group 0
    execute_sql(get_dup_names_with_2_name_types_and_same_ror_status_sql(), pool).await?;                               // group 1
    execute_sql(get_dup_names_with_2_lang_codes_and_same_ror_status_sql(), pool).await?;                               // group 2
    execute_sql(get_dup_names_with_2_name_types_and_diff_ror_status_sql(), pool).await?;           // group 3
    execute_sql(get_dup_names_with_2_lang_codes_and_diff_ror_status_sql(), pool).await?;           // group 4

    info!("Duplicates names identified");

    // Then, for each group, identify the records to be deleted and add then to the 
    // src.dup_names_deleted table

    execute_sql(get_update_2_name_types_dup_names_with_name_type_info_sql(), pool).await?;         // initial clarification of types involved
    execute_sql(get_dups_with_label_pairs_and_diff_ror_status_to_be_deleted_sql(), pool).await?;   // group 0 - manufactured label pairs 
    execute_sql(get_dups_with_2_name_types_5_and_7_to_be_deleted_sql(), pool).await?;              // group 1 (label + alias)
    execute_sql(get_dups_with_2_name_types_5_and_10_to_be_deleted_sql(), pool).await?;             // group 1 (label + acronym)
    execute_sql(get_dups_with_2_name_types_7_and_10_to_be_deleted_sql(), pool).await?;             // group 1 (alias + acronym)
    execute_sql(get_dups_with_2_name_types_and_diff_ror_status_to_be_deleted_sql(), pool).await?;  // group 3
    execute_sql(get_dups_with_2_lang_codes_and_same_ror_status_to_be_deleted_sql(), pool).await?;  // group 2
    execute_sql(get_dups_with_2_lang_codes_and_diff_ror_status_to_be_deleted_sql(), pool).await?;  // group 4

    // Finally, use the records in src.dup_names_deleted to delete the specified
    // records from src.names;

    execute_sql(get_delete_names_with_superfluous_non_ror_sql(), pool).await?;
    execute_sql(get_delete_names_with_superfluous_name_type_sql(), pool).await?;
    execute_sql(get_delete_names_with_superfluous_lang_code_sql(), pool).await?;
    execute_sql(replace_deprecated_lang_code_sql(), pool).await?;

    info!("Duplicates transferred from src.names to src.dup_names_deleted table");
    Ok(())
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
    
    sqlx::query(&sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))
   
}

fn get_dup_labels_with_same_lang_codes_and_diff_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, name_type, lang_code, dup_type)
    select id, value, name_type, lang_code, '2 identical labels (diff is_ror_name)' 
    from src.names where name_type = 5
    group by id, value, name_type, lang_code having count(id) > 1;"#
}

fn get_dup_names_with_2_name_types_and_same_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, lang_code, dup_type, is_ror_name)
    select id, value, lang_code, '2 name types', is_ror_name  from src.names
    group by id, value, lang_code, is_ror_name having count(id) > 1 ;"#
}

fn get_dup_names_with_2_lang_codes_and_same_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, name_type, dup_type, is_ror_name)
    select id, value, name_type, '2 lang codes', is_ror_name from src.names
     group by id, value, name_type, is_ror_name having count(id) > 1;"#
}

fn get_dup_names_with_2_name_types_and_diff_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, lang_code, dup_type)
    select g.* from
        (select id, value, lang_code, '2 name types + diff is_ror_name' from src.names
        group by id, value, lang_code having count(id) > 1) g
    left join src.dup_names dn
    on g.id = dn.id
    where dn.id is null;"#
}

fn get_dup_names_with_2_lang_codes_and_diff_ror_status_sql <'a>() -> &'a str {
    r#"insert into src.dup_names (id, value, name_type, dup_type)
    select g.* from
        (select id, value, name_type, '2 lang codes + diff is_ror_name' from src.names
        group by id, value, name_type having count(id) > 1) g
    left join src.dup_names dn
    on g.id = dn.id
    where dn.id is null;"#
}

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

fn replace_deprecated_lang_code_sql <'a>() -> &'a str {
    r#"update src.names 
    set lang_code = 'sr'
    where lang_code = 'sh';"#
}
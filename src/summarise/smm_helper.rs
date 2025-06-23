use sqlx::{Pool, Postgres};
use sqlx::postgres::PgQueryResult;
use crate::AppError;
use super::smm_structs::{DistribRow, RankedRow, TypeRow, OrgRow};

pub async fn delete_any_existing_data(vcode: &String, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {
   
    // format!() macro does not seem to recognise apostrophes, even when escaped (???)

    let wc = " WHERE vcode = \'".to_string() + vcode + "\'; ";
        
    let del_sql = format!(r#"DELETE from smm.version_summaries {}
                DELETE from smm.attributes_summary {}
                DELETE from smm.count_distributions {}
                DELETE from smm.ranked_distributions {}
                DELETE from smm.singletons {}
                DELETE from smm.org_type_and_lang_code {}
                DELETE from smm.org_type_and_relationships {}"#
                , wc, wc, wc, wc, wc, wc, wc);

   sqlx::raw_sql(&del_sql).execute(pool).await
         .map_err(|e| AppError::SqlxError(e, del_sql))
}


pub async fn create_name_attributes(sdv: &str, vcode: &String, num_orgs_str: &String, num_names: &String, 
    pool: &Pool<Postgres>) ->  Result<PgQueryResult, AppError> {

    // Name attributes summary     

    let sql  = r#"select * from
            ("#.to_string() + sdv + r#"rn.id, rn.name, count(t.id) as number_atts, 0::float as pc_of_atts, 
            count(distinct t.id) as number_orgs, 0::float as pc_of_orgs
            from lup.ror_name_types rn
            inner join src.names t
            on rn.id = t.name_type 
            group by rn.id, rn.name
            order by rn.id) a
            union
            ("# + sdv + r#"12, 'nacro', sum(n_nacro), 0::float, count(id), 0::float
            from src.admin_data t where n_nacro > 0) 
            union 
            ("# + sdv + r#"22, 'nacro (excl. cmps)', sum(n_nacro), 0::float, count(id), 0::float
            from src.admin_data t where n_nacro > 0 and is_company = false)
            order by id"#;
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
             .map_err(|e| AppError::SqlxError(e, sql))?;
    store_summary(rows, pool, 1, "name types").await?;

    let sql  = r#"select * from
            ("#.to_string() + sdv + r#"rn.id + 100 as id, rn.name||'_wolc'as name, 
            count(t.id) as number_atts, 0::float as pc_of_atts, 
            count(distinct t.id) as number_orgs, 0::float as pc_of_orgs
            from lup.ror_name_types rn
            inner join src.names t
            on rn.id = t.name_type 
            where t.lang_code is null
            group by rn.id, rn.name
            order by rn.id)
            union
            ("# + sdv + r#"112, 'nacro_wolc', sum(n_nacro_wolc), 0::float, count(id), 0::float
            from src.admin_data t where n_nacro_wolc > 0) 
            union 
            ("# + sdv + r#"122, 'nacro_wolc (excl. cmps)', sum(n_nacro_wolc), 0::float, count(id), 0::float
            from src.admin_data t where n_nacro_wolc > 0 and is_company = false) 
            order by id"#;
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
            .map_err(|e| AppError::SqlxError(e, sql))?;
    store_summary(rows, pool, 11, "name types wolc").await?;


    let sql  = "".to_string() + r#"Update smm.attributes_summary set 
            pc_of_atts = round(number_atts * 10000::float / "# + num_names + r#"::float)/100::float,
            pc_of_orgs = round(number_orgs * 10000::float / "# + num_orgs_str + r#"::float)/100::float
            where vcode = '"# + vcode + r#"' and att_type in (1, 11) "#;
    sqlx::raw_sql(&sql).execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql))
}


pub async fn create_other_attributes(sdv: &str, num_orgs_str: &String, num_types: &String, 
num_ext_ids: &String, num_links: &String, num_rels: &String, pool: &Pool<Postgres>) ->  Result<(), AppError> {
    
    // Org type attributes summary

    let sql  = sdv.to_owned() + r#"gt.id, gt.name, count(t.id) as number_atts, 
            round(count(t.id) * 10000::float/"# + num_types + r#"::float)/100::float as pc_of_atts,
            count(distinct t.id) as number_orgs,
            round(count(distinct t.id) * 10000::float/"# + num_orgs_str + r#"::float)/100::float as pc_of_orgs
            from lup.ror_org_types gt
            inner join src.type t
            on gt.id = t.org_type 
            group by gt.id, gt.name
            order by gt.id;"#;
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_summary(rows, pool, 2, "org types").await?;

    // External ids attributes summary

    let sql = sdv.to_owned() + r#"it.id, it.name, count(t.id) as number_atts, 
            round(count(t.id) * 10000::float / "# + num_ext_ids + r#"::float)/100::float as pc_of_atts,
            count(distinct t.id) as number_orgs,
            round(count(distinct t.id) * 10000::float / "# + num_orgs_str + r#"::float)/100::float as pc_of_orgs
            from lup.ror_id_types it
            inner join src.external_ids t
            on it.id = t.id_type 
            group by it.id, it.name
            order by it.id;"#;
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_summary(rows, pool, 3, "external id types").await?;

    // Links attributes summary

    let sql = sdv.to_owned() + r#"lt.id, lt.name, count(t.id) as number_atts, 
            round(count(t.id) * 10000::float / "# + num_links + r#"::float)/100::float as pc_of_atts,
            count(distinct t.id) as number_orgs,
            round(count(distinct t.id) * 10000::float / "# + num_orgs_str + r#"::float)/100::float as pc_of_orgs
            from lup.ror_link_types lt
            inner join src.links t
            on lt.id = t.link_type 
            group by lt.id, lt.name
            order by lt.id;"#;
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_summary(rows, pool, 4, "link types").await?;

    // Relationships attributes summary

    let sql = sdv.to_owned() + r#"rr.id, rr.name, count(t.id) as number_atts, 
            round(count(t.id) * 10000::float / "# + num_rels + r#"::float)/100::float as pc_of_atts,
            count(distinct t.id) as number_orgs,
            round(count(distinct t.id) * 10000::float / "# + num_orgs_str + r#"::float)/100::float as pc_of_orgs
            from lup.ror_rel_types rr
            inner join src.relationships t
            on rr.id = t.rel_type 
            group by rr.id, rr.name
            order by rr.id;"#;
    let rows: Vec<TypeRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_summary(rows, pool, 5, "rel types").await?;

    Ok(())
}


pub async fn create_count_distributions(sdv: &str, num_orgs_str: &String, pool: &Pool<Postgres>) ->  Result<(), AppError> {

    // All names count distribution

    let sql = sdv.to_owned() + r#"n_names as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_names
            order by n_names;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "names", pool).await?;

    // Labels count distribution

    let sql = sdv.to_owned() + r#"n_labels as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_labels
            order by n_labels;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "labels", pool).await?;

    // Aliases count distribution

    let sql = sdv.to_owned() + r#"n_aliases as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_aliases
            order by n_aliases;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "aliases", pool).await?;

    // Acronyms count distribution

    let sql = sdv.to_owned() + r#"n_acronyms as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_acronyms
            order by n_acronyms;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "acronyms", pool).await?;

    // Locations count distribution

    let sql = sdv.to_owned() + r#"n_locs as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_locs
            order by n_locs;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "locs", pool).await?;

    // Org types count distribution

    let sql = sdv.to_owned() + r#"n_types as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_types
            order by n_types;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "org_types", pool).await?;

    // External ids count distribution

    let sql = sdv.to_owned() + r#"n_ext_ids as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_ext_ids
            order by n_ext_ids;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "ext_ids", pool).await?;

    // Links count distribution

    let sql = sdv.to_owned() + r#"n_links as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_links
            order by n_links;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "links", pool).await?;

    // Parent count distribution

    let sql = sdv.to_owned() + r#"n_parrels as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_parrels
            order by n_parrels;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "parent orgs", pool).await?;

    // Child count distribution

    let sql = sdv.to_owned() + r#"n_chrels as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_chrels
            order by n_chrels;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "child orgs", pool).await?;

    // Related orgs

    let sql = sdv.to_owned() + r#"n_relrels as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_relrels
            order by n_relrels;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "related orgs", pool).await?;

    // Successor count distribution

    let sql = sdv.to_owned() + r#"n_sucrels as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_sucrels
            order by n_sucrels;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "successor orgs", pool).await?;

    // Predeccessor orgs

    let sql = sdv.to_owned() + r#"n_predrels as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_predrels
            order by n_predrels;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "predecessor orgs", pool).await?;


    // Domains count distribution

    let sql = sdv.to_owned() + r#"n_doms as count, count(id) as num_of_orgs, 
            round(count(id) * 10000 :: float / "# + num_orgs_str + r#":: float)/100 :: float as pc_of_orgs
            from src.admin_data
            group by n_doms
            order by n_doms;"#;
    let rows: Vec<DistribRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_distrib(rows, "domains", pool).await?;

    Ok(())
    
}

pub async fn create_ranked_count_distributions(vcode: &String, sdv: &str, num_names: i64, 
num_locs: i64, pool: &Pool<Postgres>) ->  Result<(), AppError> {

    // Non-English language ranked distribution (non-acronym names only)
    let num_nacro = get_count("select count(*) from src.names where name_type <> 10", pool).await?;
    let num_nacne = get_count("select count(*) from src.names where name_type <> 10 and lang_code <> 'en'", pool).await?;
    let sql = sdv.to_owned() + r#"lc.name as entity, count(n.id) as number,
            round(count(n.id) * 10000 :: float / "# + &num_nacne.to_string() + r#":: float)/100 :: float as pc_of_entities,
            round(count(distinct n.id) * 10000 :: float / "# + &(num_nacro.to_string()) + r#":: float)/100 :: float as pc_of_base_set
            from src.names n inner join lup.lang_codes lc 
            on n.lang_code = lc.code 
            where name_type <> 10 and lang_code <> 'en'
            group by lc.name
            order by count(n.id) desc;"#;
    let rows: Vec<RankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
            .map_err(|e| AppError::SqlxError(e, sql))?;
    store_ranked_distrib(&vcode, &rows, pool, "Remaining languages", 1, 
    num_nacne, num_names).await?;

    // Non-Latin script ranked distribution
    let num_nltn = get_count("select count(*) from src.names where script_code <> 'Latn'", pool).await?;
    let sql = sdv.to_owned() + r#"ls.iso_name as entity, count(n.id) as number,
            round(count(n.id) * 10000 :: float / "# + &num_nltn.to_string() + r#":: float)/100 :: float as pc_of_entities,
            round(count(distinct n.id) * 10000 :: float / "# + &(num_names.to_string()) + r#":: float)/100 :: float as pc_of_base_set
            from src.names n inner join lup.lang_scripts ls 
            on n.script_code = ls.code 
            where script_code <> 'Latn'
            group by ls.iso_name
            order by count(n.id) desc; "#;
    let rows: Vec<RankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_ranked_distrib(&vcode, &rows, pool, "Remaining scripts", 2,
    num_nltn, num_names).await?;

    // Country ranked distribution (and non US pc)

    let num_nus = get_count("select count(*) from src.locations where country_code <> 'US'", pool).await?;
    let sql = sdv.to_owned() + r#"country_name as entity, count(id) as number, 
            round(count(c.id) * 10000 :: float / "# + &num_nus.to_string() + r#":: float)/100 :: float as pc_of_entities,
            round(count(distinct c.id) * 10000 :: float / "# + &(num_locs.to_string()) + r#":: float)/100 :: float as pc_of_base_set
            from src.locations c
            group by country_name
            order by count(country_name) desc;"#;
    let rows: Vec<RankedRow> = sqlx::query_as(&sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, sql))?;
    store_ranked_distrib(&vcode, &rows, pool, "Remaining countries", 3,
    num_nus, num_locs).await?;

    Ok(())
}


pub async fn create_type_linked_tables(sdv: &str, pool: &Pool<Postgres>) ->  Result<(), AppError> {

    // Get the organisation type categories and total numbers.

    let org_sql  = r#"select org_type as type_id, p.name, 
            count(distinct t.id) as org_num
            from src.type t
            inner join lup.ror_org_types p
            on t.org_type = p.id
            group by org_type, p.name
            order by org_type;"#;
    let rows: Vec<OrgRow> = sqlx::query_as(org_sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, org_sql.to_string()))?;
    store_types_with_lang_code(sdv, rows, pool).await?;

    let rows: Vec<OrgRow> = sqlx::query_as(org_sql).fetch_all(pool).await
        .map_err(|e| AppError::SqlxError(e, org_sql.to_string()))?;
    store_types_and_relationships(sdv, rows, pool).await?;

    Ok(())
}


pub async fn store_types_with_lang_code(sdv: &str, rows: Vec<OrgRow>, pool: &Pool<Postgres>) -> Result<(), AppError> {

    // For each org type, and each of the three name types (therefore 9 x 3 rows),
    // get total number of names and numbers with / without lang codes.

    #[derive(sqlx::FromRow)]
    struct NameLCRow {
        vcode: String,
        ntype: String,
        total: i64,
        names_wlc: i64,
        names_wolc: i64,
        names_wlc_pc: f64,
        names_wolc_pc: f64
    }
   
    for t in rows {

        // Get the data on the names linked to these organisations

        let lc_sql = sdv.to_owned() + r#"case name_type
                    when 5 then 'label' 
                    when 7 then 'alias'  
                    when 10 then 'acronym' 
                end as ntype, 
                count(lc) as names_wlc, count(nolc) as names_wolc, count(lc) + count(nolc) as total,
                (round(count(lc) * 10000::float / (count(lc) + count(nolc))::float)/100::float) as names_wlc_pc,
                (round(count(nolc) * 10000::float / (count(lc) + count(nolc))::float)/100::float) as names_wolc_pc
                from
                    (select n.id, n.name_type,
                    case 
                        when n.lang_code is not null then 'x'
                    end as lc, 
                    case 
                        when n.lang_code is  null then 'x'
                    end as nolc
                    from src.names n 
                    inner join src.type t
                    on n.id = t.id
                    where t.org_type = "# + &t.type_id.to_string() + r#") ns
                group by ns.name_type 
                order by ns.name_type;"#;
        let rows: Vec<NameLCRow> = sqlx::query_as(&lc_sql).fetch_all(pool).await        
            .map_err(|e| AppError::SqlxError(e, lc_sql))?;

        // Store the individual rows.

        for r in rows {
            let sql = r#"INSERT INTO smm.org_type_and_lang_code (vcode, org_type, name_type, 
                        names_num, names_wlc, names_wolc, names_wlc_pc, names_wolc_pc) 
                        values($1, $2, $3, $4, $5, $6, $7, $8)"#;
            sqlx::query(sql)
            .bind(r.vcode).bind(t.name.clone()).bind(r.ntype).bind(r.total)
            .bind(r.names_wlc).bind(r.names_wolc).bind(r.names_wlc_pc).bind(r.names_wolc_pc)
            .execute(pool)
            .await        
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        }
    }

    Ok(())
}


pub async fn store_types_and_relationships(sdv: &str, rows: Vec<OrgRow>, pool: &Pool<Postgres>) -> Result<(), AppError> {

    // For each org type, and each of the 5 relationship types (therefore up to 9 x 5 rows),
    // get number of orgs having one or more relationships of each type, and the total number of orgs involved.

    #[derive(sqlx::FromRow)]
    struct TypeRelRow {
        vcode: String,
        rtype: String,
        num_rels: i64,
        num_orgs: i64,
        num_orgs_pc: f64,
    }

    for t in rows {

        // Get the data on the names linked to these organisations

        let tr_sql = sdv.to_owned() + r#"case rs.rel_type
                when 1 then 'has parent'  
                when 2 then 'has child' 
                when 3 then 'is related to' 
                when 4 then 'has predecessor' 
                when 5 then 'has successor' 
                end as rtype, 
            count(rs.id) as num_rels,
            count(distinct rs.id) as num_orgs,
            round((count(distinct rs.id) * 10000::float /"#
                + &t.org_num.to_string() + r#"::float)) /100::float as num_orgs_pc
            from
                (select r.id, r.rel_type
                from src.relationships r 
                inner join src.type t
                on r.id = t.id
                where t.org_type ="# + &t.type_id.to_string() + r#") rs 
            group by rs.rel_type 
            order by rs.rel_type;"#;

        let rows: Vec<TypeRelRow> = sqlx::query_as(&tr_sql).fetch_all(pool).await 
            .map_err(|e| AppError::SqlxError(e, tr_sql))?;

        // Store the individual rows.

        for r in rows {
            let sql = r#"INSERT INTO smm.org_type_and_relationships (vcode, org_type, 
                    rel_type, num_links, num_orgs, num_orgs_total, num_orgs_pc) 
                    values($1, $2, $3, $4, $5, $6, $7)"#;
            sqlx::query(sql)
            .bind(r.vcode).bind(t.name.clone()).bind(r.rtype).bind(r.num_rels)
            .bind(r.num_orgs).bind(t.org_num).bind(r.num_orgs_pc)
            .execute(pool)
            .await 
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        }
    }

    Ok(())
}


pub async fn store_singletons(vcode: &String, num_orgs: i64, num_names: i64, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let num_acro = get_count("select count(*) from src.names where name_type = 10", pool).await?;
    let num_nacro = get_count("select count(*) from src.names where name_type <> 10", pool).await?;

    // Duplicated names that have been removed

    let num_duplicated_names = get_count("select count(id) from src.dup_names_deleted", pool).await?;
    let pc_dup = get_pc (num_duplicated_names, num_names);  
    store_singleton(vcode, "dup_names", "Duplicated names removed, number & pc of total names", 
    num_duplicated_names, Some(pc_dup), pool).await?;

    // Names without a language code

    let total_wolc = get_count("select count(*) from src.names where lang_code is null", pool).await?;
    let pc_total_wolc = get_pc (total_wolc, num_names);
    store_singleton(vcode, "total_wolc", "Names that are wolc, number & pc of total names",  
                        total_wolc, Some(pc_total_wolc), pool).await?;

    let nacro_wolc = get_count("select count(*) from src.names where name_type <> 10 and lang_code is null", pool).await?;
    let pc_nacro_wolc =  get_pc (nacro_wolc, num_nacro);
    store_singleton(vcode, "nacro_wolc", "Nacro names wolc, number and pc of nacro names",  
    nacro_wolc, Some(pc_nacro_wolc), pool).await?;

    let nacro_ncmp_wolc = get_count(r#"select count(n.id) from 
                    src.names n
                    inner join src.admin_data ad
                    on n.id = ad.id 
                    where n.name_type <> 10 and ad.is_company = false
                    and n.lang_code is null"#, pool).await?;   
    
    let num_ncmp_names = get_count(r#"select count(n.id) from 
                    src.names n
                    inner join src.admin_data ad
                    on n.id = ad.id 
                    where n.name_type <> 10 and ad.is_company = false"#, pool).await?; 
    let pc_nacro_ncmp_wolc =  get_pc (nacro_ncmp_wolc, num_ncmp_names);
    store_singleton(vcode, "nacncmp_wolc", "Nac-ncmp names wolc, number and pc of nac-ncmp names",  
    nacro_ncmp_wolc, Some(pc_nacro_ncmp_wolc), pool).await?;
  
    // Names not in English or not in Latin script

    let num_names_ne = get_count("select count(*) from src.names where lang_code <> 'en'", pool).await?;
    let num_acro_ne = get_count("select count(*) from src.names where name_type = 10 and lang_code <> 'en'", pool).await?;
    let num_nacro_ne = get_count("select count(*) from src.names where name_type <> 10 and lang_code <> 'en'", pool).await?;

    let num_names_nl = get_count("select count(*) from src.names where script_code <> 'Latn'", pool).await?;
    let num_acro_nl = get_count("select count(*) from src.names where name_type = 10 and script_code <> 'Latn'", pool).await?;
    let num_nacro_nl = get_count("select count(*) from src.names where name_type <> 10 and script_code <> 'Latn'", pool).await?;

    let pc_names_ne = get_pc (num_names_ne, num_names);  
    let pc_acro_ne = get_pc (num_acro_ne, num_acro);   
    let pc_nacro_ne = get_pc (num_nacro_ne, num_nacro);  

    let pc_names_nl = get_pc (num_names_nl, num_names);  
    let pc_acro_nl = get_pc (num_acro_nl, num_acro);  
    let pc_nacro_nl = get_pc (num_nacro_nl, num_nacro);  

    store_singleton(vcode, "names_ne", "Names not in English, number & pc of names",  
                    num_names_ne, Some(pc_names_ne), pool).await?;
    store_singleton(vcode, "acro_ne", "Acronyms not in English, number & pc of acronyms",  
                    num_acro_ne, Some(pc_acro_ne), pool).await?;
    store_singleton(vcode, "nacro_ne", "Nacro names not in English, number & pc of nacro names",  
                    num_nacro_ne, Some(pc_nacro_ne), pool).await?;

    store_singleton(vcode, "names_nl", "Names not in Latin, number and pc of names",  
                    num_names_nl, Some(pc_names_nl), pool).await?;
    store_singleton(vcode, "acro_nl", "Acronyms not in Latin, number and pc of acronyms",  
                    num_acro_nl, Some(pc_acro_nl), pool).await?;
    store_singleton(vcode, "nacro_nl", "Nacro names not in Latin, number and pc of nacro names",  
                    num_nacro_nl, Some(pc_nacro_nl), pool).await?;          
    
    // Relationship data points

    let parch_orgs =  get_count("select count(*) from src.admin_data where n_chrels > 0 and n_parrels > 0", pool).await?;
    let parch_orgs_pc =  get_pc(parch_orgs, num_orgs);   
    store_singleton(vcode, "parch", "Orgs both parent and child, number & pc of total orgs",  
                        parch_orgs, Some(parch_orgs_pc), pool).await?;

    let par_no_child = get_rel_imbalance(1, 2, pool).await.unwrap();
    let par_no_parent = get_rel_imbalance(2, 1, pool).await.unwrap();
    let non_recip_pc = par_no_child + par_no_parent;
    let non_recip_rr = get_rel_imbalance(3, 3, pool).await.unwrap();
    let pred_no_succ = get_rel_imbalance(4, 5, pool).await.unwrap();
    let succ_no_pred = get_rel_imbalance(5, 4, pool).await.unwrap();
    let non_recip_ps = pred_no_succ + succ_no_pred;

    let parch_total =  get_count("select count(*) from src.relationships where rel_type = 1 or rel_type = 20", pool).await?;
    let rel_total =  get_count("select count(*) from src.relationships where rel_type = 4 or rel_type = 3", pool).await?;
    let predsucc_total =  get_count("select count(*) from src.relationships where rel_type = 4 or rel_type = 5", pool).await?;

    let pc_non_recip_pc = get_pc(non_recip_pc, parch_total);   
    let pc_non_recip_rr = get_pc(non_recip_rr, rel_total);  
    let pc_non_recip_ps = get_pc(non_recip_ps, predsucc_total);  

    store_singleton(vcode, "nrecip_pc", "Non-paired parent-child links, number & pc of such links", 
                    non_recip_pc, Some(pc_non_recip_pc), pool).await?;
    store_singleton(vcode, "nrecip_rr", "Non-paired 'related' links, number & pc of such links", 
                    non_recip_rr, Some(pc_non_recip_rr), pool).await?;
    store_singleton(vcode, "nrecip_ps", "Non-paired pred-succ links, number & pc of such links", 
                    non_recip_ps, Some(pc_non_recip_ps), pool).await?;

    // Data on ROR labels

    let num_label_ror = get_count(r#"select count(*) from src.names 
                                     where name_type = 5 and is_ror_name = true"#, pool).await?; 
    let num_label_nror = get_count(r#"select count(*) from src.names 
                                     where name_type = 5 and is_ror_name = false"#, pool).await?; 
    let num_nlabel_ror = get_count(r#"select count(*) from src.names 
                                     where name_type <> 5 and is_ror_name = true"#, pool).await?; 

    store_singleton(vcode, "label_ror", "Labels that are designated ROR names, number", num_label_ror, None, pool).await?;
    store_singleton(vcode, "label_nror", "Labels that are not designated ROR names, number", num_label_nror, None, pool).await?;
    store_singleton(vcode, "nlabel_ror", "Non-Label ROR names, number", num_nlabel_ror, None, pool).await?;

    let num_en_ror = get_count(r#"select count(*) from src.names 
                                  where is_ror_name = true and lang_code = 'en'"#, pool).await?;                                                    
    let num_nen_ror = get_count(r#"select count(*) from src.names 
                                  where is_ror_name = true and lang_code <> 'en' and lang_code is not null"#, pool).await?; 
    let num_wolc_ror = get_count(r#"select count(*) from src.names 
                                  where is_ror_name = true and lang_code is null"#, pool).await?; 

    let pc_en_ror = get_pc(num_en_ror, num_orgs);
    let pc_nen_ror = get_pc(num_nen_ror, num_orgs);
    let pc_wolc_ror = get_pc(num_wolc_ror, num_orgs); 

    store_singleton(vcode, "ror_en", "ROR names in English, number & pc of total orgs", num_en_ror, Some(pc_en_ror), pool).await?;
    store_singleton(vcode, "ror_nen", "ROR names not in English, number & pc of total orgs", 
                        num_nen_ror, Some(pc_nen_ror), pool).await?;
    store_singleton(vcode, "ror_wolc", "ROR names wolc, number & pc of total orgs", 
                        num_wolc_ror, Some(pc_wolc_ror), pool).await?;

    // Consider non-company organisations only.

    let num_ncmp_wolc_ror = get_count(r#"select count(n.id) from 
                    src.names n
                    inner join src.admin_data ad
                    on n.id = ad.id
                    where ad.is_company = false
                    and n.is_ror_name = true
                    and n.lang_code is null"#, pool).await?;   
    let num_ncmp_orgs = get_count(r#"select count(*) from src.admin_data where is_company = false"#, pool).await?;  
    let pc_ncmp_wolc_ror = get_pc(num_ncmp_wolc_ror, num_ncmp_orgs); 
    store_singleton(vcode, "ror_wolc_ncmp", "Noncmp ROR names wolc, number & pc of noncmp orgs", 
    num_ncmp_wolc_ror, Some(pc_ncmp_wolc_ror), pool).await?;

    // Location data

    let num_poly_locs = get_count(r#"select count(id) from src.admin_data where n_locs > 1"#, pool).await?;  
    let pc_poly_locs = get_pc(num_poly_locs, num_orgs); 
    let num_poly_subdivs = get_count(r#"select count(id) from src.admin_data where n_subdivs > 1"#, pool).await?;  
    let pc_poly_subdivs = get_pc(num_poly_subdivs, num_orgs); 
    let num_poly_countries = get_count(r#"select count(id) from src.admin_data where n_countries > 1"#, pool).await?;  
    let pc_poly_countries = get_pc(num_poly_countries, num_orgs); 

    store_singleton(vcode, "poly_locs", "Orgs with more than one location, number & pc of orgs", 
    num_poly_locs, Some(pc_poly_locs), pool).await?;
    store_singleton(vcode, "poly_subdivs", "Orgs in more than one ‘state’, number & pc of orgs",  
    num_poly_subdivs, Some(pc_poly_subdivs), pool).await?;
    store_singleton(vcode, "poly_countries", "Orgs in more than one country, number & pc of orgs",  
    num_poly_countries, Some(pc_poly_countries), pool).await?;

    Ok(())
}


pub async fn get_count (sql_string: &str, pool: &Pool<Postgres>) -> Result<i64, AppError> {
     sqlx::query_scalar(sql_string)
    .fetch_one(pool).await
    .map_err(|e| AppError::SqlxError(e, sql_string.to_string()))
}


fn get_pc (top:i64, bottom:i64) -> f64 {
    if bottom == 0
    { 0.0 }
    else {
        let res = ((top as f64) * 100.0) / bottom as f64;
        (res * 100.0).round() / 100.0  // return number to 2 decimal places
    }
}


async fn store_singleton(vcode: &String, id: &str, description: &str, number: i64, pc: Option<f64>, pool: &Pool<Postgres>)-> Result<PgQueryResult, AppError> {

    let sql = format!(r#"INSERT INTO smm.singletons (vcode, id, 
    description, number, pc) values($1, $2, $3, $4, $5)"#);

    sqlx::query(&sql)
    .bind(vcode.clone()).bind(id).bind(description).bind(number).bind(pc)
    .execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))
}


async fn store_summary(rows: Vec<TypeRow>, pool: &Pool<Postgres>, att_type: i32, att_name: &str) -> Result<(), AppError> {
    
    let sql = r#"INSERT into smm.attributes_summary (vcode, att_type, att_name, 
    id, name, number_atts, pc_of_atts, number_orgs, pc_of_orgs)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#;

    for t in rows {
        sqlx::query(sql).bind(t.vcode).bind(att_type).bind(att_name).bind(t.id).bind(t.name)
        .bind(t.number_atts).bind(t.pc_of_atts).bind(t.number_orgs).bind(t.pc_of_orgs)
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    }
    Ok(())
}


async fn store_distrib(rows: Vec<DistribRow>, count_type: &str, pool: &Pool<Postgres>)-> Result<(), AppError> {

    let sql = r#"INSERT INTO smm.count_distributions (vcode, 
    count_type, count, num_of_orgs, pc_of_orgs) values($1, $2, $3, $4, $5)"#;

    for r in rows {
        sqlx::query(&sql)
        .bind(r.vcode).bind(count_type)
        .bind(r.count).bind(r.num_of_orgs).bind(r.pc_of_orgs)
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    }
    Ok(())
}


async fn store_ranked_distrib(vcode: &String, rows: &Vec<RankedRow>, pool: &Pool<Postgres>, remainder_name: &str,
    dist_type : i32, entity_total: i64, base_set_total: i64) -> Result<(), AppError> {

    let mut i = 0;
    let mut rest_total = 0;
    let sql = r#"INSERT INTO smm.ranked_distributions (vcode, dist_type, rank, entity, 
    number, pc_of_entities, pc_of_base_set) 
    values($1, $2, $3, $4, $5, $6, $7)"#;

    for r in rows {
        i += 1;
        if i < 26 {
            sqlx::query(sql).bind(r.vcode.clone()).bind(dist_type).bind(i)
            .bind(r.entity.clone()).bind(r.number).bind(r.pc_of_entities).bind(r.pc_of_base_set)
            .execute(pool).await
            .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
        }
        else {
            rest_total += r.number;
        } 
    }
    if rest_total > 0 {

        let rest_ent_pc: f64 = get_pc(rest_total, entity_total).into();
        let rest_bs_pc: f64 = get_pc(rest_total, base_set_total).into();
        let sql = r#"INSERT INTO smm.ranked_distributions (vcode, dist_type, rank, entity, 
        number, pc_of_entities, pc_of_base_set) 
        values($1, $2, $3, $4, $5, $6, $7)"#;

        sqlx::query(sql).bind(vcode).bind(dist_type).bind(26)
        .bind(remainder_name).bind(rest_total).bind(rest_ent_pc).bind(rest_bs_pc)
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    }
    Ok(())
}


async fn get_rel_imbalance(f1_type: u8, f2_type: u8, pool: &Pool<Postgres>) -> Result<i64, AppError> {
 
    let sql = format!(r"select count(f1.id) from
          (select id, related_id from src.relationships where rel_type = {}) as f1
          left join
          (select id, related_id from src.relationships where rel_type = {}) as f2
          on f1.id = f2.related_id 
          and f1.related_id = f2.id
          where f2.id is null;", f1_type, f2_type);
           
    sqlx::query_scalar(&sql)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))

  }
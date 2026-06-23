

pub fn get_name_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_names = n
    from (
        select id, count(id) as n
        from ppr.names 
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_label_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_labels = n
    from (
        select id, count(id) as n
        from ppr.names where name_type = 5
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_alias_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_aliases = n
    from (
        select id, count(id) as n
        from ppr.names where name_type = 7
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_acronym_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_acronyms = n
    from (
        select id, count(id) as n
        from ppr.names where name_type = 10
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_nacro_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_nacro = n_names - n_acronyms;"#
}

pub fn get_names_wolc_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_names_wolc = n
    from (
        select id, count(id) as n
        from ppr.names 
        where lang_code is null
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_nacro_wolc_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_nacro_wolc = n
    from (
        select id, count(id) as n
        from ppr.names 
        where lang_code is null and name_type <> 10
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_companies_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set is_company = true
    from ppr.type t
    where ad.id = t.id
    and t.org_type = 400;"#
}

pub fn get_types_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_types = n
    from (
        select id, count(id) as n
        from ppr.type 
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_isni_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_isni = n
    from (
        select id, count(id) as n
        from ppr.external_ids 
        where id_type = 11
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_grid_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_grid = n
    from (
        select id, count(id) as n
        from ppr.external_ids 
        where id_type = 13
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_fundref_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_fundref = n
    from (
        select id, count(id) as n
        from ppr.external_ids 
        where id_type = 14
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_wikidata_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_wikidata = n
    from (
        select id, count(id) as n
        from ppr.external_ids 
        where id_type = 12
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_ext_ids_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data
    set n_ext_ids = n_isni + n_grid + n_fundref + n_wikidata;"#
}

pub fn get_wikipedia_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_wikipedia = n
    from (
        select id, count(id) as n
        from ppr.links 
        where link_type = 21
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_website_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_website = n
    from (
        select id, count(id) as n
        from ppr.links 
        where link_type = 22
        group by id) c
        where ad.id = c.id;"#
}

pub fn get_links_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data
    set n_links = n_wikipedia + n_website"#
}


pub fn get_locations_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_locs = n
    from (
        select id, count(id) as n
        from ppr.locations 
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_subdivs_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_subdivs = n
    from (
        select id, count(country_subdivision_code) as n
        from
            (select distinct id, country_subdivision_code
            from src.locations t) t
        group by t.id) c
    where ad.id = c.id;"#
}

pub fn get_countries_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_countries = n
    from (
        select id, count(country_code) as n
        from
            (select distinct id, country_code
            from src.locations t) t
        group by t.id) c
    where ad.id = c.id;"#
}

pub fn get_parrels_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_parrels = n
    from (
        select id, count(id) as n
        from ppr.relationships
        where rel_type = 1
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_chrels_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_chrels = n
    from (
        select id, count(id) as n
        from ppr.relationships
        where rel_type = 2
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_relrels_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_relrels = n
    from (
        select id, count(id) as n
        from ppr.relationships
        where rel_type = 3
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_predrels_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_predrels = n
    from (
        select id, count(id) as n
        from ppr.relationships
        where rel_type = 4
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_succrels_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_sucrels = n
    from (
        select id, count(id) as n
        from ppr.relationships
        where rel_type = 5
        group by id) c
    where ad.id = c.id;"#
}

pub fn get_domains_nums_sql <'a>() -> &'a str {
    r#"update ppr.admin_data ad
    set n_doms = n
    from (
        select id, count(id) as n
        from ppr.domains 
    group by id) c
    where ad.id = c.id;"#
}

// update core table with location data - for single location orgs

pub fn update_core_data_sql1 <'a>() -> &'a str {
    r#"update ppr.core_data c
    set location = t.name,
    csubdiv_code = t.country_subdivision_code,
    country_code = t.country_code
    from  
       (select a.id, name, country_subdivision_code, country_code
        from src.locations loc
        inner join ppr.admin_data a
        on loc.id = a.id
        where a.n_locs = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, single subdiv

pub fn update_core_data_sql2 <'a>() -> &'a str {
    r#"update ppr.core_data c
    set location = '**multi**',
    csubdiv_code = t.country_subdivision_code,
    country_code = t.country_code
    from 
        (select distinct a.id, country_subdivision_code, country_code
        from src.locations loc
        inner join ppr.admin_data a
        on loc.id = a.id
        where a.n_locs > 1
        and a.n_subdivs = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, multi subdivs, single country

pub fn update_core_data_sql3 <'a>() -> &'a str {
    r#"update ppr.core_data c
    set location = '**multi**',
    csubdiv_code = '**',
    country_code = t.country_code
    from 
        (select distinct a.id, country_code
        from src.locations loc
        inner join ppr.admin_data a
        on loc.id = a.id
        where a.n_subdivs > 1
        and a.n_countries = 1) t
    where c.id = t.id;"#
}

// update core table with location data - multi locations, multi countries

pub fn update_core_data_sql4 <'a>() -> &'a str {
    r#"update ppr.core_data c
    set location = '**multi**',
    csubdiv_code = '**',
    country_code = '**'
    from ppr.admin_data a
    where c.id = a.id 
    and a.n_countries > 1;"#
}

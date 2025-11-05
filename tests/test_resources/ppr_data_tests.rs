
use imp_ror::run;
use std::ffi::OsString;
use chrono::NaiveDate;

use imp_ror::setup::get_db_pool;
use super::ppr_data_access;
use super::ppr_record_structs::{PprCoreData, PprRelationship, PprExternalId, 
    PprName, PprLocation, PprLink, PprType, PprAdminData};


/**************************************************************
 * Run with cargo test -- --test-threads = 1
 * to keep tests running in (alphabetical!) order
 ***************************************************************/

#[tokio::test] 
async fn k_process_v2_0_data_to_ppr_and_summarise() {
 
    // Run the program with v2.0 test data

    let args : Vec<&str> = vec!["target/debug/src1.exe", "-p", "-z"];
    let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
    run(test_args).await.unwrap();

    // Assert     
    // Check numbers of records
    let pool = get_db_pool().await.unwrap();
    let rec_number = ppr_data_access::fetch_ppr_record_num("core_data", &pool).await;
    assert_eq!(rec_number, 20);
}


#[tokio::test] 
async fn l_check_numbers_in_each_ppr_table() {

    let pool = get_db_pool().await.unwrap();

    let rec_number = ppr_data_access::fetch_ppr_record_num("names", &pool).await;
    assert_eq!(rec_number, 56);
    let rec_number = ppr_data_access::fetch_ppr_record_num("relationships", &pool).await;
    assert_eq!(rec_number, 25);
    let rec_number = ppr_data_access::fetch_ppr_record_num("external_ids", &pool).await;
    assert_eq!(rec_number, 59);
    let rec_number = ppr_data_access::fetch_ppr_record_num("links", &pool).await;
    assert_eq!(rec_number, 33);
    let rec_number = ppr_data_access::fetch_ppr_record_num("type", &pool).await;
    assert_eq!(rec_number, 30);
}


#[tokio::test] 
async fn m_check_ppr_first_and_last_ids() {

    let pool = get_db_pool().await.unwrap();

    // Check first and last record Ids
    let first_id = ppr_data_access::fetch_ppr_first_record_id(&pool).await;
    assert_eq!(first_id, "006jxzx88");

    let last_id = ppr_data_access::fetch_ppr_last_record_id(&pool).await;
    assert_eq!(last_id, "05s6t3255");
}


#[tokio::test] 
async fn n_check_ppr_core_data() {

    let pool = get_db_pool().await.unwrap();

    let id = "006jxzx88";

    let core_data: PprCoreData = ppr_data_access::fetch_ppr_core_data_record (id, &pool).await;
    assert_eq!(core_data.ror_name, "Bond University");
    assert_eq!(core_data.ror_full_id, "https://ror.org/006jxzx88");
    assert_eq!(core_data.status, 1);
    assert_eq!(core_data.established.unwrap(), 1987);
    assert_eq!(core_data.location, Some("Gold Coast".to_string()));
    assert_eq!(core_data.country_code, Some("AU".to_string()));
    assert_eq!(core_data.csubdiv_code, None);

    let cr_dt = NaiveDate::parse_from_str("2018-11-14", "%Y-%m-%d").unwrap();
    let lm_dt= NaiveDate::parse_from_str("2024-05-13", "%Y-%m-%d").unwrap();
    let admin_data: PprAdminData = ppr_data_access::fetch_ppr_admin_data_record (id, &pool).await;

    assert_eq!(admin_data, PprAdminData{
        ror_name: "Bond University".to_string(), n_locs: 1, n_labels: 1,  n_aliases: 0, n_acronyms: 0,
        n_names: 1, n_names_wolc: 0, n_nacro: 1, n_nacro_wolc: 0, is_company:false, n_isni: 1, n_grid: 1, n_fundref: 1, n_wikidata: 1,
        n_ext_ids: 4, n_wikipedia: 1, n_website: 1, n_links: 2, n_types: 2, n_relrels: 2,
        n_parrels: 0, n_chrels: 0, n_sucrels: 0, n_predrels: 0, n_doms: 0,
        created: cr_dt, cr_schema: "1.0".to_string(), 
        last_modified: lm_dt, lm_schema: "2.0".to_string()});

    let id = "05s6t3255";
    
    let core_data: PprCoreData = ppr_data_access::fetch_ppr_core_data_record (id, &pool).await;
    assert_eq!(core_data.ror_name, "Food Packaging Forum Foundation");
    assert_eq!(core_data.ror_full_id, "https://ror.org/05s6t3255");
    assert_eq!(core_data.status, 1);
    assert_eq!(core_data.established.unwrap(), 2012);
    assert_eq!(core_data.location, Some("Zurich".to_string()));
    assert_eq!(core_data.country_code, Some("CH".to_string()));
    assert_eq!(core_data.csubdiv_code, Some("ZH".to_string()));

    let cr_dt = NaiveDate::parse_from_str("2023-07-27", "%Y-%m-%d").unwrap();
    let lm_dt= NaiveDate::parse_from_str("2024-12-11", "%Y-%m-%d").unwrap();
    let admin_data: PprAdminData = ppr_data_access::fetch_ppr_admin_data_record (id, &pool).await;
    assert_eq!(admin_data, PprAdminData{
        ror_name: "Food Packaging Forum Foundation".to_string(), n_locs: 1, n_labels: 1,  
        n_aliases: 1, n_acronyms: 1, n_names: 3, n_names_wolc: 1, n_nacro: 2, n_nacro_wolc: 0, is_company:false, n_isni: 1, n_grid: 0, 
        n_fundref: 0, n_wikidata: 1, n_ext_ids: 2, n_wikipedia: 1, n_website: 1, 
        n_links: 2, n_types: 1, n_relrels: 0, n_parrels: 0, n_chrels: 0, n_sucrels: 0, 
        n_predrels: 0, n_doms: 0,
        created: cr_dt, cr_schema: "1.0".to_string(), 
        last_modified: lm_dt, lm_schema: "2.1".to_string()});

}

#[tokio::test] 
async fn o_check_ppr_relationship_data() {

    let pool = get_db_pool().await.unwrap();

    let id = "03rd8mf35";
    let rels:Vec<PprRelationship> = ppr_data_access::fetch_ppr_relationship_records(id, &pool).await;
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0], PprRelationship{
        ror_name: "AECC University College".to_string(), rel_type: 1, related_id: "05wwcw481".to_string(), 
        related_name: "Bournemouth University".to_string(),});

    let id = "04ttjf776";
    let rels:Vec<PprRelationship> = ppr_data_access::fetch_ppr_relationship_records(id, &pool).await;
    assert_eq!(rels.len(), 4);
    assert_eq!(rels[0], PprRelationship{
        ror_name: "RMIT University".to_string(), rel_type: 2, related_id: "004axh929".to_string(), 
        related_name: "RMIT Vietnam".to_string(),});
    assert_eq!(rels[1], PprRelationship{
        ror_name: "RMIT University".to_string(), rel_type: 3, related_id: "010mv7n52".to_string(), 
        related_name: "Austin Hospital".to_string(),});
}


#[tokio::test] 
async fn p_check_ppr_external_id_data() {

    let pool = get_db_pool().await.unwrap();

    let id = "04ttjf776";
    let extids:Vec<PprExternalId> = ppr_data_access::fetch_ppr_external_id_records(id, &pool).await;
    assert_eq!(extids.len(), 6);
    assert_eq!(extids[0], PprExternalId{
        ror_name: "RMIT University".to_string(), id_type: 11, 
        id_value: "0000 0001 2163 3550".to_string(), 
        is_preferred: true, });
    assert_eq!(extids[1], PprExternalId{
        ror_name: "RMIT University".to_string(), id_type: 14, id_value: "100008690".to_string(), 
        is_preferred: false, });

    let id = "02vsmry93";
    let extids:Vec<PprExternalId> = ppr_data_access::fetch_ppr_external_id_records(id, &pool).await;
    assert_eq!(extids.len(), 3);
    assert_eq!(extids[1], PprExternalId{
        ror_name: "Türk Tarih Kurumu".to_string(), id_type: 14, id_value: "100020630".to_string(), 
        is_preferred: true,});
    assert_eq!(extids[2], PprExternalId{
        ror_name: "Türk Tarih Kurumu".to_string(), id_type: 12, id_value: "Q374071".to_string(), 
        is_preferred: true,});

}

#[tokio::test] 
async fn q_check_ppr_location_data() {

    let pool = get_db_pool().await.unwrap();

    let id = "006jxzx88";
    let locs:Vec<PprLocation> = ppr_data_access::fetch_ppr_location_records(id, &pool).await;
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0], PprLocation{
        ror_name: "Bond University".to_string(), geonames_id: 2165087, 
        location: "Gold Coast".to_string(), 
        lat: Some(-28.073982), lng: Some(153.41649), 
        cont_code: None, cont_name: None, 
        country_code: Some("AU".to_string()), country_name:Some("Australia".to_string()), 
        csubdiv_code: None, csubdiv_name: None,});

    let id = "05s6t3255";
    let locs:Vec<PprLocation> = ppr_data_access::fetch_ppr_location_records(id, &pool).await;
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0], PprLocation{
        ror_name: "Food Packaging Forum Foundation".to_string(), 
        geonames_id: 2657896, location: "Zurich".to_string(), lat: Some(47.36667), lng: Some(8.55), 
        cont_code: Some("EU".to_string()), cont_name: Some("Europe".to_string()), 
        country_code: Some("CH".to_string()), country_name:Some("Switzerland".to_string()), 
        csubdiv_code: Some("ZH".to_string()), csubdiv_name: Some("Zurich".to_string()),});

}

#[tokio::test] 
async fn r_check_ppr_link_data() {

    let pool = get_db_pool().await.unwrap();

    let id = "006jxzx88";
    let links:Vec<PprLink> = ppr_data_access::fetch_ppr_link_records(id, &pool).await;
    assert_eq!(links.len(), 2);
    assert_eq!(links[0], PprLink{
        ror_name: "Bond University".to_string(), link_type: 22, 
        link: "http://bond.edu.au/".to_string(),});
    assert_eq!(links[1], PprLink{
        ror_name: "Bond University".to_string(), link_type: 21,
        link: "https://en.wikipedia.org/wiki/Bond_University".to_string(),});

    let id = "05s6t3255";
    let links:Vec<PprLink> = ppr_data_access::fetch_ppr_link_records(id, &pool).await;
    assert_eq!(links.len(), 2);
    assert_eq!(links[0], PprLink{
        ror_name: "Food Packaging Forum Foundation".to_string(),
        link_type: 21, link: "https://en.wikipedia.org/wiki/Food_Packaging_Forum".to_string(),});
    assert_eq!(links[1], PprLink{
        ror_name: "Food Packaging Forum Foundation".to_string(),
        link_type: 22, link: "https://www.foodpackagingforum.org".to_string(),});

}

#[tokio::test] 
async fn s_check_ppr_type_data() {

    let pool = get_db_pool().await.unwrap();

    let id = "006jxzx88";
    let types:Vec<PprType> = ppr_data_access::fetch_ppr_type_records(id, &pool).await;
    assert_eq!(types.len(), 2);
    assert_eq!(types[0], PprType{ror_name: "Bond University".to_string(), org_type: 200,});
    assert_eq!(types[1], PprType{ror_name: "Bond University".to_string(), org_type: 600,});

    let id = "05s6t3255";
    let types:Vec<PprType> = ppr_data_access::fetch_ppr_type_records(id, &pool).await;
    assert_eq!(types.len(), 1);
    assert_eq!(types[0], PprType{ror_name: "Food Packaging Forum Foundation".to_string(), 
    org_type: 500,});

}

#[tokio::test]
async fn t_check_ppr_name_data() {

    //sleep(Duration::from_secs(7)).await;
    let pool = get_db_pool().await.unwrap();

    let id = "0198t0w55";
    let names:Vec<PprName> = ppr_data_access::fetch_ppr_name_records(id, &pool).await;
    assert_eq!(names.len(), 7);
    assert_eq!(names[0], PprName{
        value: "Institute of Reflective Investigation and Specialization".to_string(), name_type: 5, 
        is_ror_name: true, lang_code: Some("en".to_string()), script_code: Some("Latn".to_string())});
    assert_eq!(names[4], PprName{
        value: "Інститут Рефлексивних Досліджень & Спеціалізації".to_string(), name_type: 5, 
        is_ror_name: false, lang_code: Some("uk".to_string()), script_code: Some("Cyrl".to_string())});

    let id = "052rpwb50";
    let names:Vec<PprName> = ppr_data_access::fetch_ppr_name_records(id, &pool).await;
    assert_eq!(names.len(), 7);
    assert_eq!(names[0], PprName{
        value: "Yamashita Kōjōsen Byōin".to_string(), name_type: 7, 
        is_ror_name: false, lang_code: None, script_code: Some("Latn".to_string())});
    assert_eq!(names[4], PprName{
        value: "ヤマシタ コージョーセン ビョーイン".to_string(), name_type: 7, 
        is_ror_name: false, lang_code: Some("ja".to_string()), script_code: Some("Jpan".to_string())});
}



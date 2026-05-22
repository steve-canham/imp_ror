
mod json_models;
mod data_vectors;

use log::info;
use std::path::PathBuf;
use std::fs;
use sqlx::{Pool, Postgres};
use crate::AppError;
use chrono::NaiveDate;

use json_models::RorRecord;
use data_vectors::{CoreDataVecs, RequiredDataVecs, NonRequiredDataVecs, extract_id_from};


pub async fn import_data(data_folder : &PathBuf, source_file_name: &String,
                        data_version: &String, data_date: &String,
                        pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // First recreate the src schema tables - sqlscript in file (path is relative
    // and Linux specific - Windows would need a similar string but with backslashes)

    let sql = include_str!("../../sql/create_src_tables.sql");
    sqlx::raw_sql(sql).execute(pool)
        .await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Import data into matching tables. First obtain the raw data as text
    // This also checks the file exists.

    let source_file_path: PathBuf = [data_folder, &PathBuf::from(source_file_name)].iter().collect();
    let data: String = match fs::read_to_string(&source_file_path)
    {
        Ok(d) => { 
            info!("Got the data from the file");
            d
        },
        Err(e) => return Err(AppError::IoReadErrorWithPath(e, source_file_path)),
    };

    // Parse into an internal JSON structure

    let res:Vec<RorRecord> = match serde_json::from_str(&data)
    {
        Ok(r) => {
            info!("Parsed the data into ROR json objects");
            r
        },
        Err(e) => return Err(AppError::SerdeError(e)),
    };
    info!("{} records found", res.len());

    // First record data version, date and elapsed days in single record table.
    
    record_version_and_dates(data_version, data_date, pool).await?;
  
    // Set up vector variables.
    // Vectors are grouped into structs for ease of reference.

    let vector_size = 250;
    let mut cdv: CoreDataVecs = CoreDataVecs::new(vector_size);
    let mut rdv: RequiredDataVecs = RequiredDataVecs::new(vector_size);
    let mut ndv: NonRequiredDataVecs = NonRequiredDataVecs::new(vector_size);

    // Run through each record and store contents in relevant vectors.
    // After every (vector_size) records store vector contents to database
    // and clear vectors, but continue looping through records.

    let mut n = 0;
    for r in res {
        n += 1;
        let db_id = extract_id_from(&r.id).to_string();

        cdv.add_core_data(&r, &db_id);
        rdv.add_name_data(&r, &db_id, pool).await?;
        rdv.add_locs_and_types_data(&r, &db_id);
        ndv.add_non_required_data(&r, &db_id);

        if n % vector_size == 0 {
            if n % 5000 == 0 {
                info!("{} records processed", n);
            }

            // Store records to DB and clear vectorsl.
                        
            cdv.store_data(&pool).await?;
            rdv.store_data(&pool).await?;
            ndv.store_data(&pool).await?;
            
            cdv = CoreDataVecs::new(vector_size);
            rdv = RequiredDataVecs::new(vector_size);
            ndv = NonRequiredDataVecs::new(vector_size);
        }
    }

    // Store any residual vector contents.

    cdv.store_data(pool).await?;
    rdv.store_data(pool).await?;
    ndv.store_data(pool).await?;

    info!("Total records processed: {n}");
    Ok(())
}

async fn record_version_and_dates(data_version: &String, data_date: &String, pool: &Pool<Postgres>) -> Result<(), AppError> {

    let end_of_period = NaiveDate::parse_from_str(data_date, "%Y-%m-%d").unwrap();
    let start_of_period = NaiveDate::parse_from_str("2024-04-29", "%Y-%m-%d").unwrap();  // date v2 schema introduced
    let duration = end_of_period - start_of_period;
    
    let sql = r#"INSERT into src.version_details (version, data_date, data_days)
                    values ($1, $2, $3);"#;
    sqlx::query(&sql).bind(data_version).bind(data_date).bind(duration.num_days())
        .execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    Ok(())
}


pub async fn summarise_import(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Goes through each table and get total record number.

    info!("");
    info!("**************************************************");
    info!("Total record numbers for each table in src schema:");
    info!("**************************************************");
    info!("");

    write_record_num("core_data", pool).await?;
    write_record_num("admin_data", pool).await?;
    write_record_num("names", pool).await?;
    write_record_num("locations", pool).await?;
    write_record_num("external_ids", pool).await?;
    write_record_num("links", pool).await?;
    write_record_num("type", pool).await?;
    write_record_num("relationships", pool).await?;
    write_record_num("domains", pool).await?;

    info!("");
    info!("************************************");
    info!("");

    Ok(())
}


pub async fn write_record_num (table_name: &str, pool: &Pool<Postgres>) -> Result<(), AppError> {
    let sql = format!("SELECT COUNT(*) FROM src.{table_name}");
    let res: i64 = sqlx::query_scalar(&sql)
        .fetch_one(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    info!("Total records in src.{table_name}: {res}");
    Ok(())
}

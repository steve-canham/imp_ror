// The import module. Referenced in main by 'mod import'.
// It makes use of the other modules in the folder, each corresponding to a file of the same name.
// The folder modules do not need to be public - they are referenced only within this module.

mod ror_json_models;
mod ror_data_vectors;
mod ror_create_tables;

use log::{info, error};
use std::path::PathBuf;
use std::fs;
use sqlx::{Pool, Postgres};
use crate::AppError;
use chrono::NaiveDate;

use ror_json_models::RorRecord;
use ror_data_vectors::{CoreDataVecs, RequiredDataVecs, NonRequiredDataVecs, extract_id_from};

pub async fn create_ror_tables(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    match ror_create_tables::create_tables(pool).await {
        Ok(()) => info!("Tables created for ror schema"),
        Err(e) => {
            error!("An error occured while creating the ror schema tables: {}", e);
            return Err(e)
            },
    };
    Ok(())
}

pub async fn import_data(data_folder : &PathBuf, source_file_name: &String, 
                        data_version: &String, data_date: &String, 
                        pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Record data version, date and elapsed days in single record table.
    
    let end_of_period = NaiveDate::parse_from_str(data_date, "%Y-%m-%d").unwrap();
    let start_of_period = NaiveDate::parse_from_str("2024-04-29", "%Y-%m-%d").unwrap();
    let duration = end_of_period - start_of_period;
 
    let sql = r#"INSERT into ror.version_details (version, data_date, data_days)
                    values ($1, $2, $3);"#;
    sqlx::query(&sql).bind(data_version).bind(data_date).bind(duration.num_days())
    .execute(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Import data into matching tables. First obtain the raw data as text
    // This also checks the file exists...by opening it and checking no error

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
    for (i, r) in res.iter().enumerate() {
    
        let db_id = extract_id_from(&r.id).to_string();

        cdv.add_core_data(r, &db_id); 
        rdv.add_required_data(r, &db_id); 
        ndv.add_non_required_data(r, &db_id); 
        
        //if i > 705 { break;  }

        if (i + 1) % vector_size == 0 {  
            
            n += vector_size;
            if n % 5000 == 0 { 
                info!("{} records processed", n);
            }
            
            // store records to DB and clear vectors
            cdv.store_data(&pool).await;
            cdv = CoreDataVecs::new(vector_size);
            rdv.store_data(&pool).await;
            rdv = RequiredDataVecs::new(vector_size);
            ndv.store_data(&pool).await;
            ndv = NonRequiredDataVecs::new(vector_size);
        }
    }
    
    //store any residual vector contents

    cdv.store_data(&pool).await;
    rdv.store_data(&pool).await;
    ndv.store_data(&pool).await;

    info!("Total records processed: {}", n + cdv.db_ids.len());

    Ok(())

}


pub async fn summarise_import(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    // Goes through each table and get total record number.

    info!("");
    info!("************************************");
    info!("Total record numbers for each table:");
    info!("************************************");
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
    let sql = "SELECT COUNT(*) FROM ror.".to_owned() + table_name;
    let res: i64 = sqlx::query_scalar(&sql)
    .fetch_one(pool).await
    .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
   
    info!("Total records in ror.{}: {}", table_name, res);
    Ok(())
}
  
  
  


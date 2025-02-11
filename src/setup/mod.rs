/**********************************************************************************
The setup module, and the get_params function in this file in particular, 
orchestrates the collection and fusion of parameters as provided in 
1) a config toml file, and 
2) command line arguments. 
Where a parameter may be given in either the config file or command line, the 
command line version always over-writes anything from the file.
The module also checks the parameters for completeness (those required will vary, 
depending on the activity specified). If possible, defaults are used to stand in for 
mising parameters. If not possible the program stops with a message explaining the 
problem.
The module also provides a database connection pool on demand.
***********************************************************************************/

pub mod config_reader;
mod config_writer;
pub mod log_helper;
mod cli_reader;
mod lup_create_tables;
mod lup_fill_tables;

use crate::err::AppError;
use chrono::NaiveDate;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, PgPool};
use sqlx::{Postgres, Pool};
use log::{info, error};
use std::path::PathBuf;
use std::ffi::OsString;
use std::fs;
use std::time::Duration;
use regex::Regex;
use sqlx::ConnectOptions;
use config_reader::Config;
use cli_reader::Flags;

pub struct InitParams {
    pub data_folder: PathBuf,
    pub log_folder: PathBuf,
    pub output_folder: PathBuf,
    pub source_file_name: PathBuf,
    pub data_version: String,
    pub data_date: String,
    pub flags: Flags,
}

pub fn get_params(args: Vec<OsString>, config_string: String) -> Result<InitParams, AppError> {

    let cli_pars = cli_reader::fetch_valid_arguments(args)?;
    let flags = cli_pars.flags;

    if flags.create_lookups || flags.create_summary 
       || flags.create_config {

       // Any ror data and any other flags or arguments are ignored.

        Ok(InitParams {
            data_folder: PathBuf::new(),
            log_folder: PathBuf::new(),
            output_folder: PathBuf::new(),
            source_file_name: PathBuf::new(),
            data_version: "".to_string(),
            data_date: "".to_string(),
            flags: cli_pars.flags,
        })
    }
    else {

        // Normal import and / or processing and / or outputting
        // If folder name also given in CL args the CL version takes precedence

        let config_file: Config = config_reader::populate_config_vars(&config_string)?; 
        let file_pars = config_file.files;  // guaranteed to exist
        let data_pars = config_file.data_details; 
        let empty_pb = PathBuf::from("");

        let data_folder  =  file_pars.data_folder_path;

        // Does this folder exist and is it accessible? - If not and the 
        // 'R' (import ror) option is active, raise error and exit program.

        let mut data_folder_good = true;
        if !folder_exists (&data_folder) 
        {   
            data_folder_good = false;
        }

        if !data_folder_good && flags.import_ror { 
            return Result::Err(AppError::MissingProgramParameter("data_folder".to_string()));
        }

        let mut log_folder = file_pars.log_folder_path;
        if log_folder == empty_pb && data_folder_good {
            log_folder = data_folder.clone();
        }
        else {
            if !folder_exists (&log_folder) { 
                fs::create_dir_all(&log_folder)?;
            }
        }

        let mut output_folder = file_pars.output_folder_path;
        if output_folder == empty_pb && data_folder_good {
            output_folder = data_folder.clone();
        }
        else {
            if !folder_exists (&output_folder) { 
                fs::create_dir_all(&output_folder)?;
            }
        }

        // If source file name given in CL args the CL version takes precedence.
    
        let mut source_file_name= cli_pars.source_file;
        if source_file_name == empty_pb {
            source_file_name =  file_pars.src_file_name;
            if source_file_name == empty_pb && flags.import_ror {   // Required data is missing
                return Result::Err(AppError::MissingProgramParameter("src_file_name".to_string()));
            }
        }
         
        // get the output file name - from the config variables (may be a default)

        let mut data_version = "".to_string();
        let mut data_date = "".to_string();

        // If file name conforms to the correct pattern data version and data date can be derived.
        
        if cli_pars.flags.test_run {
            data_version = "v99".to_string();
            data_date = "2030-01-01".to_string()
        }
        else {
            match source_file_name.to_str() {
                Some(s_file) => {
                    if is_compliant_file_name(s_file) {
                        data_version = get_data_version(s_file);
                        data_date = get_data_date(s_file);
                 }
                },
                None => {},
            }
        }

        if data_version == "".to_string() ||  data_date == "".to_string()     
        {
            // Parsing of file name has not been completely successful, so get the version and date 
            // of the data from the CLI, or failing that the config file.

            data_version= cli_pars.data_version;
            if data_version == "" {
                data_version = data_pars.data_version;
                if data_version == "" && flags.import_ror {   // Required data is missing - Raise error and exit program.
                    return Result::Err(AppError::MissingProgramParameter("data_version".to_string()));
                }
            }
        
            data_date = match NaiveDate::parse_from_str(&cli_pars.data_date, "%Y-%m-%d") {
                Ok(_) => cli_pars.data_date,
                Err(_) => "".to_string(),
            };

            if data_date == "" {  
                    let config_date = &data_pars.data_date;
                    data_date = match NaiveDate::parse_from_str(config_date, "%Y-%m-%d") {
                    Ok(_) => config_date.to_string(),
                    Err(_) => "".to_string(),
                };

                if data_date == "" && flags.import_ror {   // Raise an AppError...required data is missing.
                    return Result::Err(AppError::MissingProgramParameter("data_date".to_string()));
                }
            }
        }

        // For execution flags read from the environment variables
       
        Ok(InitParams {
            data_folder,
            log_folder,
            output_folder,
            source_file_name,
            data_version,
            data_date,
            flags: cli_pars.flags,
        })

    }
}


fn folder_exists(folder_name: &PathBuf) -> bool {
    let xres = folder_name.try_exists();
    let res = match xres {
        Ok(true) => true,
        Ok(false) => false, 
        Err(_e) => false,           
    };
    res
}
        

pub async fn get_db_pool() -> Result<PgPool, AppError> {  

    // Establish DB name and thence the connection string
    // (done as two separate steps to allow for future development).
    // Use the string to set up a connection options object and change 
    // the time threshold for warnings. Set up a DB pool option and 
    // connect using the connection options object.

    let db_name = match config_reader::fetch_db_name() {
        Ok(n) => n,
        Err(e) => return Err(e),
    };

    let db_conn_string = config_reader::fetch_db_conn_string(&db_name)?;  
   
    let mut opts: PgConnectOptions = db_conn_string.parse()
                    .map_err(|e| AppError::DBPoolError("Problem with parsing conection string".to_string(), e))?;
    opts = opts.log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(3));

    PgPoolOptions::new()
        .max_connections(5) 
        .connect_with(opts).await
        .map_err(|e| AppError::DBPoolError(format!("Problem with connecting to database {} and obtaining Pool", db_name), e))
}


pub async fn edit_config() -> Result<(), AppError>
{
    match config_writer::edit_config_file().await {
        Ok(()) => info!("Configuration file edits completed"),
        Err(e) => {
            error!("An error occured while editing the configuration file: {}", e);
            return Err(e)
            },
    };
    
    Ok(())
}


pub async fn create_lup_tables(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    match lup_create_tables::create_tables(pool).await {
        Ok(()) => info!("Tables created for lup schema"),
        Err(e) => {
            error!("An error occured while creating the lup schema tables: {}", e);
            return Err(e)
            },
    };
    match lup_fill_tables::fill_tables(pool).await {
        Ok(()) => info!("Data added to lup tables"),
        Err(e) => {
            error!("An error occured while inserting data into the lup schema tables: {}", e);
            return Err(e)
            },
    };
    Ok(())
}


fn is_compliant_file_name(input: &str) -> bool {
    let file_name_pattern = r#"^v[0-9]+(\.[0-9]+){0,2}(-| )20[0-9]{2}-?[01][0-9]-?[0-3][0-9]"#;
    let re = Regex::new(file_name_pattern).unwrap();
    re.is_match(input)
}

fn get_data_version(input: &str) -> String {

    let version_pattern = r#"^v[0-9]+(\.[0-9]+){0,2}"#;
    let re = Regex::new(version_pattern).unwrap();
    if re.is_match(&input) {
        let caps = re.captures(&input).unwrap();
        caps[0].trim().to_string()
    }
    else {
        "".to_string()
    }
}

fn get_data_date(input: &str) -> String {            
    
    let date_pattern = r#"20[0-9]{2}-?[01][0-9]-?[0-3][0-9]"#;
    let re = Regex::new(date_pattern).unwrap();
    if re.is_match(&input) {
        let caps = re.captures(&input).unwrap();
        let putative_date = caps[0].replace("-", ""); // remove any hyphens
        match NaiveDate::parse_from_str(&putative_date, "%Y%m%d")
        {
            Ok(nd) => nd.to_string(),  // returns as YYY-mm-DD
            Err(_) => "".to_string(),
        }
    } 
    else {
        "".to_string()
    }
}


// Tests
#[cfg(test)]

mod tests {
    use super::*;
   
   // regex tests
   #[test]
   fn check_file_name_regex_works_1 () {
      let test_file_name = "v1.50 2024-12-11.json".to_string();
      assert_eq!(is_compliant_file_name(&test_file_name), true);
      assert_eq!(get_data_version(&test_file_name), "v1.50");
      assert_eq!(get_data_date(&test_file_name), "2024-12-11");
   }

   #[test]
   fn check_file_name_regex_works_2 () {
      let test_file_name = "v1.50-2024-12-11.json".to_string();
      assert_eq!(is_compliant_file_name(&test_file_name), true);
      assert_eq!(get_data_version(&test_file_name), "v1.50");
      assert_eq!(get_data_date(&test_file_name), "2024-12-11");
   }  

   #[test]
   fn check_file_name_regex_works_3 () {
      let test_file_name = "v1.50 20241211.json".to_string();
      assert_eq!(is_compliant_file_name(&test_file_name), true);
      assert_eq!(get_data_version(&test_file_name), "v1.50");
      assert_eq!(get_data_date(&test_file_name), "2024-12-11");
   }

   #[test]
   fn check_file_name_regex_works_4 () {
      let test_file_name = "v1.50-20241211.json".to_string();
      assert_eq!(is_compliant_file_name(&test_file_name), true);
      assert_eq!(get_data_version(&test_file_name), "v1.50");
      assert_eq!(get_data_date(&test_file_name), "2024-12-11");
   }

   #[test]
   fn check_file_name_regex_works_5 () {
      let test_file_name = "v1.50-2024-1211.json".to_string();
      assert_eq!(is_compliant_file_name(&test_file_name), true);
      assert_eq!(get_data_version(&test_file_name), "v1.50");
      assert_eq!(get_data_date(&test_file_name), "2024-12-11");
   }

   #[test]
   fn check_file_name_regex_works_6 () {
      let test_file_name = "v1.59-2025-01-23-ror-data_schema_v2.json".to_string();
      assert_eq!(is_compliant_file_name(&test_file_name), true);
      assert_eq!(get_data_version(&test_file_name), "v1.59");
      assert_eq!(get_data_date(&test_file_name), "2025-01-23");
   }
   
   #[test]
    fn check_file_name_regex_works_7 () {
        let test_file_name = "1.50 2024-12-11.json".to_string();
        assert_eq!(is_compliant_file_name(&test_file_name), false);

        let test_file_name = "v1.50--2024-12-11.json".to_string();
        assert_eq!(is_compliant_file_name(&test_file_name), false);

        let test_file_name = "v1.50  20241211.json".to_string();
        assert_eq!(is_compliant_file_name(&test_file_name), false);

        let test_file_name = "v1.50 20242211.json".to_string();
        assert_eq!(is_compliant_file_name(&test_file_name), false);

        let test_file_name = "v1.50.20241211.json".to_string();
        assert_eq!(is_compliant_file_name(&test_file_name), false);
    }
}
 
    // Ensure the parameters are being correctly combined.
    // The testing functions need to be async because of the call to get_params.
    // This function needs to be awaited to execute.

    // The closure is replaced by an explicitly async block rather than
    // a normal closure. Inserting '||' before or after the 'async' results
    // in multiple complaints from the compiler. The async block can also
    // be replaced by a separate async function and called explicitly.
 

    #[tokio::test]
    async fn check_config_vars_overwrite_blank_cli_values() {

        // Note that in most cases the folder path given must exist, and be 
        // accessible, or get_params will panic and an error will be thrown. 

        let config = r#"
    [data]
    data_version="v1.60"
    data_date="2025-12-11"

    [files]
    data_folder_path="E:\\MDR source data\\ROR\\data"
    log_folder_path="E:\\MDR source data\\ROR\\logs"
    output_folder_path="E:\\MDR source data\\ROR\\outputs"
    src_file_name="v1.58 20241211.json"

    [database]
    db_host="localhost"
    db_user="user_name"
    db_password="password"
    db_port="5433"
    db_name="ror"
    "#;
            let config_string = config.to_string();
            config_reader::populate_config_vars(&config_string).unwrap();
            let args : Vec<&str> = vec!["target/debug/ror1.exe"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

            let res = get_params(test_args, config_string).unwrap();

            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, false);
            assert_eq!(res.flags.export_text, false);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:\\MDR source data\\ROR\\data"));
            assert_eq!(res.log_folder, PathBuf::from("E:\\MDR source data\\ROR\\logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:\\MDR source data\\ROR\\outputs"));
            assert_eq!(res.source_file_name, PathBuf::from("v1.58 20241211.json"));
            assert_eq!(res.data_version, "v1.58");
            assert_eq!(res.data_date, "2024-12-11");
        }

 /* 
    #[tokio::test]
    async fn check_cli_vars_overwrite_env_values() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/20241211 1.58 data")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_version", Some("v1.59")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 27.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-r", "-p", "-t", "-x",
                                     "-f", "E:/ROR/data", "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, true);
            assert_eq!(res.flags.export_csv, true);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 27.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");
        }
       ).await;

    }


    #[tokio::test]
    async fn check_cli_vars_with_i_flag() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/20241211 1.58 data")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 27.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-r", "-p", "-i", 
                                        "-f", "E:/ROR/data", "-d", "2026-12-25", "-s", "schema2 data.json"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, false);
            assert_eq!(res.flags.process_data, false);
            assert_eq!(res.flags.export_text, false);
            assert_eq!(res.flags.create_lookups,true);
            assert_eq!(res.flags.create_summary, true);
            assert_eq!(res.data_folder, PathBuf::new());
            assert_eq!(res.log_folder, PathBuf::new());
            assert_eq!(res.output_folder, PathBuf::new());
            assert_eq!(res.source_file_name, "".to_string());
            assert_eq!(res.output_file_name, "".to_string());
            assert_eq!(res.data_version, "".to_string());
            assert_eq!(res.data_date, "".to_string());
        }
       ).await;

    }


    #[tokio::test]
    async fn check_cli_vars_with_a_flag_and_new_win_folders() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:\\ROR\\20241211 1.58 data")),
            ("log_folder_path", Some("E:\\ROR\\some logs")),
            ("output_folder_path", Some("E:\\ROR\\dummy\\some outputs")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 28.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-a", "-f", "E:\\ROR\\data", 
                                       "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, true);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/some logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/dummy/some outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 28.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");
        }
      ).await;

    }
    
    #[tokio::test]
    async fn check_cli_vars_with_a_flag_and_new_posix_folders() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/data")),
            ("log_folder_path", Some("E:/ROR/some logs 2")),
            ("output_folder_path", Some("E:/ROR/dummy 2/some outputs")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 28.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-a", "-f", "E:/ROR/data", 
                                       "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, true);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/some logs 2"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/dummy 2/some outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 28.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");
        }
      ).await;

    }


    #[tokio::test]
    #[should_panic]
    async fn check_wrong_data_folder_panics_if_r() {
    
    temp_env::async_with_vars(
    [
        ("data_folder_path", Some("E:/ROR/20240607 1.47 data")),
        ("log_folder_path", Some("E:/ROR/some logs")),
        ("output_folder_path", Some("E:/ROR/dummy/some outputs")),
        ("src_file_name", Some("v1.58 20241211.json")),
        ("data_date", Some("2025-12-11")),
        ("output_file_name", Some("results 28.json")),
    ],
    async { 
        let args : Vec<&str> = vec!["target/debug/ror1.exe", "-a", "-f", "E:/silly folder name", 
                                    "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let _res = get_params(test_args).await.unwrap();
        }
      ).await;
    }

    #[tokio::test]
    async fn check_wrong_data_folder_does_not_panic_if_not_r() {
    
        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/daft data")),
            ("log_folder_path", Some("E:/ROR/some logs")),
            ("output_folder_path", Some("E:/ROR/dummy/some outputs")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 28.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-p", "-f", "E:/ROR/silly folder name", 
                                        "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
            assert_eq!(res.flags.import_ror, false);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, false);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/silly folder name"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/some logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/dummy/some outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 28.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");

            }
        ).await;
    }

}
*/

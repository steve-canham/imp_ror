/***************************************************************************
 * Module uses std::env to read environmental values from an .env file 
 * (must be in same folder as src). These include the database parameters, 
 * which are loaded into a static singleton value, to allow it to be 
 * interrogated later in the program if necessary. During setup, however, 
 * the DB parameters are only used to provide a database connection string 
 * for any specified database.
 * The .env file also contains the default names for the overall folder path 
 * (used for both source data and logs) and for the source and results file 
 * name. 
 * 
 * Database parameters MUST be provided and be valid or the program can not
 * continue. 
 * The folder path and the source file name have defaults but these should 
 * NOT normally be used. They are there only as placeholders, to be overwritten by 
 * values provided as string arguments in the command line or the .env file. 
 * In other words the folder path and the source file name MUST be present 
 * EITHER in the .env file OR in the CLI arguments. 
 * If both, the CLI arguments take precedence.
 * The results file name has a timestamped default name that will be used if 
 * none is provided explicitly.
 ***************************************************************************/

use std::sync::OnceLock;
use toml;
use std::fs;
use serde::Deserialize;
use chrono::Local;
use crate::error_defs::{AppError, CustomError};
use std::path::PathBuf;

pub static DB_PARS: OnceLock<DBPars> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub data: Option<TomlDataPars>,
    pub files: Option<TomlFilePars>, 
    pub database: Option<TomlDBPars>,
}

#[derive(Debug, Deserialize)]
pub struct TomlDataPars {
    pub data_version: Option<String>,
    pub data_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TomlFilePars {
    pub data_folder_path: Option<String>,
    pub log_folder_path: Option<String>,
    pub output_folder_path: Option<String>,
    pub src_file_name: Option<String>,
    pub output_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TomlDBPars {
    pub db_host: Option<String>,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub db_port: Option<usize>,
    pub db_name: Option<String>,
}

pub struct Config {
    pub data_details: DataPars, 
    pub files: FilePars, 
}

pub struct DataPars {
    pub data_version: String,
    pub data_date: String,
}

pub struct FilePars {
    pub data_folder_path: PathBuf,
    pub log_folder_path: PathBuf,
    pub output_folder_path: PathBuf,
    pub src_file_name: PathBuf,
    pub output_file_name: PathBuf,
}


pub struct DBPars {
    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
    pub db_port: usize,
    pub db_name: String,
}



pub fn populate_config_vars(config_file_path: &str) -> Result<Config, AppError> {
    
    let config_string: String = fs::read_to_string(config_file_path)?;
    println!("{}", config_string);
    
    let toml_config = match toml::from_str::<TomlConfig>(&config_string)
    {
        Ok(c) => c,
        Err(_) => { 
            let app_err = report_critical_error ("open config file", 
            "the correct name, form and is in the correct location");
            return Result::Err(app_err)   // return error to calling function
        },
    };


    let toml_data_details = match toml_config.data {
        Some(d) => d,
        None => {
            println!("Data detals section not found in config file.");
            TomlDataPars {
                data_version: None,
                data_date: None,
            }
        },
    };

    let toml_database = match toml_config.database {
        Some(d) => d,
        None => {
            let app_err = report_critical_error ("read DB parameters from config file", 
            "a set of values under table 'database'");
            return Result::Err(app_err)
        },
    };


    let toml_files = match toml_config.files {
        Some(f) => f,
        None => {
            let app_err = report_critical_error ("read file parameters from config file", 
            "a set of values under table 'files'");
            return Result::Err(app_err)
        },
    };
   
    let config_files = verify_file_parameters(toml_files)?;
    let config_data_dets = verify_data_parameters(toml_data_details)?;
    verify_db_parameters(toml_database)?;

    Ok(Config{
        data_details: config_data_dets,
        files: config_files,
    })
}


fn verify_data_parameters(toml_data_pars: TomlDataPars) -> Result<DataPars, AppError> {

    let data_version = match toml_data_pars.data_version {
        Some(s) => s,
        None => "".to_string(),
    };

    let data_date = match toml_data_pars.data_date {
        Some(s) => s,
        None => "".to_string(),
    };

    Ok(DataPars {
        data_version,
        data_date,
    })
}



fn verify_file_parameters(toml_files: TomlFilePars) -> Result<FilePars, AppError> {

    // Check data folder and source file first as there are no defaults for these values.
    // They must therefore be present.

    let data_folder_path = match toml_files.data_folder_path {
        Some(s) => PathBuf::from(s),
        None => {
            let app_err = report_critical_error ("read data folder path from config file", 
            "a value for data_folder_path");
            return Result::Err(app_err)
        },
    };

    let src_file_name = match toml_files.src_file_name {
        Some(s) => PathBuf::from(s),
        None => {
            let app_err = report_critical_error ("read source file from config file", 
            "a value for src_file_name");
            return Result::Err(app_err)
        },
    };
        
    let log_folder_path = match toml_files.log_folder_path {
        Some(s) => PathBuf::from(s),
        None => {
            println!(r#"No value found for log folder path in config file - 
            using the provided data folder instead."#);
            data_folder_path.clone()
        },
    };

    let output_folder_path = match toml_files.output_folder_path {
        Some(s) => PathBuf::from(s),
        None => {
            println!(r#"No value found for outputs folder path in config file - 
            using the provided data folder instead."#);
            data_folder_path.clone()
        },
    };
   
    let output_file_name = match toml_files.output_file_name {
        Some(s) => PathBuf::from(s),
        None => {
            println!(r#"No value found for outputs file name in config file - 
            using default name with date-time stamp."#);
            let datetime_string = Local::now().format("%m-%d %H%M%S").to_string();
            let start_of_name = "umls import results at ".to_string();
            let output_file_string =  start_of_name + &datetime_string;
            PathBuf::from(output_file_string)
        },
    };


    Ok(FilePars {
        data_folder_path,
        log_folder_path,
        output_folder_path,
        src_file_name,
        output_file_name,
    })
}



fn verify_db_parameters(toml_database: TomlDBPars) -> Result<(), AppError> {

// Check user name and password first as there are no defaults for these values.
    // They must therefore be present.

    let db_user = match toml_database.db_user {
        Some(s) => s,
        None => {
            let app_err = report_critical_error ("read user name from config file", 
            "a value for db_user");
            return Result::Err(app_err)
        },
    };

    let db_password = match toml_database.db_password {
        Some(s) => s,
        None => {

            let app_err = report_critical_error ("read user password from config file", 
            "a value for db_password");
            return Result::Err(app_err)
        },
    };

    let db_host = toml_database.db_host.unwrap_or_else(||
    {
        println!("No value found for DB host in config file - using default of 'localhost'.");
        "localhost".to_string()
    });
        
    let db_port = toml_database.db_port.unwrap_or_else(||
    {
        println!("No value found for DB port in config file - using default of 5432");
        5432
    });

    let db_name = toml_database.db_name.unwrap_or_else(||
    {
        println!("No value found for DB name in config file - using default of 'umls'.");
        "umls".to_string()
    });

    let _ = DB_PARS.set(DBPars {
        db_host,
        db_user,
        db_password,
        db_port,
        db_name,
    });

    Ok(())
}


fn report_critical_error (sec2: &str, error_suffix: &str) -> AppError {
 
    let print_msg = r#"CRITICAL ERROR - Unable to "#.to_string() + error_suffix + r#" - 
    program cannot continue. Please check config file ('config_r_umls.toml') 
    has "# + sec2;
    println!("{}", print_msg);

    let err_msg = format!("CRITICAL ERROR - Unable to {}", error_suffix);
    let cf_err = CustomError::new(&err_msg);

    AppError::CsErr(cf_err) 
}
   

pub fn fetch_db_name() -> Result<String, AppError> {
    let db_pars = match DB_PARS.get() {
         Some(dbp) => dbp,
         None => {
            let msg = "Unable to obtain DB name when retrieving database name";
            let cf_err = CustomError::new(msg);
            return Result::Err(AppError::CsErr(cf_err));
        },
    };
    Ok(db_pars.db_name.clone())
}


pub fn fetch_db_conn_string(db_name: String) -> Result<String, AppError> {
    let db_pars = match DB_PARS.get() {
         Some(dbp) => dbp,
         None => {
            let msg = "Unable to obtain DB parameters when building connection string";
            let cf_err = CustomError::new(msg);
            return Result::Err(AppError::CsErr(cf_err));
        },
    };
    
    Ok(format!("postgres://{}:{}@{}:{}/{}", 
    db_pars.db_user, db_pars.db_password, db_pars.db_host, db_pars.db_port, db_name))
}


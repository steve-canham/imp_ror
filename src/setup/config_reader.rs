/***************************************************************************
 *
 * 
 * Database parameters MUST be provided and be valid or the program can not
 * continue. 
 * 
 * 
 ***************************************************************************/

use std::sync::OnceLock;
use toml;
use serde::Deserialize;
use crate::err::AppError;
use std::path::PathBuf;


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
    pub db_port: Option<String>,
    pub db_name: Option<String>,
}


pub struct Config {
    pub data_details: DataPars, 
    pub files: FilePars, 
    pub db_pars: DBPars,
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
}

#[derive(Debug, Clone)]
pub struct DBPars {
    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
    pub db_port: usize,
    pub db_name: String,
}

pub static DB_PARS: OnceLock<DBPars> = OnceLock::new();


pub fn populate_config_vars(config_string: &String) -> Result<Config, AppError> {

    let toml_config = toml::from_str::<TomlConfig>(&config_string)
        .map_err(|_| {AppError::ConfigurationError("Unable to parse config file.".to_string(),
                                       "File (app_config.toml) may be malformed.".to_string())})?;

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
        None => {return Result::Err(AppError::ConfigurationError("Missing or misspelt configuration section.".to_string(),
            "Cannot find a section called '[database]'.".to_string()))},
    };

    let toml_files = match toml_config.files {
        Some(f) => f,
        None => {return Result::Err(AppError::ConfigurationError("Missing or misspelt configuration section.".to_string(),
           "Cannot find a section called '[files]'.".to_string()))},
    };
       
    let config_files = verify_file_parameters(toml_files)?;
    let config_data_dets = verify_data_parameters(toml_data_details)?;
    let config_db_pars = verify_db_parameters(toml_database)?;

    let _ = DB_PARS.set(config_db_pars.clone());

    Ok(Config{
        data_details: config_data_dets,
        files: config_files,
        db_pars: config_db_pars,
    })
}


fn verify_data_parameters(toml_data_pars: TomlDataPars) -> Result<DataPars, AppError> {

    let data_version = match toml_data_pars.data_version {
        Some(s) => s.trim().to_string(),
        None => "".to_string(),
    };

    let data_date = match toml_data_pars.data_date {
        Some(s) => s.trim().to_string(),
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
    let data_folder_string = check_essential_string (toml_files.data_folder_path, "data path folder", "data_folder_path")?;

    let src_file_string = check_essential_string (toml_files.src_file_name, "source file name", "src_file_name")?;

    let log_folder_string = check_defaulted_string (toml_files.log_folder_path, "log folder", "data_folder_path", &data_folder_string);

    let output_folder_string = check_defaulted_string (toml_files.output_folder_path, "outputs folder", "data_folder_path", &data_folder_string);

    Ok(FilePars {
        data_folder_path: PathBuf::from(data_folder_string),
        log_folder_path: PathBuf::from(log_folder_string),
        output_folder_path: PathBuf::from(output_folder_string),
        src_file_name: PathBuf::from(src_file_string),
    })
}

fn verify_db_parameters(toml_database: TomlDBPars) -> Result<DBPars, AppError> {

    // Check user name and password first as there are no defaults for these values.
    // They must therefore be present.

    let db_user = check_essential_string (toml_database.db_user, "database user name", "db_user")?; 

    let db_password = check_essential_string (toml_database.db_password, "database user password", "db_password")?;
       
    let db_host = check_defaulted_string (toml_database.db_host, "DB host", "localhost", "localhost");
            
    let db_port_as_string = check_defaulted_string (toml_database.db_port, "DB port", "5432", "5432");
    let db_port: usize = db_port_as_string.parse().unwrap_or_else(|_| 5432);

    let db_name = check_defaulted_string (toml_database.db_name, "DB name", "ror", "ror");

    Ok(DBPars {
        db_host,
        db_user,
        db_password,
        db_port,
        db_name,
    })
}


fn check_essential_string (src_name: Option<String>, value_name: &str, config_name: &str) -> Result<String, AppError> {
 
    let s = match src_name {
        Some(s) => s,
        None => "none".to_string(),
    };

    if s == "none".to_string() || s.trim() == "".to_string()
    {
        return Result::Err(AppError::ConfigurationError("Essential configuration value missing or misspelt.".to_string(),
        format!("Cannot find a value for {} ({}).", value_name, config_name)))
    }
    else {
        Ok(s)
    }
}


fn check_defaulted_string (src_name: Option<String>, value_name: &str, default_name: &str, default:  &str) -> String {
 
    let s = match src_name {
        Some(s) => s,
        None => "none".to_string(),
    };

    if s == "none".to_string() || s.trim() == "".to_string()
    {
        println!("No value found for {} path in config file - 
        using the provided default value ('{}') instead.", value_name, default_name);
        default.to_owned()
    }
    else {
       s
    }
}


pub fn fetch_db_name() -> Result<String, AppError> {
    let db_pars = match DB_PARS.get() {
         Some(dbp) => dbp,
         None => {
            return Result::Err(AppError::MissingDBParameters());
        },
    };
    Ok(db_pars.db_name.clone())
}

pub fn fetch_db_conn_string(db_name: &String) -> Result<String, AppError> {
    let db_pars = match DB_PARS.get() {
         Some(dbp) => dbp,
         None => {
            return Result::Err(AppError::MissingDBParameters());
        },
    };
    
    Ok(format!("postgres://{}:{}@{}:{}/{}", 
    db_pars.db_user, db_pars.db_password, db_pars.db_host, db_pars.db_port, db_name))
}




#[cfg(test)]
mod tests {
    use super::*;
    
    // Ensure the parameters are being correctly extracted from the config file string
    
    #[test]
    fn check_config_with_all_params_present() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"

[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
log_folder_path="E:\\MDR source data\\ROR\\logs"
output_folder_path="E:\\MDR source data\\ROR\\outputs"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.files.data_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.log_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\logs"));
        assert_eq!(res.files.output_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\outputs"));
        assert_eq!(res.files.src_file_name, PathBuf::from("v1.59-2025-01-23-ror-data_schema_v2.json"));

        assert_eq!(res.data_details.data_version, "v99");
        assert_eq!(res.data_details.data_date, "2026-06-15");

        assert_eq!(res.db_pars.db_host, "localhost");
        assert_eq!(res.db_pars.db_user, "user_name");
        assert_eq!(res.db_pars.db_password, "password");
        assert_eq!(res.db_pars.db_port, 5433);
        assert_eq!(res.db_pars.db_name, "ror");
    }


    #[test]
    fn check_config_with_missing_log_and_outputs_folders() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"

[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.files.data_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.log_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.output_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.src_file_name, PathBuf::from("v1.59-2025-01-23-ror-data_schema_v2.json"));
    }


    #[test]
    fn check_config_with_blank_log_and_outputs_folders() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"

[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
log_folder_path=""
output_folder_path=""
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.files.data_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.log_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.output_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.src_file_name, PathBuf::from("v1.59-2025-01-23-ror-data_schema_v2.json"));
    }


    #[test]
    fn check_missing_data_details_become_empty_strings() {

        let config = r#"
[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
log_folder_path="E:\\MDR source data\\ROR\\logs"
output_folder_path="E:\\MDR source data\\ROR\\outputs"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.files.data_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\data"));
        assert_eq!(res.files.log_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\logs"));
        assert_eq!(res.files.output_folder_path, PathBuf::from("E:\\MDR source data\\ROR\\outputs"));
        assert_eq!(res.files.src_file_name, PathBuf::from("v1.59-2025-01-23-ror-data_schema_v2.json"));

        assert_eq!(res.data_details.data_version, "");
        assert_eq!(res.data_details.data_date, "");

        assert_eq!(res.db_pars.db_host, "localhost");
        assert_eq!(res.db_pars.db_user, "user_name");
        assert_eq!(res.db_pars.db_password, "password");
        assert_eq!(res.db_pars.db_port, 5433);
        assert_eq!(res.db_pars.db_name, "ror");
    }


    #[test]
    #[should_panic]
    fn check_missing_data_folder_panics() {
    let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"

[files]
log_folder_path="E:\\MDR source data\\ROR\\logs"
output_folder_path="E:\\MDR source data\\ROR\\outputs"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        let _res = populate_config_vars(&config_string).unwrap();
    }


    #[test]
    #[should_panic]
    fn check_missing_user_name_panics() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"

[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
log_folder_path="E:\\MDR source data\\ROR\\logs"
output_folder_path="E:\\MDR source data\\ROR\\outputs"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user=""
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        let _res = populate_config_vars(&config_string).unwrap();
    }


    #[test]
    fn check_db_defaults_are_supplied() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"

[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
log_folder_path="E:\\MDR source data\\ROR\\logs"
output_folder_path="E:\\MDR source data\\ROR\\outputs"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_user="user_name"
db_password="password"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.db_pars.db_host, "localhost");
        assert_eq!(res.db_pars.db_user, "user_name");
        assert_eq!(res.db_pars.db_password, "password");
        assert_eq!(res.db_pars.db_port, 5432);
        assert_eq!(res.db_pars.db_name, "ror");
    }


#[test]
    fn missing_port_gets_default() {

        let config = r#"
[files]
data_folder_path="E:\\MDR source data\\ROR\\data"
log_folder_path="E:\\MDR source data\\ROR\\logs"
output_folder_path="E:\\MDR source data\\ROR\\outputs"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port=""
db_name="ror"

"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();

        assert_eq!(res.data_details.data_version, "");
        assert_eq!(res.data_details.data_date, "");

        assert_eq!(res.db_pars.db_host, "localhost");
        assert_eq!(res.db_pars.db_user, "user_name");
        assert_eq!(res.db_pars.db_password, "password");
        assert_eq!(res.db_pars.db_port, 5432);
        assert_eq!(res.db_pars.db_name, "ror");
    }


}
  


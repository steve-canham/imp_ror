
use std::sync::OnceLock;
use toml;
use serde::Deserialize;
use crate::err::AppError;
use std::path::PathBuf;
use log::info;

#[derive(Deserialize)]
pub struct TomlConfig {
    pub data: Option<TomlDataPars>,
    pub folders: Option<TomlFolderPars>,
    pub database: Option<TomlDBPars>,
}

#[derive(Deserialize)]
pub struct TomlDataPars {
    pub src_file_name: Option<String>,
    pub data_version: Option<String>,
    pub data_date: Option<String>,
}

#[derive(Deserialize)]
pub struct TomlFolderPars {
    pub data_folder_path: Option<String>,
    pub log_folder_path: Option<String>,
    pub output_folder_path: Option<String>,
}

#[derive(Deserialize)]
pub struct TomlDBPars {
    pub db_host: Option<String>,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub db_port: Option<String>,
    pub db_name: Option<String>,
}


pub struct Config {
    pub data_details: DataPars,
    pub folders: FolderPars,
    pub db_pars: DBPars,
}

pub struct DataPars {
    pub src_file_name: String,
    pub data_version: String,
    pub data_date: String,
}

pub struct FolderPars {
    pub data_folder_path: PathBuf,
    pub log_folder_path: PathBuf,
    pub output_folder_path: PathBuf,
}

#[derive(Clone)]
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

    let toml_data = check_existence(toml_config.data, "data")?;
    let toml_folders = check_existence(toml_config.folders, "folders")?;
    let toml_database = check_existence(toml_config.database, "database")?;

    let config_data_dets = verify_data_parameters(toml_data)?;
    let config_folders = verify_folder_parameters(toml_folders)?;
    let config_db_pars = verify_db_parameters(toml_database)?;

    let _ = DB_PARS.set(config_db_pars.clone());

    Ok(Config{
        data_details: config_data_dets,
        folders: config_folders,
        db_pars: config_db_pars,
    })
}


fn verify_data_parameters(toml_data_pars: TomlDataPars) -> Result<DataPars, AppError> {

    Ok(DataPars {   // default values of "" available for all parameters
        src_file_name: toml_data_pars.src_file_name.unwrap_or_else(|| "".to_string()),
        data_version: toml_data_pars.data_version.unwrap_or_else(|| "".to_string()),
        data_date: toml_data_pars.data_date.unwrap_or_else(|| "".to_string()),
    })
}

fn verify_folder_parameters(toml_folders: TomlFolderPars) -> Result<FolderPars, AppError> {

    let data_folder_string = check_essential_string (toml_folders.data_folder_path, "data path folder", "data_folder_path")?;
    let log_folder_string = check_defaulted_string (toml_folders.log_folder_path, "log folder", &data_folder_string);
    let output_folder_string = check_defaulted_string (toml_folders.output_folder_path, "outputs folder", &data_folder_string);

    Ok(FolderPars {
        data_folder_path: PathBuf::from(data_folder_string),
        log_folder_path: PathBuf::from(log_folder_string),
        output_folder_path: PathBuf::from(output_folder_string),
    })
}

fn verify_db_parameters(toml_database: TomlDBPars) -> Result<DBPars, AppError> {

    // Check user name and password first as there are no defaults for these values.
    // They must therefore be present.

    let db_user = check_essential_string (toml_database.db_user, "database user name", "db_user")?;
    let db_password = check_essential_string (toml_database.db_password, "database user password", "db_password")?;

    let db_host = check_defaulted_string (toml_database.db_host, "DB host", "localhost");
    let db_port_as_string = check_defaulted_string (toml_database.db_port, "DB port", "5432");
    let db_port: usize = db_port_as_string.parse().unwrap_or_else(|_| 5432);

    let db_name = check_defaulted_string (toml_database.db_name, "DB name", "ror");

    Ok(DBPars {
        db_host,
        db_user,
        db_password,
        db_port,
        db_name,
    })
}

fn check_existence<T>(section: Option<T>, section_name: &str) -> Result<T, AppError> {
    section.ok_or_else(|| AppError::ConfigurationError("Missing or misspelt configuration section.".to_string(),
        format!("Cannot find a section called '[{}]'",section_name)))
}


fn check_essential_string (src_name: Option<String>, value_name: &str, config_name: &str) -> Result<String, AppError> {
    match src_name {
        Some(s) if !s.trim().is_empty() => Ok(s),
        _ => {
            Err(AppError::ConfigurationError("Essential configuration value missing or empty.".to_string(),
                format!("Cannot find a non-empty value for {} ({}).", value_name, config_name)))
        },
    }
}

fn check_defaulted_string (src_name: Option<String>, value_name: &str, default:  &str) -> String {
    match src_name {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            info!("No value found for the {} in config file - using the provided default value ('{}') instead.",
                value_name, default);
            default.to_owned()
        }
    }
}

pub fn config_file_exists(config_file_path: &str)-> bool {
    let config_path = PathBuf::from(config_file_path);
    match config_path.try_exists() {
        Ok(true) => true,
        _ => false, 
    }
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
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR/MDR_Logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5432"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.folders.data_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.folders.log_folder_path, PathBuf::from("/home/steve/Data/MDR/MDR_Logs/ror"));
        assert_eq!(res.folders.output_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));

        assert_eq!(res.data_details.src_file_name, "v1.59-2025-01-23-ror-data_schema_v2.json");
        assert_eq!(res.data_details.data_version, "v99");
        assert_eq!(res.data_details.data_date, "2026-06-15");

        assert_eq!(res.db_pars.db_host, "localhost");
        assert_eq!(res.db_pars.db_user, "user_name");
        assert_eq!(res.db_pars.db_password, "password");
        assert_eq!(res.db_pars.db_port, 5432);
        assert_eq!(res.db_pars.db_name, "ror");
    }


    #[test]
    fn check_config_with_missing_log_and_outputs_folders() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"


[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5432"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.folders.data_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.folders.log_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.folders.output_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));

        assert_eq!(res.data_details.src_file_name, "v1.59-2025-01-23-ror-data_schema_v2.json");
    }


    #[test]
    fn check_config_with_blank_log_and_outputs_folders() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
log_folder_path=""
output_folder_path=""

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5432"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        assert_eq!(res.folders.data_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.folders.log_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.folders.output_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));

        assert_eq!(res.data_details.src_file_name, "v1.59-2025-01-23-ror-data_schema_v2.json");
    }


    #[test]
    fn check_missing_data_details_become_empty_strings() {

        let config = r#"
[data]
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR/MDR_Logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5432"
db_name="ror"
"#;
        let config_string = config.to_string();
        let res = populate_config_vars(&config_string).unwrap();
        
        assert_eq!(res.folders.data_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.folders.log_folder_path, PathBuf::from("/home/steve/Data/MDR/MDR_Logs/ror"));
        assert_eq!(res.folders.output_folder_path, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));

        assert_eq!(res.data_details.src_file_name, "v1.59-2025-01-23-ror-data_schema_v2.json");
        assert_eq!(res.data_details.data_version, "");
        assert_eq!(res.data_details.data_date, "");

        assert_eq!(res.db_pars.db_host, "localhost");
        assert_eq!(res.db_pars.db_user, "user_name");
        assert_eq!(res.db_pars.db_password, "password");
        assert_eq!(res.db_pars.db_port, 5432);
        assert_eq!(res.db_pars.db_name, "ror");
    }


    #[test]
    #[should_panic]
    fn check_missing_data_folder_panics() {

        let config = r#"
[data]
data_version="v99"
data_date="2026-06-15"
src_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR/MDR_Logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5432"
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
ppr_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR/MDR_Logs/ror"

[database]
db_host="localhost"
db_user=""
db_password="password"
db_port="5432"
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
ppr_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR/MDR_Logs/ror"

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
[data]
ppr_file_name="v1.59-2025-01-23-ror-data_schema_v2.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR/MDR_Logs/ror"

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

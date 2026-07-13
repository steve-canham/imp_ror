
pub mod cli_reader;
pub mod config_reader;
pub mod db_pars;
pub mod log_helper;

use crate::sql::create_lup_tables;
use crate::sql::create_countries_table;
use crate::sql::create_lang_codes_table;
use crate::sql::create_scripts_table;

use crate::config::config_writer::create_config_file;
use crate::config::config_editor::edit_config_file;

use directories::ProjectDirs;
use std::ffi::OsString;
use std::sync::OnceLock;
use crate::err::AppError;
use chrono::NaiveDate;
use sqlx::{Postgres, Pool};
use std::path::PathBuf;
use std::fs;
use regex::Regex;
use config_reader::Config;
use cli_reader::Flags;

pub struct InitParams {
    pub data_folder: PathBuf,
    pub log_folder: PathBuf,
    pub output_folder: PathBuf,
    pub source_file_name: String,
    pub data_version: String,
    pub data_date: String,
    pub flags: Flags,
}

pub static LOG_RUNNING: OnceLock<bool> = OnceLock::new();

pub fn combine_params(args: Vec<OsString>) -> Result<InitParams, AppError>{
       
    // CLI parameters must be collected first as they may contain the '-c' flag,
    // which forces an initial creation or edit of the config file.
    // The config data is then processed to create a Config object, which includes 
    // database connection parameters, and parent folders for logs, source data, and output data.
    // These are created if not already in existence. 
    // CLI and config data are then combined into a single InitParams struct - the CLI's 
    // source file parameter will overrule any in the config file.
    
    let cli_pars = cli_reader::fetch_valid_arguments(args)?;    // Derived from the command line arguments. 
    let config_path = obtain_config_file_path()?;               // The OS dependent location of the config file.

    let config_string = if cli_pars.flags.create_config {    // -c flag set, so create, or edit an existing, config file.
        match config_path.try_exists() {
            Ok(true) => edit_config_file(&config_path)?,        
            _ => create_config_file(&config_path)?,          // No matching or not accessible file
        }
    }
    else {     // In great majority of cases simply read the config file.
        fs::read_to_string(&config_path)
            .map_err(|e| AppError::IoReadErrorWithPath(e, config_path.to_owned()))?
    };
   
    let flags = cli_pars.flags;
    let configuration: Config = config_reader::populate_config_vars(&config_string)?;
    let folder_pars = configuration.folders;  // guaranteed to exist
    let data_pars = configuration.data_details;

    let data_folder = if cli_pars.flags.test_run {
        cli_pars.test_folder
    }
    else {
        let dfp = folder_pars.data_folder_path;
        match flags.import_ror {
            true => {
                match folder_exists (&dfp) {
                    true => dfp,
                    _ => return Result::Err(AppError::MissingProgramParameter("data_folder".to_string()))
                }
            },
            false => dfp,
        }
    };

    let mut log_folder = folder_pars.log_folder_path;
    if log_folder == PathBuf::from("") && folder_exists (&data_folder) {
        log_folder = data_folder.clone();
    }
    else {
        if !folder_exists (&log_folder) {
            fs::create_dir_all(&log_folder)?;
        }
    }

    let mut output_folder = folder_pars.output_folder_path;
    if output_folder == PathBuf::from("") && folder_exists (&data_folder) {
        output_folder = data_folder.clone();
    }
    else {
        if !folder_exists (&output_folder) {
            fs::create_dir_all(&output_folder)?;
        }
    }

    // If source file name given in CL args the CL version takes precedence.

    let mut source_file_name = cli_pars.source_file;
    if source_file_name == "".to_string() {
        source_file_name =  data_pars.src_file_name;
        if source_file_name == "".to_string() && flags.import_ror {   // Required data is missing
            return Result::Err(AppError::MissingProgramParameter("src_file_name".to_string()));
        }
    }

    // Also ensure source file name ends in '.json', if it doesn't already.

    let name_len = source_file_name.len();
    if name_len > 5 {
        let ext = &source_file_name[(name_len - 5)..];
        if ext != ".json" {
            source_file_name = source_file_name + ".json";
       }
    }

    let mut data_version: String; 
    let mut data_date: String; 

    // If file name conforms to the correct pattern data version and data date can be derived.

    if is_compliant_file_name(&source_file_name) {
        data_version = get_data_version(&source_file_name);
        data_date = get_data_date(&source_file_name);
    }
    else {

        // Parsing of file name has not been completely successful, so get the version and date
        // of the data from the CLI, or failing that the config file.
        
        if cli_pars.flags.test_run {            // for test runs they are pre-defined
            data_version = "v99".to_string();
            data_date = "2030-01-01".to_string()
        }
        else {
            
            data_version = cli_pars.data_version;
            if data_version == "".to_string() {
                data_version = data_pars.data_version;
                if data_version == "".to_string() && flags.import_ror {   // Required data is missing - Raise error and exit program.
                        return Result::Err(AppError::MissingProgramParameter("data_version".to_string()));
                }
            }
            
            data_date = match NaiveDate::parse_from_str(&cli_pars.data_date, "%Y-%m-%d") {  // check if valid date
                Ok(_) => cli_pars.data_date,
                _ => "".to_string(),
            };
            if data_date == "" {
                let config_date = &data_pars.data_date;
                data_date = match NaiveDate::parse_from_str(config_date, "%Y-%m-%d") {
                    Ok(_) => config_date.to_string(),
                    _ => "".to_string(),
                };
    
                if data_date == "" && flags.import_ror {   // Raise an AppError...required data is missing.
                    return Result::Err(AppError::MissingProgramParameter("data_date".to_string()));
                }
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


fn obtain_config_file_path() -> Result<PathBuf, AppError> {

     if let Some(config) = ProjectDirs::from("eu", "canhamis", "imp_ror") {
         let config_folder = config.config_dir().to_path_buf();
         let file_name = "config.toml";
         Ok(config_folder.join(file_name))
 
         // Linux:   /home/<user name>/.config/imp_ror/config.toml
         // Windows: C:\Users\<user name>\AppData\Roaming\canhamis\imp_ror\config.toml
         // macOS:   /Users/<user name>/Library/Application Support/eu.canhamis.imp_ror/config.toml

     }   
     else {
         println!("Odd! - Unable to identify an OS-specific location for the configuration file");
         Err(AppError::ConfigurationError(
             "No folder for config file found".to_string(), 
             "Fatal error - unable to proceed".to_string()))
     }
}


fn folder_exists(folder_name: &PathBuf) -> bool {
    match folder_name.try_exists() {
        Ok(true) => true,
        _ => false,   // includes Ok(false) as well as Err
    }
}

pub fn establish_log(params: &InitParams) -> Result<(), AppError> {

    if !log_set_up() {  // can be called more than once in context of integration tests
        log_helper::setup_log(params)?;
        LOG_RUNNING.set(true).unwrap(); // should always work
        log_helper::log_startup_params(params);
        if params.flags.create_config {
            let config_string = get_config_string()?;
            log_helper::write_config(&config_string);
        }
    }
    Ok(())
}

pub fn log_set_up() -> bool {
    match LOG_RUNNING.get() {
        Some(_) => true,
        None => false,
    }
}


pub async fn create_lup_tables(pool : &Pool<Postgres>) -> Result<(), AppError>
{
    let sql = create_lup_tables::get_sql();
    sqlx::raw_sql(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = create_countries_table::get_sql();
    sqlx::raw_sql(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = create_lang_codes_table::get_sql();
    sqlx::raw_sql(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    let sql = create_scripts_table::get_sql();
    sqlx::raw_sql(sql).execute(pool).await
        .map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(())
}


fn is_compliant_file_name(input: &String) -> bool {
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

pub fn get_config_string () -> Result<String, AppError> {
    let config_path = obtain_config_file_path()?;               // The OS dependent location of the config file.
    fs::read_to_string(&config_path)
        .map_err(|e| AppError::IoReadErrorWithPath(e, config_path))
}


// Tests
#[cfg(test)]

mod tests {
    use super::*;
    use std::ffi::OsString;

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

    // Ensure the parameters are being correctly combined.

    #[test]
    fn check_config_vars_overwrite_blank_cli_values() {

        // Note that in most cases the data folder path given must exist, and be
        // accessible, or get_params will panic and an error will be thrown.

        let config = r#"
[data]
data_version="v1.60"
data_date="2025-12-11"
src_file_name="v1.58 20241211.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = combine_params(test_args).unwrap();

        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.data_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.log_folder, PathBuf::from("/home/steve/Data/MDR logs/ror"));
        assert_eq!(res.output_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));
        assert_eq!(res.source_file_name, "v1.58 20241211.json");
        assert_eq!(res.data_version, "v1.58");
        assert_eq!(res.data_date, "2024-12-11");
    }


    #[test]
    fn check_cli_vars_overwrite_env_values() {
        let config = r#"
[data]
data_version="v1.60"
data_date="2025-12-11"
src_file_name="v1.58 20241211.json"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
        "#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();
        let args : Vec<&str> = vec!["dummy target", "-r", "-p", "-t", "-x",
                                    "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = combine_params(test_args).unwrap();

        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, true);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.data_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.log_folder, PathBuf::from("/home/steve/Data/MDR logs/ror"));
        assert_eq!(res.output_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));        assert_eq!(res.source_file_name, "schema2 data.json");
        assert_eq!(res.data_version, "v1.60");
        assert_eq!(res.data_date, "2026-12-25");
    }


    #[test]
    fn check_cli_vars_with_cm_flags() {

        let config = r#"
[data]
src_file_name="v1.58 20241211.json"
data_version="v1.50"
data_date="2025-12-11"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
    "#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();
        let args : Vec<&str> = vec!["dummy target", "-r", "-p", "-x", "-y", "-c", "-m",
                                    "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = combine_params(test_args).unwrap();

        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, true);
        assert_eq!(res.flags.create_lookups,false);
        assert_eq!(res.flags.create_summary, true);
        assert_eq!(res.data_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.log_folder, PathBuf::from("/home/steve/Data/MDR logs/ror"));
        assert_eq!(res.output_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));
        assert_eq!(res.source_file_name, "schema2 data.json");
        assert_eq!(res.data_version, "v1.60");
        assert_eq!(res.data_date, "2026-12-25");
    }


    #[test]
    fn check_with_x_and_y_flags() {

        let config = r#"
[data]
src_file_name="v1.58 20241211.json"
data_version="v1.60"
data_date="2025-12-11"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
    "#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();
        let args : Vec<&str> = vec!["dummy target", "-x", "-y", "-s", "schema2 data.json"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = combine_params(test_args).unwrap();

        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.export_csv, true);
        assert_eq!(res.flags.export_all_csv, true);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.data_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.log_folder, PathBuf::from("/home/steve/Data/MDR logs/ror"));
        assert_eq!(res.output_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));
        assert_eq!(res.source_file_name, "schema2 data.json");
        assert_eq!(res.data_version, "v1.60");
        assert_eq!(res.data_date, "2025-12-11");
    }

    #[test]
    fn check_cli_vars_with_a_flag_and_posix_folders() {

        let config = r#"
[data]
src_file_name="v1.58 20241211.json"
data_version=""
data_date=""

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-a"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = combine_params(test_args).unwrap();

        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.data_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/data"));
        assert_eq!(res.log_folder, PathBuf::from("/home/steve/Data/MDR logs/ror"));
        assert_eq!(res.output_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));
        assert_eq!(res.source_file_name, "v1.58 20241211.json");
        assert_eq!(res.data_version, "v1.58");
        assert_eq!(res.data_date, "2024-12-11");
    }

    #[test]
    #[should_panic]
    fn check_wrong_data_folder_panics_if_r() {

        let config = r#"
[data]
src_file_name="v1.58 20241211.json"
data_version="v1.60"
data_date="2025-12-11"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/no_data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MD logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();
        let args : Vec<&str> = vec!["dummy target", "-a", "-v", "v1.60"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let _res = combine_params(test_args).unwrap();
    }


    #[test]
    fn check_wrong_data_folder_does_not_panic_if_not_r() {

        let config = r#"
[data]
src_file_name="v1.58 20241211.json"
data_version="v1.60"
data_date="2025-12-11"

[folders]
data_folder_path="/home/steve/Data/MDR source data/ROR/no_data"
output_folder_path="/home/steve/Data/MDR source data/ROR/outputs"
log_folder_path="/home/steve/Data/MDR logs/ror"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="ror"
"#;
        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-p"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = combine_params(test_args).unwrap();

        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.data_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/no_data"));
        assert_eq!(res.log_folder, PathBuf::from("/home/steve/Data/MDR logs/ror"));
        assert_eq!(res.output_folder, PathBuf::from("/home/steve/Data/MDR source data/ROR/outputs"));
        assert_eq!(res.source_file_name, "v1.58 20241211.json");
        assert_eq!(res.data_version, "v1.58");
        assert_eq!(res.data_date, "2024-12-11");
    }

}

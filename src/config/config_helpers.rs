use crate::err::AppError;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;
use std::fs::File;
use chrono::{NaiveDate, Local};


pub fn user_input() -> Result<String, AppError> {
    print!("    >> ");
    io::stdout().flush().unwrap();        // ensure >> prompt is shown
    let mut input = String::new();        // establish buffer
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input.trim().to_string()),      // Ok value is number of bytes read (as usize)
        Err(e) => Err(AppError::UserInputError(e)),
    }
}


pub fn user_input_no_default() -> Result<String, AppError> {
    let mut putative_value = "not valid".to_string();
    while putative_value == "not valid".to_string() {
        let users_selection = user_input()?;
        if users_selection.trim() != ""
        {
            putative_value = users_selection;
        }
        else
        {
            println!("    You must provide a value for this parameter! - please try again");
        }
    }
    Ok(putative_value)
}


pub fn user_input_or_default(default_value: &str) -> Result<(String, String), AppError> {
    print!("    >> ");
    io::stdout().flush().unwrap();        // ensure >> prompt is shown
    let mut input = String::new();        // establish buffer
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let mut suffix = "".to_string();
            let mut res = input.trim().to_string();
            if res == "" {
                res = default_value.to_string();
                suffix = " (= default)".to_string();
            }
            Ok((res, suffix))
        },          
        Err(e) => Err(AppError::UserInputError(e)),
    }
}


pub fn user_input_or_use_current(curr_value: &String) -> Result<String, AppError> {
    print!("    >> ");
    io::stdout().flush().unwrap(); 
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let mut res = input.trim().to_string();
            if res == "" {
                res = curr_value.to_string();
            }
            Ok(res)
        },    
        Err(e) => Err(AppError::UserInputError(e)),
    }
}


pub fn get_port_as_integer (port_string: &String) -> i32 {
    match port_string.parse()
    {
        Ok(n) => {
            if n < 0 {
                println!("    The port must be input as a positive integer!");
                -1
            }
            else {
                n
            }
        },
        Err(_) => {
            println!("    The port must be input as a positive integer!");
            -1
        },
    }
}

pub fn get_folder() -> Result<String, AppError> { 
    
    let mut putative_folder = "not valid".to_string();
    while putative_folder == "not valid".to_string() {
        let users_selection = user_input()?;
        if folder_exists(&PathBuf::from(&users_selection))
        {
            putative_folder = users_selection;
        }
        else
        {
            println!("    That folder does not appear to exist - please try again");
        }
    }
    Ok(putative_folder)
}


pub fn get_folder_or_use_current(curr_value: &PathBuf) -> Result<String, AppError> { 
    
    let mut putative_folder = "not valid".to_string();
    while putative_folder == "not valid".to_string() {
        let users_selection = user_input()?;
        if users_selection == "" {
            putative_folder = get_as_string(curr_value)?;
        }
        else {
            if folder_exists(&PathBuf::from(&users_selection))
            {
                putative_folder = users_selection;
            }
            else
            {
                println!("    That folder does not appear to exist - please try again");
            }
        }
    }
    Ok(putative_folder)
}


pub fn get_valid_date_string(date_string: &String) -> String {
    match NaiveDate::parse_from_str(date_string, "%Y-%m-%d") {
        Ok(nd) => {
            let today = Local::now().date_naive();
            if nd > today {
                println!("    The date entered is after today (!) - please try again");
                "no_valid_value".to_string()
            }
            else {
                date_string.to_string()
            }
        },
        Err(_) => {
            println!("    The date entered does not conform to the ISO pattern (yyyy-MM-dd) required - please try again");
            "no_valid_value".to_string()
        },
    }
}


pub fn folder_exists(folder_name: &PathBuf) -> bool {
    match folder_name.try_exists() {
        Ok(true) => true,
        _ => false,    // Ok(false) and Err(e)
    }
}


pub fn get_as_string(folder_path: &PathBuf) -> Result<String, AppError> {
    match folder_path.clone().into_os_string().into_string() {
        Ok(s) => Ok(s),
        Err(e) => Err(AppError::NonUTF8PathError(e)),    
    }
}


pub fn is_compliant_version(input: &String) -> Result<bool, AppError> {
    let version_pattern = r#"^v[0-9]+(\.[0-9]+){0,2}"#;
    let re = Regex::new(version_pattern)
        .map_err(|e| AppError::RegexError(e, version_pattern.to_string()))?;
    Ok(re.is_match(&input))
}


pub fn write_out_file(config_string: &String) -> Result<(), AppError> {
    let mut file = File::create("./app_config.toml")     // creates new or truncates existing
        .map_err(|e| AppError::IoWriteErrorWithPath(e, PathBuf::from("./app_config.toml")))?;

    file.write_all(config_string.as_bytes())
        .map_err(|e| AppError::IoWriteErrorWithPath( e, PathBuf::from("./app_config.toml")))
}

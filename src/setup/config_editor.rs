

use crate::err::AppError;
use std::io;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;
use chrono::NaiveDate;
use std::fs;
use super::config_reader::{Config, populate_config_vars};

pub fn edit_config_file() -> Result<(), AppError>
{
    // config file already exists so get the current file

    let config_file = PathBuf::from("./app_config.toml");
    let config_string: String = fs::read_to_string(&config_file)
                    .map_err(|e| AppError::IoReadErrorWithPath(e, config_file))?;
    let current_config: Config = populate_config_vars(&config_string)?; 

    let p1 = "      WELCOME TO IMP_ROR               EDITING EXISTING CONFIGURATION FILE";
    let star_line = "**********************************************************************************";
    let p2 = "N.B. For each of the data points below, pressing return will transfer the existing";
    let p3 = "configuration parameter (shown in brackets in the prompt) to the edited file";
    let section = format!("\n\n{}\n{}\n{}\n{}\n{}\n", star_line, p1, star_line, p2, p3);
    println!("{}", section);


    let p1 = "Section 1: DATABASE PARAMETERS";
    let p2 = "DATABASE HOST";
    let curr_value = current_config.db_pars.db_host;
    let p3 = "Please input the name of your database host (usually the server name or IP address).";
    let p4 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
    let section = format!("\n{}\n\n{}\n{}\n{}\n", p1, p2, p3, p4);
    println!("{}", section);
 
    let host = user_input(curr_value)?;
    let db_host = format!("db_host=\"{}\"", host);
    println!("{}", db_host);
    
    let p1 = "USER NAME";
    let curr_value = current_config.db_pars.db_user;
    let p2 = "Please input the name of the user account being used to access the database.";
    let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
    let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
    println!("{}", section);

    let user = user_input(curr_value)?;
    let db_user = format!("db_user=\"{}\"", user);
    println!("{}", db_user);

    let p1 = "USER PASSWORD";
    let curr_value = current_config.db_pars.db_password;
    let p2 = "Please input the name of the user password being used to access the database.";
    let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
    let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
    println!("{}", section);

    let password = user_input(curr_value)?;
    let db_password = format!("db_password=\"{}\"", password);
    println!("{}", db_password);

    let p1 = "PORT";
    let curr_value = current_config.db_pars.db_port;
    let p2 = "Please input the port number being used to access the database.";
    let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value.to_string());
    let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
    println!("{}", section);

    let mut port = -1;
    while port < 0 {
        let try_port = user_input(curr_value.to_string())?;
        port = match try_port.parse()
        {
            Ok(n) => n,
            Err(_) => {
                println!("{}", "The port must be input as an integer!");
                -1
            },
        };
    }
    let db_port = format!("db_port=\"{}\"", port);
    println!("{}", db_port);

    let p1 = "DATABASE NAME";
    let curr_value = current_config.db_pars.db_name;
    let p2 = "Please input the name of the database.";
    let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
    let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
    println!("{}", section);

    let dname = user_input(curr_value)?;
    let db_name = format!("db_name=\"{}\"", dname);
    println!("{}", db_name);


    let p1 = "Section 2: FOLDERS";
    let p2 = "DATA FOLDER";
    let curr_value = current_config.folders.data_folder_path;
    let p3 = "Please input the full path of the folder where the ROR JSON source file is to be found.";
    let p4 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value.display().to_string());
    let section = format!("\n{}\n\n{}\n{}\n{}\n", p1, p2, p3, p4);
    println!("{}", section);

    let mut df = "".to_string();
    while df == "".to_string() {
        let try_df = user_input(curr_value.display().to_string())?.replace("\\", "/");
        if folder_exists(&PathBuf::from(&try_df))
        {
            df = try_df;
        }
        else
        {
            println!("{}", "That folder does not appear to exist - please try again");
        }
    }
    let data_folder = &df;
    let data_folder_path = format!("data_folder_path=\"{}\"", df);
    println!("{}", data_folder_path);

    // check is a valid path - repeat request if not
    // change single to dounble slashes
   
    let p1 = "OUTPUTS FOLDER";
    let curr_value = current_config.folders.output_folder_path;
    let p2 = "Please input the full path of the folder where the outputs from the program should be placed.";
    let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value.display().to_string());
    let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
    println!("{}", section);

    let output_folder = user_input(curr_value.display().to_string())?;
    let output_folder_path: String;
    if output_folder == "" {
        output_folder_path = format!("output_folder_path=\"{}\"", data_folder);
    }
    else {
        let op_folder = output_folder.to_string().replace("\\", "/");
        output_folder_path = format!("output_folder_path=\"{}\"", op_folder);
    }
    println!("{}", output_folder_path);

    let p1 = "LOG FOLDER";
    let curr_value = current_config.folders.log_folder_path;
    let p2 = "Please input the full path of the folder where the logs from the program should be placed.";
    let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value.display().to_string());
    let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
    println!("{}", section);

    let log_folder = user_input(curr_value.display().to_string())?;
    let log_folder_path: String;
    if log_folder == "" {
        log_folder_path = format!("log_folder_path=\"{}\"", data_folder);
    }
    else {
        let lg_folder = log_folder.to_string().replace("\\", "/");
        log_folder_path = format!("log_folder_path=\"{}\"", lg_folder);
    }
    println!("{}", log_folder_path);


    let p1 = "Section 3: DATA PARAMETERS";
    let p2 = "SOURCE FILE NAME";
    let curr_value = current_config.data_details.ppr_file_name;
    let p3 = "The source file can be provided as a command line argument, or in the configuration file, or in both.";
    let p4 = "NOTE that any source file name provided in the command line will over-write the value in the config file.";
    let p5 = "NOTE also that without a source file named in the configuration file, i.e. if enter is pressed without ";
    let p6 = "entering a value, a source file name will ALWAYS have to be provided in the command line.";
    let p7 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
    let section = format!("\n{}\n\n{}\n\n{}\n{}\n{}\n\n{}\n{}\n{}\n{}\n", p1, p2, star_line, p3, p4, p5, p6, p7, star_line);
    println!("{}", section);

    let ppr_file = user_input(curr_value)?;
    let ppr_file_name = format!("ppr_file_name=\"{}\"", ppr_file);
    println!("{}", ppr_file_name);

    let mut data_version = format!("data_version=\"\"");
    let mut data_date = format!("data_date=\"\"");

    if ppr_file != "" {

        let p1 = "As you have stored a source file name in the configuration you may need to also store";
        let p2 = "the associated data version and date. These can be left as the defaults (empty strings)";
        let p3 = "if the version and date can be derived from the source file name (see documentation for the required pattern).";
        let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
        println!("{}", section);


        let p1 = "DATA VERSION";
        let curr_value = current_config.data_details.data_version;
        let p2 = "Please input the data version, as a 'v' followed by the version number, e.g. '1.56.1'.";
        let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
        let section = format!("\n{}\n{}\n{}\n\n", p1, p2, p3);
        println!("{}", section);

        let mut d_version = "zzzz".to_string();
        while d_version == "zzzz".to_string() {
            let d_v = user_input(curr_value.clone())?;
            if d_v == "".to_string() || is_compliant_version(&d_v)? {
                d_version = d_v;
            }
            else {
                println!("{}", "The version entered does not conform to the pattern required - please try again");
            }
        }
        data_version = format!("data_version=\"{}\"", d_version);
        println!("{}", data_version);

        // check starts with a v and has following digits / decimal points
       
        let p1 = "DATA DATE";
        let curr_value = current_config.data_details.data_date;
        let p2 = "Please input the data date as an ISO string, yyyy-MM-dd, e.g. '2025-07-22'.";
        let p3 = format!("To accept the current value ('{}') simply press enter, otherwise type the name and press enter.", curr_value);
        let section = format!("\n{}\n{}\n{}\n", p1, p2, p3);
        println!("{}", section);

        let mut d_date = "zzzz".to_string();
        while d_date == "zzzz".to_string() {
            let d_d = user_input(curr_value.clone())?;
            if d_d == "".to_string() || NaiveDate::parse_from_str(&d_d, "%Y-%m-%d").is_ok() {
                d_date = d_d;
            }
            else {
                println!("{}", "The date entered does not conform to the ISO pattern (yyyy-MM-dd) required - please try again");
            }
        }
        data_date = format!("data_date=\"{}\"", d_date);
        println!("{}", data_date);
    }


    let database_section = format!("[database]\n{}\n{}\n{}\n{}\n{}\n", db_host, db_user, db_password, db_port, db_name);
    let folders_section = format!("[folders]\n{}\n{}\n{}\n", data_folder_path, output_folder_path, log_folder_path);
    let data_section = format!("[data]\n{}\n{}\n{}\n", ppr_file_name, data_version, data_date);
    let config_string = format!("\n{}\n\n{}\n\n{}\n", data_section, folders_section, database_section);

    let mut file = File::create("./app_config.toml")     // creates new or truncates existing
    .map_err(|e| AppError::IoWriteErrorWithPath( e, PathBuf::from("./app_config.toml")))?;

    // Write to the file
    file.write_all(config_string.as_bytes())
        .map_err(|e| AppError::IoWriteErrorWithPath( e, PathBuf::from("./app_config.toml")))?;
    
    Ok(())
}


fn user_input(curr_value: String) -> Result<String, AppError> {
    print!(">>");
    io::stdout().flush().unwrap(); 
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| AppError::UserInputError(e))?;
    let mut res = input.trim().to_string();
    if res == "" {
        res = curr_value;
    }
    Ok(res)
}


fn folder_exists(folder_name: &PathBuf) -> bool {
    let res = match folder_name.try_exists() {
        Ok(true) => true,
        Ok(false) => false, 
        Err(_e) => false,           
    };
    res
}


fn is_compliant_version(input: &String) -> Result<bool, AppError> {
    let version_pattern = r#"^v[0-9]+(\.[0-9]+){0,2}"#;
    let re = Regex::new(version_pattern)
        .map_err(|e| AppError::RegexError(e, version_pattern.to_string()))?;
    Ok(re.is_match(&input))
}


use crate::err::AppError;
use super::config_helpers::*;
use std::path::PathBuf;
use std::fs;
use crate::setup::config_reader::{Config, populate_config_vars};
use log::info;

pub fn edit_config_file() -> Result<(), AppError>
{
    // *****************************************************
    // Obtain the existing configuratrion.
    // *****************************************************

    let config_file = PathBuf::from("./app_config.toml");
    let config_string: String = fs::read_to_string(&config_file)
                    .map_err(|e| AppError::IoReadErrorWithPath(e, config_file))?;
    let current_config: Config = populate_config_vars(&config_string)?; 

    // *****************************************************
    // Introduction.
    // *****************************************************
    
    let p = r#"
    **********************************************************************************
                          EDITING EXISTING CONFIGURATION FILE
    **********************************************************************************

    The program will prompt for a value for each of the required parameters. 
    N.B. In each case, pressing return will transfer the existing
    configuration parameter (shown in brackets in the prompt) to the edited file.
    "#;
    print!("{p}");

    // *****************************************************
    // Database parameters.
    // *****************************************************

    let curr_host_entry = current_config.db_pars.db_host;
    let curr_user_entry = current_config.db_pars.db_user;
    let curr_pwrd_entry = current_config.db_pars.db_password;
    let curr_port_entry = current_config.db_pars.db_port;
    let curr_db_entry = current_config.db_pars.db_name;
    
    let p = format!(r#"
    Section 1: DATABASE PARAMETERS

    DATABASE HOST
    Please input the name of your database host (usually the server name or IP address).
    To accept the current value ('{curr_host_entry}') simply press enter, otherwise type the name 
    and press enter.
    "#);
    println!("{p}");
 
    let host_entry = user_input_or_current(&curr_host_entry)?;
    let db_host_entry = format!(r#"db_host="{host_entry}""#);
    println!("    {db_host_entry}");
    
    let p = format!(r#"
    USER NAME
    Please input the name of the user account being used to access the database.
    To accept the current value ('{curr_user_entry}') simply press enter, otherwise type the name 
    and press enter.
    "#);
    println!("{p}");
       
    let user = user_input_or_current(&curr_user_entry)?;
    let db_user_entry = format!(r#"db_user="{user}""#);
    println!("    {db_user_entry}");

    let p = format!(r#"
    USER PASSWORD
    Please input the name of the user password being used to access the database.
    To accept the current value ('{curr_pwrd_entry}') simply press enter, otherwise type the name
    and press enter.
    "#);
    println!("{p}");

    let password = user_input_or_current(&curr_pwrd_entry)?;
    let db_password_entry= format!(r#"db_password="{password}""#);
    println!("    {db_password_entry}");

    let p = format!(r#"
    PORT
    Please input the port number being used to access the database.
    To accept the current value ('{curr_port_entry}') simply press enter, otherwise 
    type the name and press enter.
     "#);
    println!("{p}");

    let mut port: i32 = -1;
    while port < 0 {
        let users_port_selection = user_input()?;
        if users_port_selection == "" {
            port = curr_port_entry as i32;
        }
        else {
            port = get_port_as_integer(&users_port_selection);
        }
    }
    let db_port_entry = format!(r#"db_port="{port}""#);
    println!("    {db_port_entry}");

    let p = format!(r#"
    DATABASE NAME
    Please input the name of the database.
    To accept the current value ('{curr_db_entry}') simply press enter, otherwise type the name 
    and press enter.
    "#);
    println!("{p}");

    let dname = user_input_or_current(&curr_db_entry)?;
    let db_name_entry = format!(r#"db_name="{dname}""#);
    println!("    {db_name_entry}");

    // *****************************************************
    // Folder parameters.
    // *****************************************************

    let curr_df_value = get_pathbuf_as_string(&current_config.folders.data_folder_path)?;
    let curr_of_value = get_pathbuf_as_string(&current_config.folders.output_folder_path)?;
    let curr_lf_value = get_pathbuf_as_string(&current_config.folders.log_folder_path)?;
    
    let p = format!(r#"
    Section 2: FOLDERS
    
    DATA FOLDER
    Please input the full (Linux / Posix) path of the folder where the ROR JSON source file is to be found.
    To accept the current value ({}) 
    simply press enter, otherwise type the name and press enter.
    "#, curr_df_value);
    println!("{p}");

    let data_folder = get_folder_or_use_current(&curr_df_value)?;
    let data_folder_entry = format!(r#"data_folder_path="{data_folder}""#);
    println!("    {data_folder_entry}");

    let p = format!(r#"
    OUTPUTS FOLDER
    Please input the full path of the folder where the outputs from the program should be placed.
    To accept the current value ({}) 
    simply press enter, otherwise type the name and press enter."#, curr_of_value);
    println!("{p}");

    let output_folder = get_folder_or_use_current(&curr_of_value)?;
    let output_folder_entry = format!(r#"output_folder_path="{output_folder}""#);
    println!("    {output_folder_entry}");
        
    let p = format!(r#"
    LOG FOLDER
    Please input the full path of the folder where the logs from the program should be placed.
    To accept the current value ({}) 
    simply press enter, otherwise type the name and press enter."#, curr_lf_value);
    println!("{p}");

    let log_folder = get_folder_or_use_current(&curr_lf_value)?;
    let log_folder_entry = format!(r#"log_folder_path="{log_folder}""#);
    println!("    {log_folder_entry}");

    // *****************************************************
    // Data parameters.
    // *****************************************************
    
    let curr_src_file = current_config.data_details.src_file_name;

    let p = format!(r#"
    Section 3: DATA PARAMETERS
    
    SOURCE FILE NAME
    The source file can be provided as a command line argument, or in the configuration file, or in both.
    NOTE that any source file name provided in the command line will over-write the value in the config file.
    NOTE also that without a source file named in the configuration file, a source file name will ALWAYS have to be provided in the command line.
    To accept the current value ('{}') simply press enter, otherwise type the name and press enter.
    If there is currently a source file name and you wish to remove it type an empty string (""). 
    In this case the version and date will also be blanked.
    "#, curr_src_file);
    println!("{p}");

    let mut src_file = user_input_or_current(&curr_src_file)?;
    let mut src_file_entry = format!(r#"src_file_name="{src_file}""#);
    if src_file == r#""""# {  // i.e. has been put back to empty string
        src_file = "".to_string();
        src_file_entry = format!(r#"src_file_name="""#);
    }
    println!("    {src_file_entry}");

    let mut data_version_entry = format!(r#"data_version="""#);  // defaults
    let mut data_date_entry = format!(r#"data_date="""#);
    
    if src_file != "" {
        let p = r#"
    As you have stored a source file name in the configuration you may need to also store
    the associated data version and date. These can be left as the defaults (empty strings)
    if the version and date can be derived from the source file name (see documentation for the required pattern)."#;
        println!("{p}");

        let curr_data_version = current_config.data_details.data_version;
        let curr_data_date = current_config.data_details.data_date;
        
        let p = format!(r#"
    DATA VERSION
    Please input the data version, as a 'v' followed by the version number in ROR's versioning format, e.g. '1.56.1'.
    To accept the current value ('{}') simply press enter, otherwise type the name and press enter.
    "#, curr_data_version);
        println!("{p}");

        let mut d_version = "no_valid_value".to_string();
        while d_version == "no_valid_value".to_string() {
            let users_version_selection = user_input_or_current(&curr_data_version)?;
            if users_version_selection == "".to_string() || is_compliant_version(&users_version_selection)? {
                d_version = users_version_selection;
            }
            else {
                println!("    The version entered does not conform to the pattern required - please try again");
            }
        }
        data_version_entry= format!(r#"data_version="{d_version}""#);

        let p = format!(r#"
    DATA DATE
    Please input the data date as an ISO string, yyyy-MM-dd, e.g. '2025-07-22'.
    To accept the current value ('{}') simply press enter, otherwise type the name and press enter.
    "#, curr_data_date);
        println!("{p}");

        let mut d_date = "no_valid_value".to_string();
        while d_date == "no_valid_value".to_string() {
            let users_date_selection = user_input_or_current(&curr_data_date)?;
            if users_date_selection == "".to_string() {
                d_date = "".to_string();
            }
            else {
                d_date = get_valid_date_string(&users_date_selection);
            }
        }
        data_date_entry = format!(r#"data_date="{d_date}""#);
    }
    println!("    {}", data_version_entry);
    println!("    {}", data_date_entry);

    // *****************************************************
    // Recreate file.
    // *****************************************************
    
    let data_section = format!("[data]\n{}\n{}\n{}\n", src_file_entry, data_version_entry, data_date_entry);
    let folders_section = format!("[folders]\n{}\n{}\n{}\n", data_folder_entry, output_folder_entry, log_folder_entry);
    let database_section = format!("[database]\n{}\n{}\n{}\n{}\n{}\n", db_host_entry, db_user_entry, db_password_entry, db_port_entry, db_name_entry);
    let config_string = format!("\n{}\n\n{}\n\n{}\n", data_section, folders_section, database_section);
    write_out_file(&config_string)?;
    info!("Configuration file edits completed");
    Ok(())
}

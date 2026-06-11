use crate::err::AppError;
use super::config_helpers::*;
use chrono::{NaiveDate, Local};
use log::info;

pub fn create_config_file() -> Result<(), AppError>
{
    let p = r#"
    **********************************************************************************
        WELCOME TO IMP_ROR               INITIAL CONFIGURATION SET UP
    **********************************************************************************
    The initial task is to set up an app_config file, to hold the details needed
    to connect to the database, and some required folder paths. The program will ask a 
    series of questions to obtain the parameters. Defaults are available in some cases.
    "#;
    print!("{p}");

    let p = r#"
    Section 1: DATABASE PARAMETERS

    DATABASE HOST
    Please input the name of your database host (usually the server name or IP address).
    To accept the default ('localhost') simply press enter, otherwise type the name and press enter.
    "#;
    println!("{p}");
 
    let mut host = user_input()?;
    let mut suffix = "";
    if host == "" {
        host = "localhost".to_string();
        suffix = " (= default)";
    }
    let db_host = format!(r#"db_host="{host}""#);
    println!("    {db_host}{suffix}");
    
    let p = r#"
    USER NAME
    Please input the name of the user account being used to access the database.
    No default is available, please type the name and press enter.
    "#;
    println!("{p}");

    let user = user_input()?;
    let db_user = format!(r#"db_user="{user}""#);
    println!("    {db_user}");

    let p = r#"
    USER PASSWORD
    Please input the name of the user password being used to access the database.
    No default is available, please type the password and press enter.
    "#;
    println!("{p}");

    let password = user_input()?;
    let db_password = format!(r#"db_password="{password}""#);
    println!("    {db_password}");

    let p = r#"
    PORT
    Please input the port number being used to access the database.
    To accept the default ('5432') simply press enter, otherwise type the number and press enter.
    "#;
    println!("{p}");

    let mut port: i32 = -1;
    let mut suffix = "";
    while port < 0 {
        let users_port_selection = user_input()?;
        if users_port_selection == "" {
            port = 5432;
            suffix = " (= default)";
        }
        else {
            port = match users_port_selection.parse()
            {
                Ok(n) => n,
                Err(_) => {
                    println!("{}", "The port must be input as an integer!");
                    -1
                },
            };
        }
    }
    let db_port = format!(r#"db_port="{port}""#);
    println!("    {db_port}{suffix}");

    let p = r#"
    DATABASE NAME
    Pease input the name of the database.
    To accept the default ('ror') simply press enter, otherwise type the name and press enter.
    "#;
    println!("{p}");

    let mut suffix = "";
    let mut dname = user_input()?;
    if dname == "" {
        dname = "ror".to_string();
        suffix = " (= default)";
    }
    let db_name = format!(r#"db_name="{dname}""#);
    println!("    {db_name}{suffix}");

    let p = r#"
    Section 2: FOLDERS
    
    DATA FOLDER
    Please input the full (Linux / Posix) path of the folder where the ROR JSON source file is to be found.
    No default is available, please type the path and press enter.
    "#;
    println!("{p}");

    let data_folder = get_folder()?;
    let data_folder_path = format!(r#"data_folder_path="{data_folder}""#);
    println!("    {data_folder_path}");

    let p = r#"
    OUTPUTS FOLDER
    Please input the full path of the (Linux / Posix) folder where the outputs from the program should be placed.
    To accept the default (same as 'DATA FOLDER') simply press enter, otherwise type the path and press enter.
    "#;
    println!("{p}");

    let output_folder = get_folder()?;
    let output_folder_path = format!(r#"output_folder_path="{output_folder}""#);
    println!("    {output_folder_path}");

    let p = r#"
    LOG FOLDER
    Please input the full path of the (Linux / Posix) folder where the logs from the program should be placed.
    To accept the default (same as 'DATA FOLDER') simply press enter, otherwise type the path and press enter.
    "#;
    println!("{p}");

    let log_folder = get_folder()?;
    let log_folder_path = format!(r#"log_folder_path="{log_folder}""#);
    println!("    {log_folder_path}");

    let p = r#"
    Section 3: DATA PARAMETERS
    
    SOURCE FILE NAME
    The source file can be provided as a command line argument, or in the configuration file, or in both.
    NOTE that any source file name provided in the command line will over-write the value in the config file.
    NOTE also that without a source file named in the configuration file, i.e. if enter is pressed without 
    entering a value, a source file name will ALWAYS have to be provided in the command line.
    "#;
    println!("{p}");

    let users_src_file_selection = user_input()?;
    let src_file_name = format!(r#"src_file_name="{users_src_file_selection}""#);
    println!("    {src_file_name}");

    let mut data_version = format!(r#"data_version="""#); // defualts if users_ppr_file_selection is ""
    let mut data_date = format!(r#"data_date="""#);

    if users_src_file_selection != "" {

        let p1 = r#"
    As you have stored a source file name in the configuration you may need to also store
    the associated data version and date. These can be left as the defaults (empty strings)
    if the version and date can be derived from the source file name (see documentation for the required pattern).
    "#;
        println!("{p1}");

        let p1 = r#"
    DATA VERSION
    Please input the data version, as a 'v' followed by the version number, e.g. '1.56.1'.
    To accept the default (an empty string) simply press enter, otherwise type the version and press enter.
    "#;
        println!("{p1}");

        let mut suffix = "";
        let mut d_version = "no_valid_value".to_string();
        while d_version == "no_valid_value".to_string() {
            let users_version_selection = user_input()?;
            if users_version_selection == "".to_string()  {
                d_version = "".to_string();
                suffix = " (= default)";
            }
            else if is_compliant_version(&users_version_selection)? {   // check starts with v, and has digits and points following
                d_version = users_version_selection;
            }
            else {
                println!("{}", "The version entered does not conform to the pattern required - please try again");
            }
        }
        data_version = format!(r#"data_version="{d_version}""#);
        println!("    {data_version}{suffix}");

        let p1 = r#"
    DATA DATE
    Please input the data date as an ISO string, yyyy-MM-dd, e.g. '2025-07-22'.
    To accept the default (an empty string) simply press enter, otherwise type the date and press enter.
    "#;
        println!("{p1}");

        let mut suffix = "";
        let mut d_date = "no_valid_value".to_string();
        while d_date == "no_valid_value".to_string() {
            let users_date_selection = user_input()?;
            if users_date_selection == "".to_string() {
                d_date = "".to_string();
                suffix = " (= default)";
            }
            else {
                match NaiveDate::parse_from_str(&users_date_selection, "%Y-%m-%d") {
                    Ok(nd) => {
                        let today = Local::now().date_naive();
                        if nd > today {
                            println!("{}", "The date entered is after today (!) - please try again");
                        }
                        else {
                            d_date = users_date_selection;
                        }
                    },
                    Err(_) => {
                        println!("{}", "The date entered does not conform to the ISO pattern (yyyy-MM-dd) required - please try again");
                    },
                }
            }
        }
        data_date = format!(r#"data_date="{d_date}""#);
        println!("    {data_date}{suffix}");
    }

    let data_section = format!("[data]\n{}\n{}\n{}\n", src_file_name, data_version, data_date);
    let folders_section = format!("[folders]\n{}\n{}\n{}\n", data_folder_path, output_folder_path, log_folder_path);
    let database_section = format!("[database]\n{}\n{}\n{}\n{}\n{}\n", db_host, db_user, db_password, db_port, db_name);
    let config_string = format!("\n{}\n\n{}\n\n{}\n", data_section, folders_section, database_section);
    write_out_file(&config_string)?;
    info!("Configuration file creation completed");
    Ok(())
}

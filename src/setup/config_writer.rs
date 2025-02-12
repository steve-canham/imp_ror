use crate::err::AppError;
use std::io;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;
use chrono::NaiveDate;

pub async fn create_config_file() -> Result<(), AppError>
{
    let prompt1 = " WELCOME TO IMP_ROR ";
    let star_line = "********************";
    let prompt2 = "The initial task is to set up an app_config file, to hold the details needed";
    let prompt3 = "to connect to the database, and some required folder paths.";
    let section = format!("\n\n{}\n{}\n{}\n{}\n{}\n", star_line, prompt1, star_line, prompt2, prompt3);
    println!("{}", section);


    let prompt4 = "Section 1: DATABASE PARAMETERS";
    let prompt5h = "DATABASE HOST";
    let prompt5 = "Please input the name of your database host (usually the server name or IP address).";
    let prompt7 = "To accept the default ('localhost') simply press enter, otherwise type the name and press enter.";
    let section = format!("\n{}\n\n{}\n{}\n{}\n", prompt4, prompt5h, prompt5, prompt7);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let mut host = input.trim();
    if host == "" {
        host = "localhost";
    }
    let db_host = format!("db_host=\"{}\"", host);
    println!("{}", db_host);

    let prompt8h = "USER NAME";
    let prompt8 = "Please input the name of the user account being used to access the database.";
    let prompt9 = "No default is available, type the name and press enter.";
    let section = format!("\n{}\n{}\n{}\n", prompt8h, prompt8, prompt9);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let user = input.trim();
    let db_user = format!("db_user=\"{}\"", user);
    println!("{}", db_user);

    let prompt10h = "USER PASSWORD";
    let prompt10 = "Please input the name of the user password being used to access the database.";
    let prompt11 = "No default is available, type the password and press enter.";
    let section = format!("\n{}\n{}\n{}\n", prompt10h, prompt10, prompt11);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let password = input.trim();
    let db_password = format!("db_password=\"{}\"", password);
    println!("{}", db_password);

    let prompt12h = "PORT";
    let prompt12 = "Please input the port number being used to access the database.";
    let prompt13 = "To accept the default ('5432') simply press enter, otherwise type the number and press enter.";
    let section = format!("\n{}\n{}\n{}\n", prompt12h, prompt12, prompt13);
    println!("{}", section);

    let mut port = -1;
    while port < 0 {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let try_port = input.trim();
        if try_port == "" {
            port = 5432;
        }
        else {
            port = match try_port.parse()
            {
                Ok(n) => n,
                Err(_) => {
                    println!("{}", "The port must be input as an integer!");
                    -1
                },
            };
        }
    }
    let db_port = format!("db_port=\"{}\"", port);
    println!("{}", db_port);

    let prompt14h = "DATABASE NAME";
    let prompt14 = "Please input the name of the database.";
    let prompt15 = "To accept the default ('ror') simply press enter, otherwise type the name and press enter.";
    let section = format!("\n{}\n{}\n{}\n", prompt14h, prompt14, prompt15);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let mut dname = input.trim();
    if dname == "" {
        dname = "ror";
    }
    let db_name = format!("db_name=\"{}\"", dname);
    println!("{}", db_name);


    let prompt16 = "Section 2: FOLDERS";
    let prompt17h = "DATA FOLDER";
    let prompt17 = "Please input the full path of the folder where the ROR JSON source file is to be found.";
    let prompt18 = "No default is available, type the path and press enter.";
    let section = format!("\n{}\n\n{}\n{}\n{}\n", prompt16, prompt17h, prompt17, prompt18);
    println!("{}", section);

    let mut df = "".to_string();
    while df == "".to_string() {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let try_df = input.trim().replace("\\", "\\\\");
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
   
    let prompt19h = "OUTPUTS FOLDER";
    let prompt19 = "Please input the full path of the folder where the outputs from the program should be placed.";
    let prompt20 = "To accept the default (the 'DATA FOLDER') simply press enter, otherwise type the path and press enter.";
    let section = format!("\n{}\n{}\n{}\n", prompt19h, prompt19, prompt20);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let output_folder = input.trim();
    let output_folder_path: String;
    if output_folder == "" {
        output_folder_path = format!("output_folder_path=\"{}\"", data_folder);
    }
    else {
        let op_folder = output_folder.to_string().replace("\\", "\\\\");
        output_folder_path = format!("output_folder_path=\"{}\"", op_folder);
    }
    println!("{}", output_folder_path);

    let prompt21h = "LOG FOLDER";
    let prompt21 = "Please input the full path of the folder where the logs from the program should be placed.";
    let prompt22 = "To accept the default (the 'DATA FOLDER') simply press enter, otherwise type the path and press enter.";
    let section = format!("\n{}\n{}\n{}\n", prompt21h, prompt21, prompt22);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let log_folder = input.trim();
    let log_folder_path: String;
    if log_folder == "" {
        log_folder_path = format!("log_folder_path=\"{}\"", data_folder);
    }
    else {
        let lg_folder = log_folder.to_string().replace("\\", "\\\\");
        log_folder_path = format!("log_folder_path=\"{}\"", lg_folder);
    }
    println!("{}", log_folder_path);


    let prompt23 = "Section 3: SOURCE FILE";
    let prompt24h = "SOURCE FILE NAME";
    let prompt24 = "Normally the source file is provided as a command line argument.";
    let prompt25 = "You can define it in the configuration, however, which means it does not have to be given in the command line.";
    let prompt26 = "Note that any source file provided in the command line will over-write the value in the config file.";
    let prompt27 = "To always use the command line simply press enter, otherwise type the source file name and press enter.";
    let section = format!("\n{}\n\n{}\n{}\n{}\n{}\n{}\n", prompt23, prompt24h, prompt24, prompt25, prompt26, prompt27);
    println!("{}", section);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let src_file = input.trim();
    let src_file_name = format!("src_file_name=\"{}\"", src_file);
    println!("{}", src_file_name);

    let mut data_version = format!("data_version=\"\"");
    let mut data_date = format!("data_date=\"\"");

    if src_file != "" {

        let prompt28 = "As you have stored a source file name in the configuration you may need to also store";
        let prompt29 = "the associated data version and date. These can be left as the defaults (empty strings)";
        let prompt30 = "if the version and date can be parsed from the source file name (see documentation for the required pattern).";
        let section = format!("\n{}\n{}\n{}\n", prompt28, prompt29, prompt30);
        println!("{}", section);


        let prompt31h = "DATA VERSION";
        let prompt31 = "Please input the data version, as a 'v' followed by the version number, e.g. '1.56.1'.";
        let prompt32 = "To accept the default (an empty string) simply press enter, otherwise type the version and press enter.";
        let section = format!("\n{}\n{}\n{}\n\n", prompt31h, prompt31, prompt32);
        println!("{}", section);

        let mut d_version = "zzzz".to_string();
        while d_version == "zzzz".to_string() {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let d_v = input.trim();
            if d_v == "" || is_compliant_version(d_v) {
                d_version = d_v.to_string();
            }
            else {
                println!("{}", "The version entered does not conform to the pattern required - please try again");
            }
        }
        data_version = format!("data_version=\"{}\"", d_version);
        println!("{}", data_version);

        // check starts with a v and has following digits / decimal points
       
        let prompt33h = "DATA DATE";
        let prompt33 = "Please input the data date as an ISO string, yyyy-MM-dd, e.g. '2025-07-22'.";
        let prompt34 = "To accept the default (an empty string) simply press enter, otherwise type the date and press enter.";
        let section = format!("\n{}\n{}\n{}\n", prompt33h, prompt33, prompt34);
        println!("{}", section);

        let mut d_date = "zzzz".to_string();
        while d_date == "zzzz".to_string() {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let d_d = input.trim();
            if d_d == "" || NaiveDate::parse_from_str(d_d, "%Y-%m-%d").is_ok() {
                d_date = d_d.to_string();
            }
            else {
                println!("{}", "The date entered does not conform to the ISO pattern (yyyy-MM-dd) required - please try again");
            }
        }
        data_date = format!("data_date=\"{}\"", d_date);
        println!("{}", data_date);

        // check is a date
    }

    let database_section = format!("[database]\n{}\n{}\n{}\n{}\n{}\n", db_host, db_user, db_password, db_port, db_name);

    let files_section = format!("[files]\n{}\n{}\n{}\n{}\n", data_folder_path, output_folder_path, log_folder_path, src_file_name);

    let data_section = format!("[data]\n{}\n{}", data_version, data_date);
       
    let config_string = format!("\n{}\n\n{}\n\n{}\n", data_section, files_section, database_section);

    println!("{}", config_string);

    let mut file = File::create("./app_config.toml")     // creates new or truncates existing
        .expect("Failed to create or open the file");

    // Write to the file
    file.write_all(config_string.as_bytes())
        .expect("Failed to write to the file");
    println!("Data written to file successfully!");

    Ok(())
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


fn is_compliant_version(input: &str) -> bool {
    let version_pattern = r#"^v[0-9]+(\.[0-9]+){0,2}"#;
    let re = Regex::new(version_pattern).unwrap();
    re.is_match(&input) 
}



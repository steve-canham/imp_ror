use thiserror::Error;
use log::error;
use crate::setup::log_set_up;


// The error types used within the program.

#[derive(Error, Debug)]
pub enum AppError {

    #[error("Error in configuration file: {0:?} {1:?} ")]
    ConfigurationError(String, String),

    #[error("Database Parameters Unavailable")]
    MissingDBParameters(),

    #[error("The parameter '{0}' is required, but has not been supplied")]
    MissingProgramParameter(String),

    #[error("The parameters provided are inconsistent or incompatible")]
    InconsistentProgramParameter(String),

    #[error("The version specified does not match the version currently strored")]
    IncompatibleVersions(String, String),

    #[error("The version specified does not yet exist in the summary tables")]
    MissingVersion(String),

    #[error("couldn't read file {1:?}")]
    IoReadErrorWithPath(#[source] std::io::Error, std::path::PathBuf,),

    #[error("couldn't write file {1:?}")]
    IoWriteErrorWithPath(#[source] std::io::Error, std::path::PathBuf,),

    #[error("Problem accessing folder or file")]
    FileSystemError(String, String),

    #[error("Error when setting up log configuration: {0:?} {1:?}")]
    LogSetupError(String, String),

    #[error("Error when processing command line arguments: {0:?}")]
    ClapError(#[from] clap::Error),

    #[error("JSON processing error: {0:?}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Error when creating a DB Pool: {1:?}")]
    DBPoolError(String, #[source] sqlx::Error,),

    #[error("Error when processing sql: {0:?}")]
    SqlxError(#[source] sqlx::Error, String),

    #[error("Error when using regex: {0:?}")]
    RegexError(#[source] regex::Error, String),

    #[error("Error reading user input: {0:?}")]
    UserInputError (#[from] std::io::Error),
}


pub fn report_error(e: AppError) -> () {

    match e {
        AppError::ConfigurationError(p, d) => print_error (p, d, "CONFIGURATION ERROR"),

        AppError::ClapError(e) => print_error ("Error occureed when parsing CLI argumants".to_string(), 
                    e.to_string(), "CLAP ERROR"),

        AppError::MissingDBParameters() => print_error ("Unable to obtain database parameters.".to_string(),
                    "Attempting to read OnceLock<DB_PARS>".to_string(), "DB PARAMETERS ERROR"),            

        AppError::MissingProgramParameter(p) =>  print_error (
                  "A required parameter is neither in the config file nor the command line arguments".to_string(), 
                  format!("Parameter is: {}", p), "MISSING PARAMETER"),

        AppError::InconsistentProgramParameter(s)  =>  print_error (
                 "The parameters provided are inconsistent or incompatible".to_string(), 
                 s, "INCONSISTENT PARAMETERS"),

        AppError::IncompatibleVersions(v_requested, v_stored)  =>  print_error (
                    format!("The version specified ('{}'), does not match the data stored in the ror schema ('{}').", v_requested, v_stored),
                    " Run -r or -a with the specified version, to re-import the data and allow its processing and summarising.".to_string(), 
                    " INCOMPATIBLE VERSIONS"),

        AppError::MissingVersion(v_requested)  =>  print_error (
                        format!("Data for the version specified ('{}') does not yet exist in the summary tables.", v_requested),
                        " Run -r or -a with the specified version, to import the data and allow its processing and summarising.".to_string(), 
                        " MISSING VERSION"),

        AppError::LogSetupError(p, d) => print_error (p, d, "LOG SETUP ERROR"),

        AppError::IoReadErrorWithPath(e, p) => print_error (e.to_string(), 
                  "Path was: ".to_string() + p.to_str().unwrap(), "FILE READING PROBLEM"),
        
        AppError::IoWriteErrorWithPath(e, p) => print_error (e.to_string(), 
                  "Path was: ".to_string() + p.to_str().unwrap(), "FILE WRITING PROBLEM"),

        AppError::FileSystemError(p, d) => print_error (p, d, "FILE SYSTEM PROBLEM"),
        
        AppError::SerdeError(e) => print_error ("Error occureed when parsing JSON file".to_string(), 
                    e.to_string(), "SERDE JSON ERROR"),
        
        AppError::DBPoolError(d, e) => print_error(d, e.to_string(), "DB POOL ERROR"),
  
        AppError::SqlxError(e, s) => print_error (e.to_string(), 
                        format!("SQL was: {}", s),  "SQLX ERROR"),

        AppError::RegexError(e, d) => print_error(e.to_string(), d, "REGEX ISSUE"),
   
        AppError::UserInputError(e) => print_simple_error (e.to_string(), "USER INPUT ERROR"),
    }
}


fn print_error(description: String, details: String, header: &str) {
    let star_num = 100;
    let hdr_line = get_header_line (star_num, &header);
    let starred_line = str::repeat("*", star_num);
    let err_output = format!("\n{}\n{}\n{}\n{}\n\n", hdr_line, description, details, starred_line);
    output_error(err_output);
}

fn print_simple_error(msg: String, header: &str) {
    let star_num = 100;
    let hdr_line = get_header_line (star_num, &header);
    let starred_line = str::repeat("*", star_num);
    let err_output = format!("\n{}\n{}\n{}\n\n", hdr_line, msg, starred_line);
    output_error(err_output);
}

fn get_header_line (star_num: usize, header: &str) -> String {
    let hdr_len = header.len();
    let mut spacer = "";
    if hdr_len % 2 != 0  {
        spacer = " ";
    }
    let star_batch_num = (star_num - 2 - hdr_len) / 2;
    let star_batch = str::repeat("*", star_batch_num);
    format!("{} {}{} {}", star_batch, header, spacer, star_batch)
}

fn output_error (err_output: String) {

    eprint!("{}", err_output);

    if log_set_up(){
        error!("{}", err_output);
    }

    // Not intended to run unattended (at the moment)
    // system independent error logging therefore not yet required.

}


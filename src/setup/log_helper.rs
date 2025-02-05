/***************************************************************************
 * Establishes the log for the programme's operation using log and log4rs, 
 * and includes various helper functions.
 * Once established the log file appears to be accessible to any log
 * statement within the rest of the program (after 'use log:: ...').
 ***************************************************************************/

use chrono::Local;
use std::path::PathBuf;
use crate::error_defs::AppError;
use crate::setup::InitParams;

use log::{info, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

pub fn setup_log (data_folder: &PathBuf, source_file_name : &PathBuf) -> Result<log4rs::Handle, AppError> {
    let log_file_path = get_log_file_path(data_folder, source_file_name);
    config_log (&log_file_path)
}

fn get_log_file_path(data_folder: &PathBuf, source_file_name : &PathBuf) -> PathBuf {
    
    // Derives the log file name, returns the full path

    let datetime_string = Local::now().format("%m-%d %H%M%S").to_string();
    let mut log_file_name = format!("ror {} ", datetime_string);
    let source_file_string = source_file_name.display().to_string();
    if source_file_string != "" {
        let source_file = &source_file_string[..(source_file_string.len() - 5)];
        log_file_name = format!("{} from {}.log", log_file_name, source_file);
    }
    else {
        log_file_name = format!("{} initialisation.log", log_file_name);
    }
    [data_folder, &PathBuf::from(&log_file_name)].iter().collect()
    
}

fn config_log (log_file_path: &PathBuf) -> Result<log4rs::Handle, AppError> {
    
    // Initially establish a pattern for each log line.

    let log_pattern = "{d(%d/%m %H:%M:%S)}  {h({l})}  {({M}.{L}):>38.48}:  {m}\n";

    // Define a stderr logger, as one of the 'logging' sinks or 'appender's.

    let stderr = ConsoleAppender::builder().encoder(Box::new(PatternEncoder::new(log_pattern)))
        .target(Target::Stderr).build();

    // Define a second logging sink or 'appender' - to a log file (provided path will place it in the current data folder).

    let try_logfile = FileAppender::builder().encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build(log_file_path);
    let logfile = match try_logfile {
        Ok(lf) => lf,
        Err(e) => return Err(AppError::IoErr(e)),
    };

    // Configure and build log4rs instance, using the two appenders described above

    let config = Config::builder()
        .appender(Appender::builder()
                .build("logfile", Box::new(logfile)),)
        .appender(Appender::builder()
                .build("stderr", Box::new(stderr)),)
        .build(Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Info),
        ).unwrap();

    match log4rs::init_config(config)
    {
        Ok(h) => return Ok(h),
        Err(e) => return Err(AppError::LgErr(e)),
    };

}


pub fn log_startup_params (ip : &InitParams) {
    
    // Called at the end of set up to record the input parameters

    info!("PROGRAM START");
    info!("");
    info!("************************************");
    info!("");
    info!("data_folder: {}", ip.data_folder.display());
    info!("log_folder: {}", ip.log_folder.display());
    info!("output_folder: {}", ip.output_folder.display());
    info!("source_file_name: {}", ip.source_file_name.display());
    info!("output_file_name: {}", ip.output_file_name.display());
    info!("data_version: {}", ip.data_version);
    info!("data_date: {}", ip.data_date);
    info!("create look up tables: {}", ip.flags.create_lookups);
    info!("create summary tables: {}", ip.flags.create_summary);
    info!("import_ror: {}", ip.flags.import_ror);
    info!("process_data: {}", ip.flags.process_data);
    info!("export_text: {}", ip.flags.export_text);
    info!("export_csv: {}", ip.flags.export_csv);
    info!("export_all_csv: {}", ip.flags.export_full_csv);
    info!("");
    info!("************************************");
    info!("");
}

// Module uses clap crate to read command line arguments. 

use clap::{command, Arg, ArgMatches};
use crate::err::AppError;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CliPars {
    pub source_file: String,
    pub data_version: String,
    pub data_date: String,
    pub flags: Flags, 
    pub test_folder: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub import_ror: bool,
    pub export_csv: bool,
    pub export_all_csv: bool,
    pub inc_withdrawn: bool,
    pub create_config: bool,
    pub create_lookups: bool,
    pub create_summary: bool,
    pub test_run: bool,
}

pub fn fetch_valid_arguments(args: Vec<OsString>) -> Result<CliPars, AppError> {

    let parse_result = parse_args(args.to_vec())?;

    // The string parameters below guaranteed to unwrap OK as all have a default value of "".

    let source_file = parse_result.get_one::<String>("src_file").unwrap();
    let data_version = parse_result.get_one::<String>("data_version").unwrap();
    let data_date = parse_result.get_one::<String>("data_date").unwrap();
    let test_folder_as_string = parse_result.get_one::<String>("test_folder").unwrap();
    let test_folder = PathBuf::from(test_folder_as_string);
    
    // Flag values are false if not present, true if present.
    
    let mut a_flag = parse_result.get_flag("a_flag");
    let mut x_flag = parse_result.get_flag("x_flag");
    let mut y_flag = parse_result.get_flag("y_flag");
    let w_flag = parse_result.get_flag("w_flag");
    let i_flag = parse_result.get_flag("i_flag");
    let mut c_flag = parse_result.get_flag("c_flag");
    let mut k_flag = parse_result.get_flag("k_flag");
    let mut m_flag = parse_result.get_flag("m_flag");
    let mut t_flag = parse_result.get_flag("t_flag");
        
    if i_flag {
        c_flag = true;
        k_flag = true;
        m_flag = true;
    }

    // If c, m, k or all three flags set (may be by using 'i' initialise flag)
    // Only the k and / or c and / or m actions allowed
      
    if k_flag || m_flag || c_flag {
        a_flag = false;
        x_flag = false;
        y_flag = false;
        t_flag = false;        
    }

    // If a test run check a meaningful folder for test data
    // and set other flags to import data
    
    else if t_flag {   
        match test_folder.try_exists() {
            Ok(true) => (),
            _ => return           // includes Ok(false) as well as Err
               Result::Err(AppError::MissingProgramParameter("valid test folder".to_string())),   
        }
        a_flag = true;   
        x_flag = false;
        y_flag = false;
    }

    // More usual situation is -a, -x, or -y, possibly with -w.
    // -a flag can be accompanied by -x or -y (will be done first)
    // If -x and -y flags both given, only -y is allowed.
    // If none of a, x, y or t flags set a to true, as the default - this
    // will also need the source file designated, perhaps in config file.
    
    else 
    {
        if x_flag && y_flag {
            x_flag = false;
        }
       
        if !a_flag && !x_flag && !y_flag {
            a_flag = true;   
        }
    }

    let flags = Flags {
        import_ror: a_flag,
        export_csv: x_flag,
        export_all_csv: y_flag,
        create_config: c_flag,
        create_lookups: k_flag,
        create_summary: m_flag,
        inc_withdrawn: w_flag,
        test_run: t_flag,
    };

    Ok(CliPars {
        source_file: source_file.clone(),
        data_version: data_version.clone(),
        data_date: data_date.clone(),
        test_folder: test_folder.clone(),
        flags: flags,
    })
}

fn parse_args(args: Vec<OsString>) -> Result<ArgMatches, clap::Error> {

    command!()
        .about("Imports data from ROR json file (v2) and imports it into a database")
        .arg(
             Arg::new("src_file")
            .short('f')
            .long("file")
            .visible_aliases(["source file"])
            .help("A string with the source file name (over-rides any config file value)")
            .default_value("")
        )
        .arg(
            Arg::new("data_version")
           .short('v')
           .long("data_version")
           .required(false)
           .help("A string with the version ascribed to the data by ror, in a semver format")
           .default_value("")
        )
        .arg(
            Arg::new("data_date")
           .short('d')
           .long("date")
           .required(false)
           .help("A string with a date in ISO format that gives the date of the data")
           .default_value("")
        )
        .arg(
            Arg::new("a_flag")
           .short('a')
           .long("all")
           .required(false)
           .help("A flag signifying import, process, and generate report for designated version, excluding withdrawn organisations")
           .action(clap::ArgAction::SetTrue)
         )
         .arg(
             Arg::new("x_flag")
            .short('x')
            .long("export")
            .required(false)
            .help("A flag signifying output a summary of the current or specified version into csv files")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("y_flag")
           .short('y')
           .long("export_all")
           .required(false)
           .help("A flag signifying output a summary of the data for all versions into csv files")
           .action(clap::ArgAction::SetTrue)
       )
       .arg(
           Arg::new("w_flag")
          .short('w')
          .long("inc_wd")
          .required(false)
          .help("A flag signifying retain withdrawn organisations if import, or use wd included versions if export")
          .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("i_flag")
            .short('i')
            .long("init")
            .required(false)
            .help("A flag signifying that the system should be initialised (= -c, -k, -m)")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("c_flag")
            .short('c')
            .long("config")
            .required(false)
            .help("A flag signifying that a configuration file needs to be built or edited")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("k_flag")
            .short('k')
            .long("lookup")
            .required(false)
            .help("A flag signifying that look up tables need to be rebuilt")
            .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("m_flag")
            .short('m')
            .long("summ_setup")
            .required(false)
            .help("A flag signifying that summary tables should be recreated")
            .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("t_flag")
            .short('t')
            .long("test")
            .required(false)
            .help("A flag signifying that this is part of an integration test run - suppresses logs")
            .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("test_folder")
            .short('u')
            .long("test_folder")
            .help("A CLI derived source folder for testing purposes")
            .default_value("")
        )
    .try_get_matches_from(args)

}


#[cfg(test)]
mod tests {
    use super::*;
    
    // Ensure the parameters are being correctly extracted from the CLI arguments

    #[test]
    fn check_cli_no_explicit_params() {
        let target = "dummy target";
        let args: Vec<&str> = vec![target];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "");
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }
  
    #[test]
    fn check_cli_with_a_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-a"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "");
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }
   

    #[test]
    fn check_cli_with_ckm_flags() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-c", "-k", "-m"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "");
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, true);
        assert_eq!(res.flags.create_lookups, true);
        assert_eq!(res.flags.create_summary, true);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }
    

    #[test]
    fn check_cli_with_t_flag_and_folder() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target,  "-t", "-u", "/home/steve/Data/Repos/imp_ror/tests/test_data"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "");
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, true);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }

    
    #[test]
    #[should_panic]
    fn check_cli_with_t_flag_and_no_folder_panics() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target,  "-t"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let _res = fetch_valid_arguments(test_args).unwrap();
    }
    

    #[test]
    fn check_cli_with_m_and_x_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-m", "-x"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "");
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, true);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }


    #[test]
    fn check_cli_with_x_and_y_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-x", "-y"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "");
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, true);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }
      

    #[test]
    fn check_cli_with_string_pars() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-f", "schema2.1 data.json", "-d", "2026-12-25", "-v", "v1.63"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "schema2.1 data.json");
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "2026-12-25");
        assert_eq!(res.data_version, "v1.63");
    }


    #[test]
    fn check_cli_with_most_params_explicit() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-f", "schema2.1 data.json", "-d", "2026-12-25", 
                                            "-v", "v1.63", "-x", "-y", "-a"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, "schema2.1 data.json");
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_all_csv, true);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "2026-12-25");
        assert_eq!(res.data_version, "v1.63");
    }

}


/***************************************************************************
 * Module uses clap crate to read command line arguments. These include 
 * possible A, S, T and C flags, and possible strings for the data folder and 
 * source file name. If no flags 'S' (= import data) is returned by default.
 * Folder and file names return an empty string ("") rather than null if not 
 * present. 
 ***************************************************************************/

use clap::{command, Arg, ArgMatches};
use crate::err::AppError;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CliPars {
    pub source_file: PathBuf,
    pub data_version: String,
    pub data_date: String,
    pub flags: Flags, 
    pub test_folder: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub import_ror: bool,
    pub process_data: bool,
    pub export_text: bool,
    pub export_csv: bool,
    pub export_full_csv: bool,
    pub create_config: bool,
    pub create_lookups: bool,
    pub create_summary: bool,
    pub test_run: bool,
}

pub fn fetch_valid_arguments(args: Vec<OsString>) -> Result<CliPars, AppError>
{ 
    let parse_result = parse_args(args.to_vec())?;

    // These parameters guaranteed to unwrap OK as all have a default value of "".

    let source_file_as_string = parse_result.get_one::<String>("src_file").unwrap();
    let source_file = PathBuf::from(source_file_as_string);

    let test_folder_as_string = parse_result.get_one::<String>("test_folder").unwrap();
    let test_folder = PathBuf::from(test_folder_as_string);

    let data_version = parse_result.get_one::<String>("data_version").unwrap();
    let data_date = parse_result.get_one::<String>("data_date").unwrap();

    // Flag values are false if not present, true if present.

    let a_flag = parse_result.get_flag("a_flag");
    let i_flag = parse_result.get_flag("i_flag");

    let mut r_flag = parse_result.get_flag("r_flag");
    let mut p_flag = parse_result.get_flag("p_flag");
    let mut t_flag = parse_result.get_flag("t_flag");
    let mut x_flag = parse_result.get_flag("x_flag");
    let mut y_flag = parse_result.get_flag("y_flag");
    let mut j_flag = parse_result.get_flag("j_flag");
    let mut c_flag = parse_result.get_flag("c_flag");
    let mut m_flag = parse_result.get_flag("m_flag");
    let mut z_flag = parse_result.get_flag("z_flag");

    // If c, m, j or all three flags set (may be by using 'i' (initialise) flag)
    // Only do the j and / or c and / or m actions
  
    if i_flag || c_flag || m_flag || j_flag {
        
        if i_flag {
            c_flag = true;
            m_flag = true;
            j_flag = true;
        }

        r_flag = false;
        p_flag = false;
        t_flag = false;
        x_flag = false;
        y_flag = false;
        z_flag = false;        
    }
    
    else {

        if a_flag  // 'a' (do all) flag set
        {
            r_flag = true;  
            p_flag = true;
            t_flag = true;
        }
        else 
        {
            // if none of r, p, t, x or y flags set
            // set r to be true, as the default with no flags

            if r_flag == false && p_flag == false && t_flag == false
                && x_flag == false && y_flag == false {
                r_flag = true;  
            }
        }
    }

    let flags = Flags {
        import_ror: r_flag,
        process_data: p_flag,
        export_text: t_flag,
        export_csv: x_flag,
        export_full_csv: y_flag,
        create_config: j_flag,
        create_lookups: c_flag,
        create_summary: m_flag,
        test_run: z_flag,
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
            .short('s')
            .long("source")
            .visible_aliases(["source file"])
            .help("A string with the source file name (over-rides environment setting")
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
           .help("A flag signifying run the entire program, equivalent to R and P")
           .action(clap::ArgAction::SetTrue)
         )
        .arg(
            Arg::new("r_flag")
           .short('r')
           .long("import")
           .required(false)
           .help("A flag signifying import from ror file to ror schema tables only")
           .action(clap::ArgAction::SetTrue)
        )
        .arg(
             Arg::new("p_flag")
            .short('p')
            .long("process")
            .required(false)
            .help("A flag signifying process ror data to src data and analyse and store results")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("t_flag")
           .short('t')
           .long("textout")
           .required(false)
           .help("A flag signifying output a summary of the current or specified version into a text file")
           .action(clap::ArgAction::SetTrue)
       )
       .arg(
             Arg::new("x_flag")
            .short('x')
            .long("filesout")
            .required(false)
            .help("A flag signifying output a summary of the current or specified version into csv files")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("y_flag")
           .short('y')
           .long("allfilesout")
           .required(false)
           .help("A flag signifying output a summary of the data for all versions into csv files")
           .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("i_flag")
           .short('i')
           .long("install")
           .required(false)
           .help("A flag signifying initial run, creates summary and context tables only")
           .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("j_flag")
            .short('j')
            .long("config")
            .required(false)
            .help("A flag signifying that a configurastion file needs to be built or edited")
            .action(clap::ArgAction::SetTrue)
        )
       .arg(
            Arg::new("c_flag")
            .short('c')
            .long("context")
            .required(false)
            .help("A flag signifying that context tables need to be rebuilt")
            .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("m_flag")
            .short('m')
            .long("summsetup")
            .required(false)
            .help("A flag signifying that summary tables should be recreated")
            .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("z_flag")
            .short('z')
            .long("test")
            .required(false)
            .help("A flag signifying that this is part of an integration test run - suppresses logs")
            .action(clap::ArgAction::SetTrue)
       )
       .arg(
            Arg::new("test_folder")
            .short('f')
            .long("folder")
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
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
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
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.process_data, true);
        assert_eq!(res.flags.export_text, true);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }

    #[test]
    fn check_cli_with_i_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-i"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, true);
        assert_eq!(res.flags.create_lookups, true);
        assert_eq!(res.flags.create_summary, true);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }

    #[test]
    fn check_cli_with_jcm_flags() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-j", "-c", "-m"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, true);
        assert_eq!(res.flags.create_lookups, true);
        assert_eq!(res.flags.create_summary, true);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }

    #[test]
    fn check_cli_with_c_and_p_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-c", "-p"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, true);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }

    #[test]
    fn check_cli_with_j_and_t_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-j", "-t"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, true);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }
    

    #[test]
    fn check_cli_with_m_and_r_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-m", "-r"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
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
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, false);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, true);
        assert_eq!(res.flags.export_full_csv, true);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }


    #[test]
    fn check_cli_with_rpt_and_x_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-r", "-p", "-t", "-x"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.process_data, true);
        assert_eq!(res.flags.export_text, true);
        assert_eq!(res.flags.export_csv, true);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }

    
    #[test]
    fn check_cli_with_z_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-z"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::new());
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, true);
        assert_eq!(res.data_date, "");
        assert_eq!(res.data_version, "");
    }


    #[test]
    fn check_cli_with_string_pars() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-s", "schema2.1 data.json", "-d", "2026-12-25", "-v", "v1.63"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::from("schema2.1 data.json"));
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.process_data, false);
        assert_eq!(res.flags.export_text, false);
        assert_eq!(res.flags.export_csv, false);
        assert_eq!(res.flags.export_full_csv, false);
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
        let args : Vec<&str> = vec![target, "-s", "schema2.1 data.json", "-d", "2026-12-25", 
                                            "-v", "v1.63", "-r", "-p", "-t", "-x", "-y", "-z"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_file, PathBuf::from("schema2.1 data.json"));
        assert_eq!(res.flags.import_ror, true);
        assert_eq!(res.flags.process_data, true);
        assert_eq!(res.flags.export_text, true);
        assert_eq!(res.flags.export_csv, true);
        assert_eq!(res.flags.export_full_csv, true);
        assert_eq!(res.flags.create_config, false);
        assert_eq!(res.flags.create_lookups, false);
        assert_eq!(res.flags.create_summary, false);
        assert_eq!(res.flags.test_run, true);
        assert_eq!(res.data_date, "2026-12-25");
        assert_eq!(res.data_version, "v1.63");
    }

}


## Installation and Configuration

The system is designed to be flexible, but uses reasonable defaults to make routine use straightforward. 
For the moment, some basic knowledge of running Postgres, and of using Rust in a development environment, is required.

### Installing the system and pre-requisites
1. Download and install Rust, and download the imp_ror code from this GitHub page, accessing it within a Rust development environment. VS Code, with Rust extensions installed, is recommended, and is used by about 75% of current Rust developers. The newer Zed IDE, itself written in Rust, is also a good choice.
2. Install Postgres if not already available and establish an empty database (by default called 'ror', though any name can be used). The empty database must be created prior to the initial run of the system, but all other database operations are handled by the system. Select or create a Postgres user account to be used when running imp_ror. This account will obviously need full access to the ror database. 
3. Download the v2 ror json files required, from Zenodo, and place them in the folder to be used as the source data folder.

### Configuration file location

The system uses a configuration file (config.toml) to provide details of the database connection, and the folders holding data, output files amd logs. The file is located in the conventional location for application configuration files, which is OS dependent. The locations are:

- Linux:   /home/&lt;user name&gt;/.config/imp_ror/config.toml  
- Windows: C:\Users\\&lt;user name&gt;\AppData\Roaming\canhamis\imp_ror\config.toml
- macOS:   /Users/&lt;user name&gt;/Library/Application Support/eu.canhamis.imp_ror/config.toml

The assumption is that these locations can be created, and are available and accessible to the user (they normally would be). Note that different users on the same machine would each need to establish their own configuaration file. 

### Initialising the system

The initial run of the system must create the configuration file, as well as establish lookup tables and a new set of summary tables. In the development environment (e.g. in the terminal in VS Code or Zed) first run the system with the '-i' initialisation flag    
***cargo run -- -i***  

All Rust development environments use a program called 'cargo' to manage code, so 'cargo run' will simply build an .exe file from the source code and then run it, taking into account any parameters provided. Note that a double hyphen separator is required between 'cargo' and the program's -i parameter - this is to differentiate parameters for the program from those for cargo itself.  

Under a banner of 'INITIAL CONFIGURATION SET UP' the system will ask for various parameters. These should be typed in at each '&gt;&gt;' prompt (or the default accepted when applicable) in response to the program's questions. The first group of data points required are:

#### Database parameters

A cluster of data points are required to create a database connection each time the program starts. Once established, they should rarely need to be changed. 
- The postgres database host. This is normally the server name or IP address. For a local Postgres installation, as often used in initial testing, it can be 'localhost', and this is available as a default (obtained by simply pressing return).
- The postgres user name. The name of the Postgres user account used to access the 'ror' database. That account obviously needs full access to the database. It may be a general admin account, as set up during Postgres installation, or an account specifically setup for managing the ror data. No default is available so this parameter must be typed in.
- The postgres user password. The password of the user account above. No default is available so this must be provided.
- The postgres port number. An integer representing the port number used by the server. By default this is '5432' (the standard Postgres port) but it needs to match whatever has been set up on the server during Postgres installation.
- The postgres database name. By default this is 'ror' (available by simply pressing return) but it should match the name of the database that has been creeated on the postgres server.

#### Folder parameters

The second group of parameters relates to folders where program inputs are to be found, and outputs to be located. They are:
- The path of the folder where the ROR data files will be stored. This 'data folder' must be provided. It can be expressed in either 'Windows' form, with back slashes between path segments, or in 'posix' form, as used on linux and Mac machines, with forward slashes.
- The path of the folder where the program will put its outputs (text and csv files). *Note that the user must have write permissions for this folder.* If not provided, i.e. the user simply presses return, this value will default to be the same as the data folder already provided above.
- The path of the folder where the program will put its logs (a log is generated for each run of the program). *Note that the user must have write permissions for this folder.* If not provided, i.e. the user simply presses return, this value will default to be the same as the data folder already provided above.

#### Source data parameters

The final group of parameters relate to the source data and are optional, in the sense that they are more normally provided in the command line, but there are times (e.g. if testing) when it can be useful to include them in the configuration. They include:
- The name of the source file, as downloaded from ROR. This has a standard name pattern, with a version indicator followed by a date indicator followed by some text.  
If the intention is to always use the command line to indicate the source file this can be left as an empty string by simply pressing return. In that case the two remaining parameters (data version and data date) are also set as empty strings and the configuration process ends.  
N.B. Even when a source file name is provided, any source file in the command line will take precedence over and 'mask' one provided in the configuration file.  
- The ROR version of the file to be imported. This is a string, with a 'v' followed by a set of numbers in a semantic versioning format, e.g. 'v1.45.1', 'v1.57', 'v2.1'. In most cases, including when retaining the name as downloaded from ROR, the system can parse this from the source file name (see Operation and Commands for details).  
In these circumstances it can be stored in the configuration file as an empty string by simply pressing return.
- The date of the file to be imported, as 'data_date'. This should be in the YYYY-mm-DD ISO format. In most cases, including when retaining the name as downloaded from ROR, the system can parse this from the source file name.  
In these circumstances it can be stored in the configuration file as an empty string by simply pressing return.

#### Final setup actions

Once these questions have been answered the system creates a new configuration file and prints a copy of it to the log.
It then immediately uses the database connection parameters provided to connect to the database and creates:
- the lookup tables, with their data, and 
- the summary tables, which at this stage are empty.  

The system is then ready for routine use.

#### Editing or replacing the configuration file

Note that the configuration file can be edited at any time using '**cargo run -- -c**'.  
During an edit existing values become the default and can be re-applied by pressing return at the prompt. Only the values that need to be changed need to be re-typed.  
If the config file is deleted, moved or renamed so that the system cannot find it, running 'cargo run -- -c' will allow it to be re-established.  
Running 'cargo run -- -i' again will do the same, but it will also recreate the summary tables, dropping any existing sunmmary data. This might not be what is required.

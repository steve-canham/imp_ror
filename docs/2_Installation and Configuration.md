<h2>Installation and Configuration</h2>

The system is designed to be flexible, but uses reasonable defaults to make routine use straightforward. 
For the moment, some basic familiarity with running Postgres and Rust (in a development environment) is required.

<h3>Installing the system and pre-requisites</h3>
a) Download and install Rust, and download the code from this GitHub page, accessing it within a Rust development environment. VS Code, with Rust extensions installed, 
is strongly recommended, and is used by about 75% of current Rust developers.<br/>
b) Install Postgres if not already available and establish an empty database (by default called 'ror', though any name cn be used). The empty database must be
created prior to the initial run of the system, but all other database operations are handled by the system.<br/>
c) Download the ror files required, from Zenodo, and place the V2 json files in the folder to be used as the source data folder.<br/>

<h3>Initialising the system</h3>
The system uses a configuration file (app_config.toml) to provide details of the database connection, and the folders holding data, output files amd logs. This configuration file is established of the intial running of the system. It is not stored on GitHub as it contains sensitive information.<br/>
<i>The initial run:</i>
<ul>
<li>Run the system without any parameters. In VS Code, this involves inputting 'cargo run' in the terminal - other development environments will have other mechanisms to trigger the same command. All Rust development environments use a program called <i>cargo</i> to manage code, so 'cargo run' will simply build an .exe file from the source code and then run it.</li>
<li>Because the program will not be able to find a configuration file it will automatically switch to 'initialisation mode'. </li> 
<li>Under a banner of 'INITIAL CONFIGURATION SET UP' the system will ask for various parameters. These should be typed in at each '&gt;&gt;' prompt (or the default accepted when applicable) in response to the program's questions. The first group of data points required are:
<ul>
    <li>The postgres database host. This is normally the server name or IP address. For a local Postgres installation, as often used in initial testing, it can be 'localhost', and this is available as a default (obtained by simply pressing return).</li>
    <li>The postgres user name. The name of the Postgres user account used to access the 'ror' database. That account obviously needs full access to the database. It may be a general admin account, as set up during Postgres installation, or an account specifically setup for managing the ror data. No default is available so this parameter must be typed in.</li>
    <li>The postgres user password. The password of the user account above. No default is available so this must be provided.</li>
    <li>The postgres port number. An integer representing the port number used byb the server. By default this is '5432' (the standard Postgres port) but it needs to match whatever has been set up on the server during Postgres installation.</li>
    <li>The postgres database name. By default this is 'ror' (available by simply pressing return) but it should match the name of the database that has been creeated on the postgres server.</li>
</ul>
<li>Those data points are used (all are necessary) to create a database connection each time the program starts. Once established, they should rarely need to be changed. The second group of parameters are:</li>
<ul>
    <li>The path of the folder where the ROR data files will be stored. This 'data folder' must be provided. It can be expressed in either 'Windows' form, with back slashes between path segments, or in 'posix' form, with forward slashes. 
    Internally any back slashes are concerted to forward slashes. On a Windows machine file paths can be read and processed using either back or forward slashes (or indeed a mixture of the two) so the use of forward slashes internally is not a problem.</li>
    <li>The path of the folder where the program will put its outputs (text and csv files). <i>Note that this folder should be on the Postgres server.</i> If not provided, i.e. the user simply presses return, this value will default to be the same as the data folder already provided above.</li>
    <li>The path of the folder where the program will put its logs (a log is generated for each run of the program). If not provided, i.e. the user simply presses return, this value will default to be the same as the data folder already provided above.</li>
</ul>
<li>The final group of parameters relate to the source data and are optional, in the sense that they are more normally provided in the command line, but there are times (e.g. if testing) when it can be useful to include them in the configuration. They include:
<ul>
    <li>The name of the source file, as downloaded from ROR. This has a standard name pattern, with a version indicator followed by a date indicator followed by some text. If the intention is to always use the command line to indicate the source file this can be left as an empty string 
    by simply pressing return. In that case the two remaining parameters (data version and data date) are also set as empty strings and the configuration process ends. 
    N.B. Even when a source file name is provided, any source file in the command line will take precedence over and 'mask' one provided in the configuration file. The file is a .json file. If the '.json' is not present in the source file name the system will add it.</li> 
    <li>The ROR version of the file to be imported. This is a string, with a 'v' followed by a set of numbers in a semantic versioning format, e.g. 'v1.45.1', 'v1.57'. In most cases, including when retaining the name as downloaded from ROR, 
    the system can parse this from the source file name (see Operation and Commands for details). In these circumstances it can be stored in the configuration file as an empty string by simply pressing return.</li>
    <li>The date of the file to be imported, as 'data_date'. This should be in the YYYY-mm-DD ISO format. In most cases, including when retaining the name as downloaded from ROR, the system can parse this from the source file name. 
    In these circumstances it can be stored in the configuration file as an empty string by simply pressing return.</li>
</ul>
Once these 10 (or 8) questions have been answered the system creates a new configuration file and prints a copy of it to the log.<br/>
It then immediately uses the database connection parameters provided to connect to the database and create the lookup and summary tables.<br/>
The system is then ready for routine use.<br/>

Note that the configuration file can be edited at any time (using 'cargo run -- -c'). During an edit existing values becomne the default and can be re-applied by pressing return at the prompt. Thus only the values that need to be changed need to be re-typed.<br/>
If the app_config file is deleted, moved or renamed so that the system cannot find it, it will automatically switch to the initialisation mode described above.


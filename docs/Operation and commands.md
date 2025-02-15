<h3>Operations and Arguments</h3>

<h4>Configuration using Environmental varables</h4>

Once the pre-requisites are installed and the source code is downloaded, a .toml configuration file, called "imp_ror.toml", must be added to the system's source folder, i.e. in the same folder as the cargo.toml file. This .toml file should not be added to any public source control system, as it will include sensitive information such as database crdentials. It must contain values against the following settings, though in several cases sensible defaults are provided:
<ul>
<li>The database server name, as 'db_host'. This defaults to 'localhost'</li>
<li>The database user name, as 'db_user'. No default value.</li>
<li>The password for that user, as 'db_password'. No default value.</li>
<li>The database port, as 'db_port'. This defaults to '5432', the standard Postgres port.</li>
<li>The database name, as 'db_name'. This defaults to 'ror'.</li>
<li>The full path of the folder in which the souce JSON file can be found, as 'data_folder_path'.</li>
<li>The full path of the folder where logs should be written, as 'log_folder_path'. If missing the data_folder_path is used.</li>
<li>The full path of the folder where output text files should be written, as 'output_folder_path'. If missing the data_folder_path is used.</li>
</ul>

The following are normally supplied by command line arguments, which will always over-write values in the configuration file. During testing and development however, against a fixed source file, it can be easier to include them in the .env file instead.
<ul>
<li>The name of the souce JSON file, as 'src_file_name'.</li>
<li>The version of the file to be imported, as 'data_version'. A string, with a 'v' followed by a set of numbers in a semantic versioning format, e.g. 'v1.45.1', 'v1.57'.
<li>The date of the file to be imported, as 'data_date'. This should be in the YYYY-mm-DD ISO format. 
</ul>
As explained below, in practice the version and data can usually be obtained from the file name.<br/>

<h4>Command line arguments</h4>

The folowing command line arguments are available:

<i><b>-s</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -source]. Followed by a double quoted string representing the source file name, including the '.json' extension.

<i><b>-v</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -data_version]. Followed by a double quoted string representing a version number, e.g. "v1.52". In many circumstances can be derived from the source file name.

<i><b>-d</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -date]. Followed by a double quoted string in ISO YYYY-mm-DD format, representing the date of the data. In many circumstances can be derived from the source file name.

<i><b>-r</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -import]. A flag that causes import of the specified source data to ror schema tables, but not to the src schema. The source file, data version and data date must be specified.  

<b><i>Note that if the source file name follows a simple convention (described below) it is possible for the system to derive the version and date from the name. The file as named by ROR follows this convention, so in most cases, unless the file is renamed in an entirely different way, it is not necessary to specify the data'a version and date separately.</b></i>

<i><b>-p</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -process]. A flag that causes processing and summarising of the data in the ror schema tables to the src and smm schema tables. By default the system uses the version that is currently resident in the ror tables. If a version is specified and it is different from that in the ror tables the user is prompted to run -r (or -a) to first add the data to the ror tables.

<i><b>-t</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -report]. A flag that causes production of a text file summarising the main features of a version currently held within the system's summary tables. The version can be specified explicitly using the -v flag. If not specified the 'current' version is used, i.e. the last imported one, which has its data in the ror and src schema. The data of any specified version must already be in the summary data table (i.e. have had -p applied to it). The name of the output file is normally constructed from the version and the date-time of the run, but can be specified in the configuration file, e.g. during testing. 

<i><b>-a</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -all]. Equivalent to -r -p -t, i.e. run all three main processes, in that order. The source file, data version and data date must be specified, but the latter two can usually be derived from the first.

<i><b>-x</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -export]. A flag that causes production of a collection of 7 csv files, representing the data in the summary tables for the specified version. The version can be specified explicitly using the -v flag. If not specified the 'current' version is used, i.e. the last imported one, which has its data in the ror and src schema. The name of the files are constructed from the version and the date-time of the run. Note that the files are sgenerated on the Postgres server. 

<i><b>-y</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -export-all]. A flag that causes production of a collection of 7 csv files, representing <i>all</i> the data in the summary tables, for all imported versions. (v1.57 data is not exported, as it appears to be exactly the same as v1.58, just without the added geographical details of the v2.1 schema). The name of the files are constructed from the version and the date-time of the run. Note that the files are sgenerated on the Postgres server.

<b><i>Note that if any of the three 'set up' flags described below, -i, -c or -m, are used, all other flags and parameters will be ignored. The system will simply rebuild the lookup and / or summary tables.</b></i>

<i><b>-i</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -install].  Equivalent to -c -m, i.e. initialise the permanent data tables.

<i><b>-c</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -context]. A flag that causes the re-establishment of the lookup tables. Useful after any revision of those tables or the data within them.

<i><b>-m</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -summsetup]. A flag that causes the re-establishment of the summary tables in the smm schema. NOTE - ANY EXISTING DATA IN THOSE TABLES WILL BE DESTROYED. It may therefore be necessary to re-run against source files if a series of data points over time needs to be re-established.

<h4>File name convention and deriving version and data</h4>

If the file name starts with a 'v' followed by a semantic versioning string, followed by a space or a hyphen and then the date in ISO format, either with hyphens or without, then (whatever any following text in the name) the system is able to extract the data date and version from the file name. It is then no longer necessary to provide the data version and date separately. 

File names such as <b>v1.58-2024-12-11-ror-data_schema_v2.json, v1.51-20240821.json, v1.48 20240620.json</b>, and <b>v1.47 2024-05-30.json</b> all follow the required pattern. The first is the form of the name supplied by ROR, so renaming the file is not necessary (though it can help to simplify things if the '-ror-data_schema_v2.json' tail is removed).

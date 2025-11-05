<h2>Operations and Commands</h2>

<h3>Routine Operation</h3>

N.B. Using cargo to run a program with parameters requires a double hyphen before those parameters (or flags, or switches) are listed. This is to clearly differentiate the program's parameters from any that might apply to cargo itself.

In most cases the ROR data is most easily processed by running the program with an -a (= all) parameter and the -s parameter together with the source file name: <b><i>cargo run -- -a -s "&lt;source-file-name&gt;"</i></b>, <br/>
e.g. <i>cargo run -- -a -s "v1.59-2025-01-23-ror-data_schema_v2.json"</i>. <br/>
The source file name can be provided without the '.json' extension, in which case the program will add it. In addition the double quotes can be removed if the name does not contain spaces. Text after the date indicator is not read. The command:
<i>cargo run -- -a -s v1.59-2025-01-23</i> is therefore equivalent to the command above.<br/>
Alternatively the source file name can be provided within the configuration file, when <i>cargo run -- -a </i> is sufficient.

The -a command will take the data in the json file through a four stage pipeline: 
<ul>
<li>importing it into a set of 'src' schema tables, with very little change;</li>
<li>transforming it, albeit lightly, into a series of 'ppr' schema tables, and </li>
<li>summarising statistics of the data set and storing those in 'smm' schema tables.</li>
<li>generating a text file presenting the summary data from the imported version, in a series of tables.</li>
</ul>
Note that successive use of the -a command will overwrite the data in the src and ppr schema tables, with data from whatever is the most recently imported version, but that the smm schema data for each version is stored permanently.<br/>
<i>To extract data as CSV</i>
<ul>
<li>cargo run -- -x</i> will generate a set of 7 csv files with the summary data linked to the current (most recently imported) version. Specifying a different version is also possible as long as it has been previously imported and summarised.</li>
<li>cargo run -- -y</i> will generate a set of 7 csv files with the summary data from all the versions imported to that point.</li>
</ul>
In most cases the commands above are sufficient, but to maximise flexibility a range of parameters are available, as listed below.

<h3>Command line arguments</h3>

The folowing command line arguments are available:

<i><b>-s</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -source]. Followed by a string representing the source file name. This must be double quoted if it includes a space. (If it does not include the '.json' extension that will be added by the system before processing). 

<i><b>-v</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -data_version]. Followed by a double quoted string representing a version number, e.g. "v1.52". In many circumstances can be derived from the source file name (see below).

<i><b>-d</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -date]. Followed by a double quoted string in ISO YYYY-mm-DD format, representing the date of the data. In many circumstances can be derived from the source file name (see below).

<i><b>N.B. If the file name starts with a 'v' followed by a semantic versioning string, followed by a space or a hyphen and then the date in ISO format, either with hyphens or without, then (whatever any following text in the name) the system is able to extract the data version and date from the file name. It is then no longer necessary to provide the data version and date separately. File names such as v1.58-2024-12-11-ror-data_schema_v2.json, v1.51-20240821.json, v1.48 20240620.json, and v1.47 2024-05-30.json all follow the required pattern. The first is the form of the name supplied by ROR, so renaming the file is not necessary.</b></i>

<i><b>-a</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -all]. Run all three main processes for a particular ROR data version (equivalent to -r, -p, and -t together). The source file, data version and data date must be specified, but the latter two can usually be derived from the first.

<i><b>-r</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -import]. A flag that causes import of the specified source data to src schema tables, but not to the ppr schema. The source file, data version and data date must be specified. Allows the initial import process to take place in isolation. 

<i><b>-p</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -process]. A flag that causes processing and summarising of the data in the src schema tables to the ppr and smm schema tables. The system necessarily uses the version that is currently resident in the src tables.  If a version is specified and it is different from that in the src tables the user is prompted to run -r (or -a) to first add the data to the src tables.

<i><b>-t</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -text]. A flag that causes production of a text file summarising the main features of a version currently held within the system's summary tables. The version can be specified explicitly using the -v flag (it must have been summarised previously). If not specified the 'current' version is used, i.e. the last imported one, which has its data in the src and ppr schema. The name of the file is constructed from the version and the date-time of the run. 

<i><b>-x</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -export]. A flag that causes production of a collection of 7 csv files, representing the data in the summary tables for the specified version. The version can be specified explicitly using the -v flag (it must have been summarised previously). If not specified the 'current' version is used, i.e. the last imported one, which has its data in the src and ppr schema. The name of the files are constructed from the version and the date-time of the run. Note that the files are generated on the Postgres server. 

<i><b>-y</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -exportall]. A flag that causes production of a collection of 7 csv files, representing <i>all</i> the data in the summary tables, for all imported versions. (v1.57 data is not exported, as it appears to be exactly the same as v1.58, just without the added geographical details of the v2.1 schema). The name of the files are constructed from the version and the date-time of the run. Note that the files are generated on the Postgres server.

<b><i>Note that if any of the three 'set up' flags described below, -c, -k or -m, are used, all other flags and parameters will be ignored. The system will simply rebuild the configuration file and / or lookup and / or summary tables.</b></i>

<i><b>-c</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -config]. A flag that causes the configuration file to be edited (prompts for each data point are re-presented to the user).

<i><b>-k</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -lookup]. A flag that causes the lookup tables to be regenerated. Generally only used if the code or data for these tables has been revised. 

<i><b>-m</b></i>&nbsp;&nbsp;&nbsp;&nbsp;[or -summsetup]. A flag that causes the re-establishment of the summary tables in the smm schema. NOTE - ANY EXISTING DATA IN THOSE TABLES WILL BE DESTROYED. 

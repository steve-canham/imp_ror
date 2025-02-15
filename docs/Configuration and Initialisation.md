<h3>Installation and Routine use</h3>

The system is designed to be flexible, but has reasonable defaults to make routine use straightforward. 
For the moment, it does require some basic familiarity with running Rust and Postgres, and with using a development environment.

<h4>Installing the system and pre-requisites</h4>
a) Download and install Rust, and download the code from this GitHub page, accessing it within a Rust development environment. VS Code, with Rust extensions installed, is strongly recommended, and is used by about 75% of current Rust developers.<br/>
b) Install Postgres if not already available and establish an empty database (by default called 'ror', though any name cn be used). The empty database must be
created prior to the initial run of the system, but all other database operations are handled by the system.<br/>
c) Download the ror files required, from Zenodo, and place the V2 json files in the folder to be used as the source data folder.<br/>

<h4>Initialising and running the system</h4>
The system uses a configuration file to provide details of the database connection, and the folders holding data, output files amd logs. This configuration file is established of the intial running of the system.<br/>
<i>On initial run:</i>
<ul>
<li>In VS Code, type 'cargo run' and press return. No other parameters are required. (All Rust development environments use a program called <i>cargo</i> to manage code, so cargo run will simply build an .exe file from the source code and then run it.)</li>
<li>Other development environments may have </li>
Because the 
a) All Rust development environments use a program called <i>cargo</i> to manage code. To run imp_ror, use 'cargo run' followed by command line parameters, input to a terminal linked to the editor. The parameters are preceded by a double hyphen, to separate them from the cargo run command itself, e.g. cargo run -- -a.<br/>
b) <b>The initial run should be cargo run</b> This initialises the system by creating the permanent schema and tables, that hold the lookup and summary data tables.<br/>
c) Further runs are most easily done by running <b><i>cargo run -- -a -s "&lt;source-file-name&gt;"</i></b>, e.g. <i>cargo run -- -a -s "v1.59-2025-01-23-ror-data_schema_v2.json"</i>. Alternatively the source file name can be provided within the configuration file, when <i>cargo run -- -a </i> is sufficient.<br/>
d) The -a command will take the data in the json file through a four stage pipeline: 
<ul>
<li>importing it into a set of 'ror' schema tables, with very little change;</li>
<li>transforming it, albeit lightly, into a series of 'src' schema tables, and </li>
<li>summarising statistics of the data set and storing those in 'smm' schema tables.</li>
<li>generating a text file presenting the summary data from the imported version, in a series of tables.</li>
</ul>
d) Note that successive use of the -a command will overwrite the data in the ror and src schema tables, with data from whatever is the most recently imported version, but that the smm schema data for each version is stored permanently.<br/>
e) <i>cargo run -- -x</i> will generate a set of 7 csv files with the summary data linked to the current (most recently imported) version. Specifying a different version is also possible as long as it has been previously imported and summarised.<br/>
f) <i>cargo run -- -y</i> will generate a set of 7 csv files with the summary data from all the versions imported to that point.<br/>
Further details on the command line options available are in Operations and Arguments below.


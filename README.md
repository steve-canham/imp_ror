**** TO REPLACE ROR1... **** TO REPLACE ROR1... **** TO REPLACE ROR1... <br>
******* HAS SAME FUNCTIONALITY BUT SLIGHTLY SIMPLER TO USE ************ <br>
********* BUT N.B. ROR1 IS STABLE - IMP_ROR STILL EVOLVING ************ <br>

A small program to process and summarise ROR organisation data, as made available by ROR 
on Zenodo (see https://ror.readme.io/docs/data-dump). A new version of the data is posted 
on a roughly monthly basis. The program processes and retains a single version at a time, 
but retains summaries of the key features of all versions imported. Data is stored using a 
Postgres database. As outputs, the system can summarise any specific version as a text file, 
or create a set of CSV files representing the stored summary data, for a single version or 
for all those so far imported.

The system uses the version 2 schema files as input, and so covers data 
made available from April 2024 onwards. It can handle versions 2.0 and 2.1, the latter in 
use from December 2024.

The system is written in Rust and uses a command line interface (CLI) for control. 
<i>N.B. At the moment, the program is not yet available as a stand alone .exe or .lib file, 
though it is hoped to create these in the future. The current system therefore needs the 
source code to be downloaded from GitHub, and then run within a Rust development environment 
(see 'Installation and Routine use' below).</i>

<h3>Purpose</h3>

imp_ror was developed for three main reasons:

a) To provide a mechanism to quickly download and integrate ROR data within other 
systems, on a regular basis (e.g. as each new version is published). The ROR data is available
both in its 'raw' ror state, i.e. with almost no additional processing applied (see 
'The base ror data schema' below), and in a lightly modified state, with a limited degree 
of processing applied (see 'The src data schema' below). The latter might be of more immediate
use in many use cases, or a better basis for additional processing.

b) To allow comparison of the different ROR data versions over time, allowing monitoring of 
the development of ROR data, and the easier identification of possible inconsistencies or 
anomalies in the data (to help with possible feedback to ROR).

c) To become more familiar with Rust, by using the language in a small but realistic development scenario, 
implementing features that would be necessary in most similar CLIs. These include data access and 
manipulation, processing of command line arguments and configuration file variables (and interactions 
between the two), logging, file handling (of both JSON and text files), and unit and integration tests. 
The system is still 'basic' Rust, however, and does not use more advanced features of the language.

<h3>Installation and Routine use</h3>

The system is designed to be flexible, but has reasonable defaults to make routine use straightforward. 
For the moment, it does require some basic familiarity with running Rust and Postgres, and with using a development environment.

<h4>Installing the system and pre-requisites</h4>
a) Download and install Rust, and download the code from this GitHub page, accessing it within a Rust development environment - e.g. VS Code with Rust extensions installed.<br/>
b) Install Postgres if not already available and establish an empty database (by default called 'ror', though this can be changed). The database must be
created prior to the intial run of the system, but all other database operations are handled by the system.<br/>
c) Set up a configuration file with the database connection settings and a few key file parameters (see Operations and Arguments for details).<br/>
d) Download the ror files required, from Zenodo, and place the V2 json files in the folder to be used as the source data folder.<br/>

<h4>Initialising and running the system</h4>
a) All Rust development environments use a program called <i>cargo</i> to manage code. To run imp_ror, use 'cargo run' followed by command line parameters, input in a terminal linked to the editor. The parameters are preceded by a double hyphen, to separate them from the cargo run command itself, e.g. cargo run -- -a.<br/>
b) <b>The initial run should be <i>cargo run -- -i</i></b> This initialises the system by creating the permanent schema and tables, that hold the lookup and summary data tables.<br/>
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

<h3>The base ror data schema</h3>

The initial import is to tables that almost exactly mirror the original structure of the json file. 
These tables are all grouped within a schema called 'ror'. The import process recreates the tables each time.

The id used to identify each ror entry, in all tables, is the last 9 characters of the full ROR id, 
i.e. the ROR URL with the prefix "https://ror.org/" removed. This is clearer than using the full 
ROR URL, though that full string is retained as a field in the core_data table. That table also contains 
the status and year established. The other singleton data of the ror record, relating to date and schema of creation 
and last modification, are collected separately into an 'admin_data' table.

All other data potentially represents multiple entities for each organisation, Field names are, in most cases, the same 
or have an obvious correspondence to the field names as listed in the ROR documentation. The main exception to this is 
caused by the fact that 'type', whilst used in several places in the ROR definitions, is a reserved word in Rust 
(and many other languages), while 'TYPE' can cause issues in Postgres in some contexts. For safety and future 
compatibility the following changes are made:
<ul>
<li>'type' in names becomes 'name_type'.</li>
<li>'type' in types becomes 'org_type'.</li>
<li>'type' in external ids becomes 'id_type'.</li>
<li>'type' in links becomes 'link_type'.</li>
<li>'type' in relationships becomes'rel_type'.</li>
</ul>

The ror schema data also includes a tiny single-row table ('version details') that holds the version and 
date of the data within the system. This means that these parameters need only be input once. 

<h3>The src data schema</h3>

The 'ror' schema data can be processed to form a new set of tables within the 'src' schema. 
The processing is relatively limited but includes:

a) Replacement of the strings of categorised values by integers. The integers are as given by
lookup tables (set up within the 'lup' or lookup schema) which effectively provide enumerations 
of these categorised values, e.g. the organisation, name, link, external id and relationship types. 
This is intended to make any future data processing quicker and future display more flexible.

b) Ensuring all names specified as a 'ROR name' have a name type designated. In a small number of cases 
(about 30) this is not the case. They are therefore classified as labels, which allows them to 
be processed in the same way as all other ROR names.

c) The removal of duplicates from the names table. There are a small number of organisations that 
have two names with the same value - in some cases, though certainly not all, caused by the correction 
described in b). In most cases (currently about 65) these are names with two types listed in the source file, 
usually both 'label' and 'alias'. In further cases (currently about 10) the names are the same type but have two 
different language codes applied. These duplications are removed according to the folowing rules:
<ul>
<li>If one of the duplicate pairs is a ror name and the other is not, the one that is not is removed.</li> 
<li>If one is a label and the other an alias or acronym, the alias or acronym is removed.</li> 
<li>If one is an alias and the other an acronym, the alias is removed, as the names in this group all appear to be acronyms.</li> 
<li>For the remaining (very few) duplicated names, the language code least associated with the organisation's location, or if that 
is not clear that is referring to the more obscure language, is removed. This is an arbitrary decision but the choices are not 
difficult in practice.</li>
</ul>

d) The addition of script codes to the name data. Though most of the the names listed (apart 
from acronyms and company names) have language codes linked to them there is no explicit indication of 
the script being used. The great majority of the names use latin characters, but a substantial number 
use other script systems, such as Cyrilic, Greek, Arabic, Han, Hebrew or Gujarati. Details on scripts are 
provided by ISO 15924, which also provides the Unicode code pages on which each script can be found. 
Examining the Unicodes of the characters in the names allows the script to be readily identified, and this 
information is added to each name record, as being of potential value when selecting names for display.

e) The expansion of the admin_data table, to include for each organisation the numbers of entities 
of each type it is linked with, e.g. how many names (of various types), links and external ids (of 
various types), relationships (of various types), locations, etc. are included in that organisation's 
ror record. This is to make it easier both to use and display the information, to support some of the 
production of summary data, and to more easily identify organisations that are missing certain 
types of data.

f) The renaming of a few field names to make them clearer, more consistent or simpler, e.g. 
country_subdivision_code becomes csubdiv_code, lang becomes lang_code, etc.

g) For one record, the replacement of a deprecated language code with the current equivalent.
 
The src data is designed to be used as the basis for ad hoc SQL queries of the data. They are also used as 
the basis of the summary statistics described below, and are designed to provide a more useful set of base 
data when integrating ror data into other systems. Only one set of src data exists at any one time - the 
tables are recreated each time a version's data is transformed into them.

<h3>Summary data and the smm schema</h3>

The Summary (smm) schema includes a set of persistent tables that summarise various aspects of the ROR dataset. 
It includes records for all versions of the ROR data that have been imported (within each table the initial field 
is the data version, allowing easy selection of the summary data for any particular version). To make processing 
and export easier, many of the summary tables are aggregate, i.e. they hold data about different entities in the 
same table, because that data has the same structure. The tables are:

<ul>
<li>version_summary - Gives the number of organisations, and the numbers of linked entities (names, organisation types, locations, external ids, links, relationships, domains), for a specified version, equivalent to the record numbers in each of the tables in the src schemas when the version is processed. It also includes the version date, and the number of days that date represents since 29/04/2024, the earliest of the datasets in the system. This was the date of the 1.45.2 patch - in general the latest patch of any version is preferred.</li>

<li>attributes_summary - Entities in the system often have categorised attributes, e.g. the various types of name, organisation, relationship, external id and link. For each attribute category this table provides the numbers found, and the percentage this represents of the total attributes of this type, the number of organisations with this attribute type, and the percentage this represents of all organisations. For names, additional rows are given for 'nacro' or non-acronym names, i.e. labels and aliases together, and also for names (of each type) that are without a language code ('wolc').</li>

<li>count_distributions - Indicates the numbers and percentage of organisations that are linked to different counts of properties. This includes the numbers and percentages of organisations with 'n' names, labels, aliases, acronyms, organisational types, locations, external ids, links, and domains where n varies over whatever count values are found in the dataset. For instance, for organisational types n currently varies (January 2025) from 1 to 3, for names, from 1 to 28.</li>

<li>ranked_distributions - Three ranked distributions are provided: giving the usage of non English languages, the usage of non-Latin scripts, and the countries listed in locations. In each case the numbers for the 25 most common (language / script / country) values are listed, with numbers for remaining languages, scripts and countries rolled up into a 26th 'remaining' entry. The percentages each entry represents of the property of interest (non English languages, non Latin scripts and countries other than the US) and the percentage of the 'base set' (names, names and locations respectively) are also provided. </li>

<li>org_type_and_lang_code - For each combination of organisational type and name type, gives the numbers and percentages of names with and without language codes.</li>

<li>org_type_and_relationships - For each combination of organisational type and relationship type, gives the numbers and percentages (of that organisational type) which include that relationship.</li>

<li>singletons - There are a variety of measures which do not easily fit into any of the tables listed above. They are provided as a table which includes an id and a description for each data point, the number found and where relevant a percentage (both defined in the description). The singleton data points include, for instance, the numbers of labels that are designated as the ROR name, the numbers and percentages of English and non English ROR names, and the ROR names without language codes, including and excluding company names. They also include the numbers and percentage of organisations that have both parents <i>and</i> child links, i.e. are part of a hierarchy of at least 3 levels, plus the numbers of any non-reciprocated relationship records.</li>
</ul>

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

<h4>Development environment</h4>

The system was developed on a Windows 11 machine, using Rust 1.80.1, Postgres 17, VS Code and 
DBeaver. Efforts will be made to make the system cross-platform, though this has not yet been tested.

A program to process and summarise ROR organisation data, as made available by ROR 
on Zenodo (see https://ror.readme.io/docs/data-dump). A new version of the data is posted 
on a roughly monthly basis. 

<b>*** N.B. The program is written and tested on Linux (Kubuntu 24.04) thopugh was originally developed on Windows 11. It should also work, but has not yet been tested, on Macs. The current version MAY also work on Windows machines, though this is to be tested. ***</b>

The program processes and retains a single version at a time, 
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
source code to be downloaded and then run within a Rust development environment.</i>

<h2>Please use the Docs!</h2>

The summary below assumes some familiarity with Rust, Cargo and Postgres, and is only designed to provide 
a quick overview of the system.

Anyone seeking a more detailed understanding of what the system does, how to set it up and use it, including 
the various command line arguments available, and the data structures created should consult the more 
detailed documentation within the docs folder on this GitHub page. Those documents are much more detailed and 
assume less familiarity with Rust and its development environment. They cover:
<ul>
<li>Background and intended purpose</li>
<li>Installation and Configuration</li>
<li>Operation and Commands</li>
<li>Database schemas and data structures</li>
<li>Coding scripts of ROR Names</li>
</ul>


<h2>Summary of Installation and Routine use</h2>

<h4>Pre-requisites</h4>
<ul>
<li> Download the code on this GitHub page and access it within a Rust development environment. VS Code or Zed is recommended.</li>
<li> Install Postgres if not already available and establish an empty database (by default called 'ror', though any name can be used). The empty database must be
created prior to the initial run of the system, but all other database operations are handled by the system.</li>
<li> Download the ror files required, from Zenodo, and place the V2 json files in the folder to be used as the source data folder.</li>
</ul>

<h4>Initialising the system</h4>
The system uses a configuration file to provide details of the database connection, and the folders holding data, output files amd logs. This configuration file must be established during the intial running of the system.<br/>
<i>On initial run:</i>
<ul>
<li>In the development environment (e.g. in the terminal in VS Code or Zed) input <br/><b><i>'cargo run -- -i'</i></b><br/> The 'i' (initialisation) flag causes the system to create the configuration file.</li>
<li>To create the file the system will ask for various parameters: the postgress database host, user name, user password, port and database name, the folder where data source files will be found, 
and the folders to be used for outputs and logs. It will also ask for an optional source file name, though in most scenarios this can be left as an empty string (see Docs for details).</li>
<li>The values should be supplied in response to each of the prompts, after which a configuration file is constructed.</li>
<li>The system then creates lookup tables and (empty) summary tables as the second part of the initial setup.</li>
<li>The system is then ready for routine use</li>
</ul>

<h4>Routine use</h4>
<ul>
<li>In most scenarios, data can be input, processed and output by the system using a single command.</li>
<li>This involves running the system with the -a (all) flag and the -s source file flag, as in <br/><b><i>cargo run -- -a -s "&lt;source-file-name&gt;"</i></b><br/>, e.g. <i>cargo run -- -a -s "v1.59-2025-01-23-ror-data_schema_v2.json"</i>.</li>
<li>The -a command will take the data in the json file through a four stage pipeline:</li> 
<ul>
    <li>importing it into a set of 'src' schema tables, with very little change;</li>
    <li>transforming it, albeit lightly, into a series of 'ppr' schema tables, and </li>
    <li>summarising statistics of the data set and storing those in 'smm' schema tables.</li>
    <li>generating a text file presenting the summary data from the imported version, in a series of tables.</li>
    <li>more information on the ror, ppr and smm tables is provided inthe Docs.</li>
</ul>
<li>Successive use of the -a command will overwrite the data in the src and ppr schema tables, with data from whatever is the most recently imported version. The summary smm schema data for each version is, however, stored permanently.</li>
<li><i>cargo run -- -x</i> will generate a set of 7 csv files with the summary data linked to the current (most recently imported) version. Specifying a different version is also possible as long as it has been previously imported and summarised.</li>
<li><i>cargo run -- -y</i> will generate a set of 7 csv files with the summary data from all the versions imported to that point.</li>
</ul>

<h4>Command line arguments</h4>

For the full list of command line arguments please see the doc '3_Operation and Commands.md':

<h2>Version Information</h2>

<h4>Version 1.0</h4>
21/02/2025  -  All functionality described above present

<h4>Version 1.1</h4>
24/04/2025  -  Changes:
<ul>
<li>Script coding process improved to become more comprehensive and accurate</li>
<li>Improved script coding process fully documented</li>
<li>Location data in core table improved to handle organisations with multiple locations more accurately</li>
<li>Location data - number of orgs with multiple locations, states, countries - added to summary report</li>
<li>Fixed bug where orgs with completely duplicate names (same name, language, type) are better handled (only one organisation affected)</li>
</ul>

<h4>Version 1.2</h4>
05/11/2025  -  Changes:
<ul>
<li>Code to transform ppr data into more mdr compatible data, whilst initially developed here, moved out of the project (and into the mk_org project)</li>
<li>Schema used to store data post-processing renamed from 'src' to 'ppr'</li>
<li>Initial schema used for import of ror data renamed from 'ror' to 'src'</li>
<li>Improved (and simplified) handling of duplicate names in source ror data</li>
<li>Added a few data points relating to duplicate name processing to the summary tables / reports</li>
</ul>

<h4>Version 1.3</h4>
25/01/2026  -  Changes:
<ul>
<li>Export csv code process changed to use Rust code directly (using the 'csv' crate) rather than delegating to the Postgres 'Copy' command. <br/>
This was because on Linux permissions to write files are more restricted, (compared to Windows) and setting the required permissions for the Postgres account is difficult.<br/>
Using the user's own permissions, running the Rust executable through cargo run, is easier and more transparent to manage.</li>
</ul>

<h4>Version 1.4</h4>
24/06/2026  -  Changes:
<ul>
<li>All code reviewed, with substantial refactoring and simplification in many source files.</li>
<li>Code changed to be more idiomatic Rust where possible, e.g. greater use of guarded match arms</li>
<li>Creating and editing configuration file separated from setup and code substantially improved.</li>
<li>Location of the configuration file made OS dependent, using conventional choices for application configuration data.</li>
<li>SQL code largely separated from other code but retained within a Rust framework (to make supporting multiple platforms easier).</li>
<li>Setup code simplified and setup flags simplified.</li>
</ul>

<h2>Background</h2>

<h3>Summary</h3>

imp_ror is a small program to process and summarise ROR organisation data, as made available by ROR 
on Zenodo (see https://ror.readme.io/docs/data-dump). A new version of the data is posted 
on a roughly monthly basis. The program processes and retains a single version at a time, 
but retains summaries of the key features of all versions imported. Data is stored using a 
Postgres database. 

As outputs, the system can summarise any specific version as a text file, 
or create a set of CSV files representing the stored summary data, for a single version or 
for all those so far imported.

The system uses the version 2 schema files as input, and so covers data 
made available from April 2024 onwards. It can handle schema versions 2.0 and 2.1, the latter in 
use from December 2024.

The system is written in Rust and uses a command line interface (CLI) for control. 

<i>N.B. At the moment, the program is not yet available as a stand alone .exe or .lib file, 
though it is hoped to create these in the future. The current system therefore requires the 
source code to be downloaded and then run within a Rust development environment.</i>

<h3>Purpose</h3>

imp_ror was developed for three main reasons:

a) To provide a mechanism to quickly download and integrate ROR data within other 
systems, on a regular basis (e.g. as each new version is published). The ROR data is available
both in its 'raw' state, i.e. with almost no additional processing applied (see 
'The base src data schema' in doc.4), and in a lightly modified state, with a limited degree 
of processing applied (see 'The ppr data schema' in doc 4. below). The latter is likely to be of more immediate
use in many use cases, as well as a better basis for additional processing.

b) To allow comparison of the different ROR data versions over time, allowing monitoring of 
the development of ROR data, and the easier identification of possible inconsistencies or 
anomalies in the data (to help with possible feedback to ROR).

c) To become more familiar with Rust, by using the language in a small but realistic development scenario, 
implementing features that would be necessary in many similar CLIs. These include data access and 
manipulation, processing of command line arguments and configuration of file variables (and interactions 
between the two), logging, file handling (of both JSON and text files), and unit and integration tests. 
The system is still 'basic' Rust, however, and does not use more advanced features of the language.

<h3>Development environment</h3>
 
The system was originally developed on a Windows 11 machine, using Rust 1.80.1, Postgres 17, VS Code and 
DBeaver. 
It is now being developed / maintained on a Linux machine (Kubuntu 24.04), using Rust 1.91, Postgres 18 and pgAdmin.
Efforts will be made to make the system cross-platform, though this has not yet been tested.
<br/><br/>
N.B. If running tests against the code, using cargo test, use the command<br/>
<b>cargo test -- --test-threads=1</b><br/>
This forces all tests onto a single thread and allows the integration tests to occur in the correct (alphabetical) order. <br/>
A standard 'cargo test' command will allow unit test to pass, but the integration tests will fail, complaining that they cannot obtain database parameters.<br/>
(By default, cargo will attempt to run all tests in parallel).




<h2>Database Schemas</h2>

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

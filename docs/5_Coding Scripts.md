
The following steps are required to identify and include the script codes for ROR names.  
***Note that a table with all the necessary script information, lup.lang_scripts, is automatically constructed by the imp_ror system when it is initialised. Steps 1 and 2 are therefore not required if coding scripts using imp_ror.***

### 1. Establish a table of scripts and their parameters.

Ensure that a table exists of the scripts within the Unicode / ISO 15924 system. This table already exists within imp_ror but if necessary can be constructed from data available online. For each script the table should include

- Its code. These are 4 characters long within ISO 15924, though as described below longer codes are added for a few additional scripts.
- Its name in unicode.
- Its name in ISO 15924 - usually but not always the same as its unicode name.
- (Optionally) A directionality indicator, normally ‘LtR’ or ‘RtL’. For some East Asian scripts a vertical directionality may be added, e.g. ‘LtR, BtT’, ‘VRtL, LtR’.
- (Optionally) The number of characters in the script.
- (Optionally) Notes. Optional but sometimes a line to explain usage is useful.
- Number of first character within Unicode, in hex.
- Number of last character within Unicode, in hex.
- Number value of first character within Unicode, in decimal.
- Number of last character within Unicode, in decimal.
- Source system.  ‘ISO 15924’ for most scripts. A few added manually have a different source.


Note that Japanese is really a writing system rather than a script, as it can use three different scripts (Han - or Kanji, Hiragana, and Katakana) and often mixes these within the same sentence, or in ROR within the same name. Nevertheless it is given its own ISO 15924 entry, code ‘Jpan’.

### 2. Ensure additional non-ISO scripts are included.

As well as the 67 scripts listed within ISO 15924, it is also necessary to add:

a) A ‘Latin Extended’ script (code ‘Latn2’) that covers Unicode characters from 1E00 to 1EFF.  
These are mostly Latin letters with unusual or two different accent marks (one above and one below the letter). In the ROR data they occur mostly in romanised versions of Vietnamese names.  
Including them ensures that they are detected by the system. The source here is given as ‘web’, as the data is available from a variety of Unicode related resources.

b) 9 different ‘combination’ scripts, used within the imp_ror system to designate a name that includes two scripts – usually Latin and another (Greek, Cyrillic, Hani etc.).  
Latin and Japanese is the most common combination, but even that only applies to about 90 names – the exact number depends on how ‘using two scripts’ is defined.  
Codes are those of the source scripts separated by a comma, e.g. ‘Latn, Grek’, or ‘Latn, Hang’.

### 3. Create a copy of the name data.

To reduce side effects from any errors in the process, and make it easier to manage and develop, it is better to copy the ROR name data into a temporary table.  
This becomes a names ‘scratch pad’ and will be discarded after the scripting process ends. In the imp_ror system this table is called ppr.names_pad.  

The table includes two fields for the name – one that is the original value and the second for the name after it has been pre-processed.  
It also has two additional fields, latin and nonlatin, which will be used later to process mixed script names. Otherwise the name data is simply copied across from the source ‘names’ table, that contains all the names in the ROR system. The script code field must be initialised as an empty string rather than null. The SQL used to create and fill this table is:
```
      create table ppr.names_pad
      (
            id                varchar     not null
          , original_name     varchar     not null
          , name              varchar     null
          , lang_code         varchar     null
          , script_code       varchar     null
          , latin             varchar     null
          , nonlatin          varchar     null
      );
      create index names_pad_idx on ppr.names_pad(id);
  
      Insert into ppr.names_pad (id, original_name, name, lang_code, script_code)
      select id, value, value, lang_code, ''
      from ppr.names;
```

### 4. Pre-process the name data.
A variety of punctuation marks that are traditionally included in the ‘Latin’ block also occur within non-Latin names. They need to be removed or the script will report, wrongly, a huge number of mixed script names. 
In addition, one character that occurs in Katakana (the katakana middle dot) also needs to be removed, as this turns up in a few latin text Japanese names. The characters are listed below:

- Full stop (period), comma, semi-colon, colon,
- Left and right parentheses, left and right square brackets,
- Apostrophes, double straight quotes and guillemets («, »),
- Hyphens, ampersands, slashes and pipes (|),
- Middle dot (like a small bullet point)
- Underscores
- All spaces

Character removal is done using the SQL replace function, targeted to the appropriate records, the SQL being constructed in Rust by interpolating the character to be removed (char) as below
```
    let sql  = format!(r#"update ppr.names_pad
        set name = replace(name, '{}', '')
        where name like '%{}%'; "#, char, char);
```
A slightly different formulation is used if the unicode representation of a character is used, rather than the character itself.   
This can be useful for characters not present on a normal keyboard, e.g. the guillemets can be removed using their unicodes (00AB and 00BB).
```
    let sql  = format!(r#"update ppr.names_pad
        set name = replace(name, U&'\{}', ''
        where name like U&'%\{}%'; "#, unicode, unicode);
```
There are many other punctuation marks of course. Some do not appear in ROR names, whilst others (e.g. n- and m- dashes, single left and right quotes), occur but are not within the latin Unicode block. Instead they are in the ‘general punctuation’ block above hex 2000 and do not figure in the table of scripts. They are therefore ignored by the coding process.

Some of this pre-processing could be avoided if the ‘Latin block’ was defined more tightly in the scripts table, e.g. starting at hex 30 (decimal 48) rather than 0. This is risky, however, because if the table was reconstructed for any reason this change would need to be remembered and repeated. As the removal process only takes a few seconds it is safer to use the standard definition of the latin block and remove the problematic characters afterwards.

### 5. Identify and Apply the Script Codes.

The unicodes within the names can now be examined to determine the scripts being used. First the codes and start and end positions of each script, in both hex and decimal, are loaded from the database to form a collection (in Rust a Vector) of script parameters. The system runs through each script in term.

For scripts with boundaries, in hex, that are 4 characters long, or less, whether or not a name contains characters of that script can be tested using regular expressions. The formulation 
```
       let sql  = format!(r#"update ppr.names_pad
           set script_code = script_code||', '||'{}'
           where name ~ '[\u{:0>4}-\u{:0>4}]'"#, 
           r.code, r.hex_start, r.hex_end);
```
interpolates the code and start and end hex positions obtained from the current script (‘r’) to obtain a sql statement like:
```
	    update ppr.names_pad
          set script_code = script_code||', '||'Cyrl'
          where name ~ '[\u0400-\u04FF]'
```
If the regular expression test succeeds, i.e. if the name contains any character in the specified hex range, the current code is added, after a separating comma, to the existing script code. This is why the script code value must start as an empty string – if it was null the concatenation would also result in a null.

As the process works through each of the scripts, a name associated with more than one code will have all codes added, though this only applies to a very small proportion of names.

For scripts with hex boundaries of 5 characters the approach above results in an error – a limitation of the Postgres regular expression syntax. There are relatively few such scripts (12 in total) and they are all extremely obscure.  In these cases the system simply checks the ascii value of the initial letter of the name, and checks that against the ascii (i.e. decimal) script limits:
```
     let sql  = format!(r#"update ppr.names_pad
         set script_code = script_code||', '||'{}' 
         where ascii(substr(name, 1, 1)) >= {}
         and  ascii(substr(name, 1, 1)) <= {}"#, 
         r.code, r.ascii_start, r.ascii_end);
```
In fact, no names using these scripts appear to be present.

At the end of the process all names are associated with at least one script – the vast majority just one. The spurious initial ‘, ’ is then removed from all script codes. 

### 6. Post-processing of Script codes.

Some additional steps are needed to improve the accuracy of the coding.  

a) ‘Latn, Latn2’ coding:  
A ‘Latn, Latn2’ coding can be simplified to just ‘Latn’ without any loss of information. In fact Latn2 characters only seem to occur within ROR names in conjunction with ‘normal’ latin characters, there are no names only using Latin2 (though no guarantee that will not happen in the future).

b) Clean Japanese coding:  
Using ‘Jpan’ as the script coding for all non Latin names from Japan, even when they only consist of a single script, helps to distinguish such names from similar (Han) names used in China, Taiwan, Hong Kong and, occasionally, Korea. More importantly, it allows a use case where all non Latin names relevant to a Japanese ROR user can be filtered and presented, or filtered out, without the complexities of distinguishing different national uses of Han.

Where scripts used are exclusive to Japan (i.e. containing one of Katakana or Hiragara) the script code can therefore be changed to simply ‘Jpan’.  If the code indicates Han characters then it can only safely be classified as ‘Jpan’ if the country of the organisation is also Japan. The system therefore uses location data (in fact the country code stored in the core data table, but that is derived from location data) to identify the Han script from Japan (Kanji) and recode that as ‘Jpan’.

Similarly, names with Latin and one or both of Katakana or Hiragara can be recoded as ‘Latn, Jpan’. Names containing Latin and Han can be recoded as ‘Latn, Jpan’ only if the organisation is based in Japan.

### 7. Dealing with Double Script names.
Once the extended Latin and Japanese codes have been simplified there remains, currently, just over 300 names with – ostensibly – characters from 2 different scripts. In all but 2 cases this is a combination of Latin and one other script. 

It is useful to separate the two components, the Latin and non Latin, storing them in the fields of the same name, to differentiate the various ways in which this double coding occurs, recoding in some cases.

Regular expressions are again used to locate and extract the Latin and non-Latin portions of the name.  
In the SQL below the inner select statement extracts the latin portion(s) of a double coded name using the REGEXP_MATCHES function. Each portion is a separate record, so for each name they are aggregated together as ‘combined_array’, by the outer select statement. The resultant set of records can then be used to update the latin field of names_pad table.
```
    update ppr.names_pad n
    set latin = combined_array
    from
        (SELECT id, name,
              array_to_string(array_agg(latin), '') AS combined_array
        FROM 
              (select id, name,
              REGEXP_MATCHES(name,'[\u0000-\u02FF]+', 'g') as latin
              from ppr.names_pad
              where length(script_code) > 4
              and script_code like '%Latn%') as t
        GROUP BY id, name ) m
    where n.id = m.id
    and n.name = m.name
```
The corresponding SQL is given below for the non Latin portions of the name.
```
    update ppr.names_pad n
    set nonlatin = combined_array
    from
        (SELECT id, name, 
              array_to_string(array_agg(nonlatin), '') AS combined_array
         FROM 
                (select id, name, 
                REGEXP_MATCHES(name,'[\u0300-\uD800]+', 'g') as nonlatin
                from ppr.names_pad
                where length(script_code) > 4
                and script_code like '%Latn%') as t
         GROUP BY id, name ) m
    where n.id = m.id
    and n.name = m.name
```
It is clear that in many cases the ‘double coding’ is nominal, with the only latin component often being a single letter or a few numbers. The following rules are applied to make the coding more accurate and to reduce the ‘double script’ names to a more realistic set:

Greek and Cyrillic scripts have, so far as I know, the same numerals as Latin script. If the Latin component of a ‘Latn, Grek’ or ‘Latn, Cyrl’ coded name is simply numerals then they are really just Greek or Cyrillic respectively and not double coded at all. The system makes this change by again using regular expressions to identify names with Latin portions that are all numeric, e.g:
```
			update ppr.names_pad
      set script_code = 'Cyrl'
      where script_code = 'Latn, Cyrl'
      and latin ~ '^\d*$'
```
Even though Latin scripts use ‘Arabic’ numbers, Arabic scripts have their own number system using what in Arabic are called ‘Hindi’ numbers, a reference to the origin of position based number systems. But from reading on the web it is clear that Arabic text, and by implication Arabic speaking users, can sometimes include Latin numbers. Given that the numbers present in Arabic script names are usually very simple (e.g  جامعة وهران 2 محمد بن أحمد), it does not seem unreasonable to make the same change as for Greek and Cyrillic names – if the only Latin component is numeric then the name is recoded as just ‘Arab’.

The same would also be true of most Indian scripts, e.g. using the Devanagari script of Hindi, as English is taught and widely used throughout India, but there do not appear to be any ‘Latn, Deva’ names with only numerals in the Latin portion. It is not clear to me if the same applies to East Asian scripts - the assumption has been that it does not.

In many cases there is a large discrepancy between the size of the Latin and nonLatin portions of these names. Often the ‘minority’ script consists of just one or two characters, sometimes begging the question of whether the inclusion of the minority script is by accident or design.

From a usage point of view, a ‘Latin user’ is unlikely to understand a name retrieved as being (partially) Latin if that Latin component is just a character or two – in normal usage it can only be interpreted by someone familiar with the majority script. Conversely a ‘non Latin user’ will probably not make sense of a name classified as a combination of that script and Latin if almost all of the name is in Latin (unless of course they are also familiar with a Latin language like English).

It therefore makes sense to reclassify the double coded names as the majority script, when 

- the minority script consists of only 1 or 2 characters, AND
- the majority script is at least 6 characters long.

The second criterion is to avoid recoding some double scripted names where both components are very short (e.g. ‘LG화학’, or ‘智谱ai’) – possibly double scripted acronyms. These limits are arbitrary, and could be tweaked to other values, but they seem a reasonable starting point. The SQL for recoding the ‘short Latin – longer nonLatin’ scenario is given below. Similar code is used for the converse recoding.
```
          update ppr.names_pad
          set script_code = substring(script_code, 7)
          where length(script_code) > 4
          and length(latin) < 3 
          and char_length(nonlatin) > 5
```
Making the three changes above means that almost half of the ‘double scripted’ names disappear. There are (currently, April 2025) just 158 names that could be classified as using two scripts, and even that is probably stretching the definition. (The resultant set is shown in the tab double_coded_2 in the current summary spreadsheet)

Why these double coded names occur is unclear. Presumably they have appeared at some time in source material listing an organisation, and are therefore needed as part of a matching process. Many of the Japanese entries, for example, just include a Latin acronym followed by ‘株式会社’, according to Google translate the equivalent of ‘co. ltd.’ or ‘corporation’. Some may be simple errors, though it not clear how easily that could be checked and corrections made.

Only a very few seem like genuine and deliberate attempts to provide a full name in two scripts:  
‘武田薬品工業株式会社, Takeda Yakuhin Kōgyō kabushiki gaisha’ and   
‘Институт коммерции и праваWebsiteDirections’  
are two possible examples.

### 8. Applying the Codes.
At the end of the process the main ppr.names table can be updated with the codes as calculated within the ppr.name_pads table and the name_pads table can be dropped.

### 9. Additional Language Codes.
As a bonus, the script codes can help in reducing the numbers of names without language codes.  
The assumption here is that if a name is in a non Latin script and that script is used for the country’s main language, then that name is almost certainly in that language, or if an acronym was derived from words in that language.  
This allows a few hundred names to have a language code allocated automatically, and this is the final stage of the script coding process. The SQL is generated within a generic routine, the language code and country code being inserted into the template each time.
```
	   let sql  = format!(r#"update ppr.names n
          set lang_code = '{}'
          from ppr.core_data c
          where n.id = c.id
          and n.lang_code is null
          and n.script_code <> 'Latn'
          and c.country_code = '{}' ;"#, lang_code, country_code);
```
Language and country code pairs include ‘uk’ and ‘UA’ for Ukraine, ‘el’ and ‘GR’ for Greece, ‘zh’ and ‘CN’ for China, and ‘bg’ and ‘BG’ for Bulgaria. The system currently applies 13 of those pairs to the names data.

   
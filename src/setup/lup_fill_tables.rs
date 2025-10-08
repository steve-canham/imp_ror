use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::AppError;

pub async fn fill_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {

    execute_sql(fill_lookup_tables_sql(), pool).await?;
    execute_sql(fill_countries_1_sql(), pool).await?;
    execute_sql(fill_countries_2_sql(), pool).await?;
    execute_sql(fill_countries_3_sql(), pool).await?;
    execute_sql(fill_lang_codes_1_sql(), pool).await?;
    execute_sql(fill_lang_codes_2_sql(), pool).await?;
    execute_sql(fill_lang_codes_3_sql(), pool).await?;
    execute_sql(fill_script_codes_1_sql(), pool).await?;
    execute_sql(fill_script_codes_2_sql(), pool).await?;
    execute_sql(fill_script_codes_3_sql(), pool).await?;

    Ok(())
}

async fn execute_sql(sql: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, AppError> {

    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))
}


fn fill_lookup_tables_sql <'a>() -> &'a str {
    r#"insert into lup.ror_status_types(id, name) 
    values (1, 'active'), (2, 'inactive'), (3, 'withdrawn');

    insert into lup.ror_org_types(id, name) 
       values (100, 'government'), (200, 'education'), (300, 'healthcare'), 
       (400, 'company'), (500, 'nonprofit'), (600, 'funder'),
       (700, 'facility'), (800, 'archive'),  (900, 'other');
    
    insert into lup.ror_name_types(id, name) 
        values (5, 'label'), (7, 'alias'), (10, 'acronym');
    
    insert into lup.ror_id_types(id, name) 
       values (11, 'isni'), (12, 'wikidata'),
       (13, 'grid'), (14, 'fundref');
    
    insert into lup.ror_link_types(id, name) 
      values (21, 'wikipedia'), (22, 'website');
    
    insert into lup.ror_rel_types(id, name) 
       values (1, 'has parent'), (2, 'has child'), (3, 'is related to'),
        (4, 'has predecessor'), (5, 'has successor');"#
}

fn fill_countries_1_sql <'a>() -> &'a str {
    r#"insert into lup.countries(code, name) values
    ('AD', 'Andorra'), ('AE', 'United Arab Emirates'), ('AF', 'Afghanistan'), ('AG', 'Antigua and Barbuda'), ('AI', 'Anguilla'), 
    ('AL', 'Albania'), ('AM', 'Armenia'), ('AN', 'Netherlands Antilles'), ('AO', 'Angola'), ('AQ', 'Antarctica'), 
    ('AR', 'Argentina'), ('AS', 'American Samoa'), ('AT', 'Austria'), ('AU', 'Australia'), ('AW', 'Aruba'), ('AX', 'Aland Islands'), 
    ('AZ', 'Azerbaijan'), ('BA', 'Bosnia and Herzegovina'), ('BB', 'Barbados'), ('BD', 'Bangladesh'), ('BE', 'Belgium'), 
    ('BF', 'Burkina Faso'), ('BG', 'Bulgaria'), ('BH', 'Bahrain'), ('BI', 'Burundi'), ('BJ', 'Benin'), 
    ('BL', 'Saint Barthelemy'), ('BM', 'Bermuda'), ('BN', 'Brunei'), ('BO', 'Bolivia'), ('BQ', 'Bonaire, Saint Eustatius and Saba '), 
    ('BR', 'Brazil'), ('BS', 'Bahamas'), ('BT', 'Bhutan'), ('BV', 'Bouvet Island'), ('BW', 'Botswana'), 
    ('BY', 'Belarus'), ('BZ', 'Belize'), ('CA', 'Canada'), ('CC', 'Cocos Islands'), ('CD', 'Democratic Republic of the Congo'), 
    ('CF', 'Central African Republic'), ('CG', 'Republic of the Congo'), ('CH', 'Switzerland'), ('CI', 'Ivory Coast'), ('CK', 'Cook Islands'), 
    ('CL', 'Chile'), ('CM', 'Cameroon'), ('CN', 'China'), ('CO', 'Colombia'), ('CR', 'Costa Rica'), 
    ('CS', 'Serbia and Montenegro'), ('CU', 'Cuba'), ('CV', 'Cabo Verde'), ('CW', 'Curacao'), ('CX', 'Christmas Island'), 
    ('CY', 'Cyprus'), ('CZ', 'Czechia'), ('DE', 'Germany'), ('DJ', 'Djibouti'), ('DK', 'Denmark'), 
    ('DM', 'Dominica'), ('DO', 'Dominican Republic'), ('DZ', 'Algeria'), ('EC', 'Ecuador'), ('EE', 'Estonia'), 
    ('EG', 'Egypt'), ('EH', 'Western Sahara'), ('ER', 'Eritrea'), ('ES', 'Spain'), ('ET', 'Ethiopia'), 
    ('FI', 'Finland'), ('FJ', 'Fiji'), ('FK', 'Falkland Islands'), ('FM', 'Micronesia'), ('FO', 'Faroe Islands');"#
}

 fn fill_countries_2_sql <'a>() -> &'a str {
    r#"insert into lup.countries(code, name) values
    ('FR', 'France'), ('GA', 'Gabon'), ('GB', 'United Kingdom'), ('GD', 'Grenada'), ('GE', 'Georgia'), 
    ('GF', 'French Guiana'), ('GG', 'Guernsey'), ('GH', 'Ghana'), ('GI', 'Gibraltar'), ('GL', 'Greenland'), 
    ('GM', 'Gambia'), ('GN', 'Guinea'), ('GP', 'Guadeloupe'), ('GQ', 'Equatorial Guinea'), ('GR', 'Greece'), 
    ('GS', 'South Georgia and the South Sandwich Islands'), ('GT', 'Guatemala'), ('GU', 'Guam'), ('GW', 'Guinea-Bissau'), ('GY', 'Guyana'), 
    ('HK', 'Hong Kong'), ('HM', 'Heard Island and McDonald Islands'), ('HN', 'Honduras'), ('HR', 'Croatia'), ('HT', 'Haiti'), 
    ('HU', 'Hungary'), ('ID', 'Indonesia'), ('IE', 'Ireland'), ('IL', 'Israel'), ('IM', 'Isle of Man'), 
    ('IN', 'India'), ('IO', 'British Indian Ocean Territory'), ('IQ', 'Iraq'), ('IR', 'Iran'), ('IS', 'Iceland'), 
    ('IT', 'Italy'), ('JE', 'Jersey'), ('JM', 'Jamaica'), ('JO', 'Jordan'), ('JP', 'Japan'), 
    ('KE', 'Kenya'), ('KG', 'Kyrgyzstan'), ('KH', 'Cambodia'), ('KI', 'Kiribati'), ('KM', 'Comoros'), 
    ('KN', 'Saint Kitts and Nevis'), ('KP', 'North Korea'), ('KR', 'South Korea'), ('KW', 'Kuwait'), ('KY', 'Cayman Islands'), 
    ('KZ', 'Kazakhstan'), ('LA', 'Laos'), ('LB', 'Lebanon'), ('LC', 'Saint Lucia'), ('LI', 'Liechtenstein'), 
    ('LK', 'Sri Lanka'), ('LR', 'Liberia'), ('LS', 'Lesotho'), ('LT', 'Lithuania'), ('LU', 'Luxembourg'), 
    ('LV', 'Latvia'), ('LY', 'Libya'), ('MA', 'Morocco'), ('MC', 'Monaco'), ('MD', 'Moldova'), 
    ('ME', 'Montenegro'), ('MF', 'Saint Martin'), ('MG', 'Madagascar'), ('MH', 'Marshall Islands'), ('MK', 'North Macedonia'), 
    ('ML', 'Mali'), ('MM', 'Myanmar'), ('MN', 'Mongolia'), ('MO', 'Macao'), ('MP', 'Northern Mariana Islands'), 
    ('MQ', 'Martinique'), ('MR', 'Mauritania'), ('MS', 'Montserrat'), ('MT', 'Malta'), ('MU', 'Mauritius');"#
}

fn fill_countries_3_sql <'a>() -> &'a str {
    r#"insert into lup.countries(code, name) values
    ('MV', 'Maldives'), ('MW', 'Malawi'), ('MX', 'Mexico'), ('MY', 'Malaysia'), ('MZ', 'Mozambique'), ('NA', 'Namibia'), 
    ('NC', 'New Caledonia'), ('NE', 'Niger'), ('NF', 'Norfolk Island'), ('NG', 'Nigeria'), ('NI', 'Nicaragua'), 
    ('NL', 'Netherlands'), ('NO', 'Norway'), ('NP', 'Nepal'), ('NR', 'Nauru'), ('NU', 'Niue'), 
    ('NZ', 'New Zealand'), ('OM', 'Oman'), ('PA', 'Panama'), ('PE', 'Peru'), ('PF', 'French Polynesia'), 
    ('PG', 'Papua New Guinea'), ('PH', 'Philippines'), ('PK', 'Pakistan'), ('PL', 'Poland'), ('PM', 'Saint Pierre and Miquelon'), 
    ('PN', 'Pitcairn'), ('PR', 'Puerto Rico'), ('PS', 'Palestinian Territory'), ('PT', 'Portugal'), ('PW', 'Palau'), 
    ('PY', 'Paraguay'), ('QA', 'Qatar'), ('RE', 'Reunion'), ('RO', 'Romania'), ('RS', 'Serbia'), 
    ('RU', 'Russia'), ('RW', 'Rwanda'), ('SA', 'Saudi Arabia'), ('SB', 'Solomon Islands'), ('SC', 'Seychelles'), 
    ('SD', 'Sudan'), ('SE', 'Sweden'), ('SG', 'Singapore'), ('SH', 'Saint Helena'), ('SI', 'Slovenia'), 
    ('SJ', 'Svalbard and Jan Mayen'), ('SK', 'Slovakia'), ('SL', 'Sierra Leone'), ('SM', 'San Marino'), ('SN', 'Senegal'), 
    ('SO', 'Somalia'), ('SR', 'Suriname'), ('SS', 'South Sudan'), ('ST', 'Sao Tome and Principe'), ('SV', 'El Salvador'), 
    ('SX', 'Sint Maarten'), ('SY', 'Syria'), ('SZ', 'Eswatini'), ('TC', 'Turks and Caicos Islands'), ('TD', 'Chad'), 
    ('TF', 'French Southern Territories'), ('TG', 'Togo'), ('TH', 'Thailand'), ('TJ', 'Tajikistan'), ('TK', 'Tokelau'), 
    ('TL', 'Timor Leste'), ('TM', 'Turkmenistan'), ('TN', 'Tunisia'), ('TO', 'Tonga'), ('TR', 'Turkey'), 
    ('TT', 'Trinidad and Tobago'), ('TV', 'Tuvalu'), ('TW', 'Taiwan'), ('TZ', 'Tanzania'), ('UA', 'Ukraine'), 
    ('UG', 'Uganda'), ('UM', 'United States Minor Outlying Islands'), ('US', 'United States'), ('UY', 'Uruguay'), ('UZ', 'Uzbekistan'), 
    ('VA', 'Vatican'), ('VC', 'Saint Vincent and the Grenadines'), ('VE', 'Venezuela'), ('VG', 'British Virgin Islands'), ('VI', 'U.S. Virgin Islands'), 
    ('VN', 'Vietnam'), ('VU', 'Vanuatu'), ('WF', 'Wallis and Futuna'), ('WS', 'Samoa'), ('XK', 'Kosovo'), ('YE', 'Yemen'), 
    ('YT', 'Mayotte'), ('ZA', 'South Africa'), ('ZM', 'Zambia'), ('ZW', 'Zimbabwe');"#
}

fn fill_lang_codes_1_sql <'a>() -> &'a str {
    r#"insert into lup.lang_codes(code, marc_code, name, source) values
    ('af', 'afr', 'Afrikaans', 'ISO 639-1'), ('am', 'amh', 'Amharic', 'ISO 639-1'), ('ar', 'ara', 'Arabic', 'ISO 639-1'),
    ('az', 'aze', 'Azerbaijani', 'ISO 639-1'), ('be', 'bel', 'Belarusian', 'ISO 639-1'), ('bg', 'bul', 'Bulgarian', 'ISO 639-1'),
    ('bn', 'ben', 'Bengali', 'ISO 639-1'), ('bo', 'tib', 'Tibetan', 'ISO 639-1'), ('br', 'bre', 'Breton', 'ISO 639-1'),
    ('bs', 'bos', 'Bosnian', 'ISO 639-1'), ('ca', 'cat', 'Catalan', 'ISO 639-1'), ('ce', 'che', 'Chechen', 'ISO 639-1'),
    ('co', 'cos', 'Corsican', 'ISO 639-1'), ('cs', 'cze', 'Czech', 'ISO 639-1'), ('cy', 'wel', 'Welsh', 'ISO 639-1'),
    ('da', 'dan', 'Danish', 'ISO 639-1'), ('de', 'ger', 'German', 'ISO 639-1'), ('el', 'gre', 'Greek', 'ISO 639-1'),
    ('en', 'eng', 'English', 'ISO 639-1'), ('eo', 'epo', 'Esperanto', 'PubMed'), ('es', 'spa', 'Spanish', 'ISO 639-1'),
    ('et', 'est', 'Estonian', 'ISO 639-1'), ('eu', 'baq', 'Basque', 'ISO 639-1'), ('fa', 'per', 'Persian', 'ISO 639-1'),
    ('fi', 'fin', 'Finnish', 'ISO 639-1'), ('fr', 'fre', 'French', 'ISO 639-1'), ('ga', 'gle', 'Irish Gaelic', 'ISO 639-1'),
    ('gd', 'gla', 'Scottish Gaelic', 'ISO 639-1'), ('gl', 'glg', 'Galician', 'ISO 639-1'), ('gu', 'guj', 'Gujarati', 'ISO 639-1'),
    ('ha', 'hau', 'Hausa', 'ISO 639-1'), ('he', 'heb', 'Hebrew', 'ISO 639-1'), ('hi', 'hin', 'Hindi', 'ISO 639-1'),
    ('hr', 'hrv', 'Croatian', 'ISO 639-1'), ('hu', 'hun', 'Hungarian', 'ISO 639-1'), ('hy', 'arm', 'Armenian', 'ISO 639-1'),
    ('id', 'ind', 'Indonesian', 'ISO 639-1'), ('is', 'ice', 'Icelandic', 'ISO 639-1'), ('it', 'ita', 'Italian', 'ISO 639-1'),
    ('iu', 'iku', 'Inuktitut', 'ISO 639-1'), ('ja', 'jpn', 'Japanese', 'ISO 639-1'), ('jv', 'jav', 'Javanese', 'ISO 639-1'),
    ('ka', 'geo', 'Georgian', 'ISO 639-1'), ('kk', 'kaz', 'Kazakh', 'ISO 639-1'), ('kl', 'kal', 'Greenlandic, Kalaallisut', 'ISO 639-1'),
    ('km', 'khm', 'Central Khmer', 'ISO 639-1'), ('kn', 'kan', 'Kannada', 'ISO 639-1'), ('ko', 'kor', 'Korean', 'ISO 639-1'), 
    ('ks', 'kas', 'Kashmiri', 'ISO 639-1'), ('ku', 'kur', 'Kurdish', 'ISO 639-1'), ('la', 'lat', 'Latin', 'ECRIN'),
    ('lb', 'ltz', 'Luxembourgish', 'ISO 639-1'), ('lo', 'lao', 'Lao', 'ISO 639-1'), ('lt', 'lit', 'Lithuanian', 'ISO 639-1'),
    ('lv', 'lav', 'Latvian', 'ISO 639-1'), ('mi', 'mao', 'Maori', 'ISO 639-1'), ('mk', 'mac', 'Macedonian', 'ISO 639-1');"#
}

 fn fill_lang_codes_2_sql <'a>() -> &'a str {          
    r#"insert into lup.lang_codes(code, marc_code, name, source) values
    ('ml', 'mal', 'Malayalam', 'ISO 639-1'), ('mn', 'mon', 'Mongolian', 'ISO 639-1'), ('mr', 'mar', 'Marathi', 'ISO 639-1'),
    ('ms', 'may', 'Malay', 'ISO 639-1'), ('mt', 'mlt', 'Maltese', 'ISO 639-1'), ('mu', 'mul', 'Multiple languages', 'PubMed'),
    ('my', 'bur', 'Burmese', 'ISO 639-1'), ('ne', 'nep', 'Nepali', 'ISO 639-1'), ('nl', 'dut', 'Dutch', 'ISO 639-1'),  
    ('no', 'nor', 'Norwegian', 'ISO 639-1'), ('os', 'oss', 'Ossetian', 'ISO 639-1'), ('pa', 'pan', 'Punjabi', 'ISO 639-1'),
    ('pl', 'pol', 'Polish', 'ISO 639-1'), ('ps', 'pus', 'Pashto', 'ISO 639-1'), ('pt', 'por', 'Portuguese', 'ISO 639-1'),
    ('qu', 'que', 'Quechua', 'ISO 639-1'), ('rm', 'roh', 'Romansh', 'ISO 639-1'), ('ro', 'rum', 'Romanian, Moldavian', 'ISO 639-1'),
    ('ru', 'rus', 'Russian', 'ISO 639-1'), ('rw', 'kin', 'Kinyarwanda', 'ISO 639-1'), ('se', 'sme', 'Northern Sami', 'ISO 639-1'),
    ('si', 'sin', 'Sinhalese', 'ISO 639-1'), ('sk', 'slo', 'Slovak', 'ISO 639-1'), ('sl', 'slv', 'Slovenian', 'ISO 639-1'),
    ('sm', 'smo', 'Samoan', 'ISO 639-1'), ('sn', 'sna', 'Shona', 'ISO 639-1'), ('so', 'som', 'Somali', 'ISO 639-1'),
    ('sq', 'alb', 'Albanian', 'ISO 639-1'), ('sr', 'srp', 'Serbian', 'ISO 639-1'), ('sv', 'swe', 'Swedish', 'ISO 639-1'),
    ('sw', 'swa', 'Swahili', 'ISO 639-1'), ('ta', 'tam', 'Tamil', 'ISO 639-1'), ('te', 'tel', 'Telugu', 'ISO 639-1'),
    ('tg', 'tgk', 'Tajik', 'ISO 639-1'), ('th', 'tha', 'Thai', 'ISO 639-1'), ('tk', 'tuk', 'Turkmen', 'ISO 639-1'),
    ('to', 'ton', 'Tongan', 'ISO 639-1'), ('tr', 'tur', 'Turkish', 'ISO 639-1'), ('tt', 'tat', 'Tatar', 'ISO 639-1'), 
    ('ty', 'tah', 'Tahitian', 'ISO 639-1'), ('uk', 'ukr', 'Ukrainian', 'ISO 639-1'), ('un', 'und', 'Undetermined', 'PubMed'),
    ('ur', 'urd', 'Urdu', 'ISO 639-1'), ('uz', 'uzb', 'Uzbek', 'ISO 639-1'), ('vi', 'vie', 'Vietnamese', 'ISO 639-1'),
    ('xh', 'xho', 'Xhosa', 'ISO 639-1'), ('yo', 'yor', 'Yoruba', 'ISO 639-1'), ('zh', 'chi', 'Chinese', 'ISO 639-1'),
    ('zu', 'zul', 'Zulu', 'ISO 639-1');"#
}

fn fill_lang_codes_3_sql <'a>() -> &'a str {
    r#"insert into lup.lang_codes(code, marc_code, name, source) values
    ('aa', 'aar', 'Afar', 'ISO 639-1'), ('ab', 'abk', 'Abkhazian', 'ISO 639-1'), ('as', 'asm', 'Assamese', 'ISO 639-1'),
    ('ba', 'bak', 'Bashkir', 'ISO 639-1'), ('bi', 'bis', 'Bislama', 'ISO 639-1'), ('ch', 'cha', 'Chamorro', 'ISO 639-1'),
    ('cu', 'chu', 'Church Slavonic', 'ISO 639-1'), ('dv', 'div', 'Divehi', 'ISO 639-1'), ('dz', 'dzo', 'Dzongkha', 'ISO 639-1'),
    ('fo', 'fao', 'Faroese', 'ISO 639-1'), ('fy', 'fry', 'Western Frisian', 'ISO 639-1'), ('gv', 'glv', 'Manx', 'ISO 639-1'),
    ('ht', 'hat', 'Haitian', 'ISO 639-1'), ('ki', 'kik', 'Kikuyu', 'ISO 639-1'), ('kr', 'kau', 'Kanuri', 'ISO 639-1'),
    ('ky', 'kir', 'Kyrgyz', 'ISO 639-1'), ('lu', 'lub', 'Luba-Katanga', 'ISO 639-1'), ('mg', 'mlg', 'Malagasy', 'ISO 639-1'),
    ('na', 'nau', 'Nauru', 'ISO 639-1'), ('nb', 'nob', 'Norwegian Bokmål', 'ISO 639-1'), ('nn', 'nno', 'Norwegian Nynorsk', 'ISO 639-1'),
    ('ny', 'nya', 'Chichewa', 'ISO 639-1'), ('oc', 'oci', 'Occitan', 'ISO 639-1'), ('oj', 'oji', 'Ojibwa', 'ISO 639-1'),
    ('om', 'orm', 'Oromo', 'ISO 639-1'), ('or', 'ori', 'Oriya', 'ISO 639-1'), ('sa', 'san', 'Sanskrit', 'ISO 639-1'),
    ('sd', 'snd', 'Sindhi', 'ISO 639-1'), ('st', 'sot', 'Southern Sotho', 'ISO 639-1'), ('ti', 'tir', 'Tigrinya', 'ISO 639-1'),
    ('tl', 'tgl', 'Tagalog', 'ISO 639-1'), ('ug', 'uig', 'Uighur', 'ISO 639-1');"#
}


fn fill_script_codes_1_sql <'a>() -> &'a str {
    r#"insert into lup.lang_scripts(code, unicode_name, iso_name, dir, chars, notes, hex_start, hex_end, ascii_start, ascii_end, source) 
    values 
    ('Adlm', 'Adlam',  'Adlam', 'RtL', 88, 'Used in parts of West and Central Africa', '1E900', '1E95F', 125184, 125279, 'ISO 15924'),
    ('Arab', 'Arabic', 'Arabic', 'RtL', 1365, '', '0600', '06FF', 1536, 1791, 'ISO 15924'), 
    ('Armn', 'Armenian', 'Armenian', 'LtR', 96, '', '0530', '058F', 1328, 1423, 'ISO 15924'), 
    ('Bali', 'Balinese', 'Balinese', 'LtR', 124, '', '1B00', '1B7F', 6912, 7039, 'ISO 15924'), 
    ('Batk', 'Batak', 'Batak', 'LtR', 56, 'Used in Indonesia', '1BC0', '1BFF', 7104, 7167, 'ISO 15924'),
    ('Beng', 'Bengali', 'Bengali (Bangla)', 'LtR', 96, '', '0980', '09FF', 2432, 2559, 'ISO 15924'), 
    ('Bopo', 'Bopomofo', 'Bopomofo', 'LtR', 77, 'A Chinese transliteration system for Mandarin Chinese and related languages, mostly used in Taiwan', '3100', '312F', 12544, 12591, 'ISO 15924'), 
    ('Bugi', 'Buginese', 'Buginese', 'LtR', 30, 'Used in parts of Indonesia', '1A00', '1A1F', 6656, 6687, 'ISO 15924'), 
    ('Buhd', 'Buhid', 'Buhid', 'LtR', 20, 'Used in parts of the Philippines', '1740', '175F', 5952, 5983, 'ISO 15924'), 
    ('Cakm', 'Chakma', 'Chakma', 'LtR', 71, 'Used in parts of India and Bangla Desh', '11100', '1114F', 69888, 69967, 'ISO 15924'), 
    ('Cham', 'Cham', 'Cham', 'LtR', 83, 'Used in parts of Vietnam and Cambodia', 'AA00', 'AA5F', 43520, 43615, 'ISO 15924'), 
    ('Zyyy', 'Common', 'Code for undetermined script', 'n/a', 0, '', '', '', 0, 0, 'ISO 15924'), 
    ('Cyrl', 'Cyrillic', 'Cyrillic', 'LtR', 443, '', '0400', '04FF', 1024, 1279, 'ISO 15924'), 
    ('Deva', 'Devanagari', 'Devanagari (Nagari)', 'LtR', 154, 'Used in parts of India, including for Hindi and Marathi', '0900', '097F', 2304, 2431, 'ISO 15924');"#
}

fn fill_script_codes_2_sql <'a>() -> &'a str {
    r#"insert into lup.lang_scripts(code, unicode_name, iso_name, dir, chars, notes, hex_start, hex_end, ascii_start, ascii_end, source) 
    values 
    ('Ethi', 'Ethiopic', 'Ethiopic (Geʻez)', 'LtR', 523, 'Used for Amharic and related languages in and around Ethiopa', '1200', '137C', 4608, 4988, 'ISO 15924'), 
    ('Geor', 'Georgian', 'Georgian (Mkhedruli and Mtavruli)', 'LtR', 173, '', '10A0', '10FF', 4256, 4351, 'ISO 15924'), 
    ('Grek', 'Greek', 'Greek', 'LtR', 518, '', '0370', '03FF', 880, 1023, 'ISO 15924'), 
    ('Gujr', 'Gujarati', 'Gujarati', 'LtR', 91, '', '0A80', '0AFF', 2688, 2815, 'ISO 15924'), 
    ('Gong', 'Gunjala Gondi', 'Gunjala Gondi', 'LtR', 63, 'Used in parts of India', '11D60', '11DAF', 73056, 73135, 'ISO 15924'), 
    ('Guru', 'Gurmukhi', 'Gurmukhi', 'LtR', 80, 'Used in parts of India (mainly Punjab)', '0A00', '0A7F', 2560, 2687, 'ISO 15924'), 
    ('Hani', 'Han', 'Han (Hanzi, Kanji, Hanja)', 'TtB, RtL', 94215, 'Chinese characters (including those in Japanese Kanji)', '4E00', '9FFF', 19968, 40959, 'ISO 15924'), 
    ('Hang', 'Hangul', 'Hangul (Hangŭl, Hangeul)', 'LtR, VRtL', 11739, 'The Korean alphabet', 'AC00', 'D7AF', 44032, 55215, 'ISO 15924'), 
    ('Rohg', 'Hanifi Rohingya', 'Hanifi Rohingya', 'RtL', 50, 'Used by the Rohingya people in Burma', '10D00', '10D3F', 68864, 68927, 'ISO 15924'), 
    ('Hano', 'Hanunoo', 'Hanunoo (Hanunóo)', 'LtR, BtT ', 21, 'Used in parts of the Philippines', '1720', '173F', 5920, 5951, 'ISO 15924'), 
    ('Hebr', 'Hebrew', 'Hebrew', 'RtL', 134, '', '0590', '05FF', 1424, 1535, 'ISO 15924'), 
    ('Hira', 'Hiragana', 'Hiragana', 'VRtL, LtR', 380, 'Used in Japan for verbs, words not covered by Kanji or as a more informal form than Kanji', '3040', '309F', 12352, 12447, 'ISO 15924'),
    ('Jpan', 'Han, Hiragana, Katakana', 'Japanese', 'varies', null, 'Alias for Han + Hiragana + Katakana', '', '', 0, 0, 'ISO 15924'), 
    ('Java', 'Javanese', 'Javanese', 'LtR', 90, '', 'A980', 'A9DF', 43392, 43487, 'ISO 15924'), 
    ('Knda', 'Kannada', 'Kannada', 'LtR', 90, 'Used in parts of India (mainly the South)', '0C80', '0CFF', 3200, 3327, 'ISO 15924'), 
    ('Kana', 'Katakana', 'Katakana', 'VRtL, LtR', 320, 'Used in Japan for loan words and many scientific, technical terms', '30A0', '30FF', 12448, 12543, 'ISO 15924'), 
    ('Khmr', 'Khmer', 'Khmer', 'LtR', 146, 'Used in Cambodia', '1780', '17FF', 6016, 6143, 'ISO 15924'), 
    ('Sind', 'Khudawadi', 'Khudawadi, Sindhi', 'LtR', 69, 'Used in parts of India', '112B0', '112FF', 70320, 70399, 'ISO 15924'), 
    ('Geok', 'Georgian', 'Khutsuri (Asomtavruli and Nuskhuri)', 'LtR', null, 'Three different related scripts', '', '', 0, 0, 'ISO 15924'), 
    ('Laoo', 'Lao', 'Lao', 'LtR', 82, 'Used in Laos', '0E80', '0EFF', 3712, 3839, 'ISO 15924'), 
    ('Latn', 'Latin', 'Latin', 'LtR', 1475, '', '0000', '02FF', 0, 767, 'ISO 15924'), 
    ('Latn2', 'Latin Extended', 'Latin Extended', 'LtR', 255, 'Specialist characters used in romanised Vietnamese and a few other languages', '1E00', '1EFF', 7680, 7935, 'web'), 
    ('Lepc', 'Lepcha', 'Lepcha (Róng)', 'LtR', 74, 'Used in parts of India, Tibet', '1C00', '1C4F', 7168, 7247, 'ISO 15924'), 
    ('Limb', 'Limbu', 'Limbu', 'LtR', 68, 'Used in parts of India, Tibet', '1900', '194F', 6400, 6479, 'ISO 15924'), 
    ('Mlym', 'Malayalam', 'Malayalam', 'LtR', 118, 'Used in parts of India (Kerala)', '0D00', '0D7F', 3328, 3455, 'ISO 15924'), 
    ('Mtei', 'Meetei Mayek', 'Meitei Mayek (Meithei, Meetei)', 'LtR', 79, 'Used in parts of India', 'ABC0', 'ABFF', 43968, 44031, 'ISO 15924'), 
    ('Mend', 'Mende Kikakui', 'Mende Kikakui', 'RtL', 213, 'Used  in Sierra Leone', '1E800', '1E8DF', 124928, 125151, 'ISO 15924');"#
}

fn fill_script_codes_3_sql <'a>() -> &'a str {
    r#"insert into lup.lang_scripts(code, unicode_name, iso_name, dir, chars, notes, hex_start, hex_end, ascii_start, ascii_end, source) 
    values 
    ('Plrd', 'Miao', 'Miao (Pollard)', 'LtR', 149, 'Used in parts of China', '16F00', '16F9F', 93952, 94111, 'ISO 15924'), 
    ('Mong', 'Mongolian', 'Mongolian', 'VLtR, LtR', 168, '', '1800', '18AF', 6144, 6319, 'ISO 15924'), 
    ('Mroo', 'Mro', 'Mro, Mru', 'LtR', 43, 'Used in parts of Myanmar and Bangla Desh', '16A40', '16A6F', 92736, 92783, 'ISO 15924'), 
    ('Mymr', 'Myanmar', 'Myanmar (Burmese)', 'LtR', 223, '', '1000', '109F', 4096, 4255, 'ISO 15924'), 
    ('Nkoo', 'NKo', 'N’Ko', 'RtL', 62, 'Used in parts of West Africa', '07C0', '07FF', 1984, 2047, 'ISO 15924'), 
    ('Talu', 'New Tai Lue', 'New Tai Lue', 'LtR', 83, 'Used in parts of China and its southern neighbours', '1980', '19DF', 6528, 6623, 'ISO 15924'), 
    ('Newa', 'Newa', 'Newa, Newar, Newari, Nepāla lipi', 'LtR', 97, 'Used in Nepal', '11400', '1147F', 70656, 70783, 'ISO 15924'), 
    ('Olck', 'Ol Chiki', 'Ol Chiki (Ol Cemet’, Ol, Santali)', 'LtR', 48, 'Used in parts of India', '1C50', '1C7F', 7248, 7295, 'ISO 15924'), 
    ('Orya', 'Oriya', 'Oriya (Odia)', 'LtR', 91, 'Used in parts of India', '0B00', '0B7F', 2816, 2943, 'ISO 15924'), 
    ('Hmng', 'Pahawh Hmong', 'Pahawh Hmong', 'LtR', 127, 'Used in parts of China and its southern neighbours', '16B00', '16B8F', 92928, 93071, 'ISO 15924'), 
    ('Pauc', 'Pau Cin Hau', 'Pau Cin Hau', 'LtR', 57, 'Used in parts of Burma', '11AC0', '11AFF', 72384, 72447, 'ISO 15924'), 
    ('Saur', 'Saurashtra', 'Saurashtra', 'LtR', 82, 'Used in parts of India', 'A880', 'A8DF', 43136, 43231, 'ISO 15924'), 
    ('Sinh', 'Sinhala', 'Sinhala', 'LtR', 111, 'Used in Sri Lanka', '0D80', '0DFF', 3456, 3583, 'ISO 15924'), 
    ('Sund', 'Sundanese', 'Sundanese', 'LtR', 72, 'Used in parts of Indonesia', '1B80', '1BBF', 7040, 7103, 'ISO 15924'), 
    ('Tglg', 'Tagalog', 'Tagalog (Baybayin, Alibata)', 'LtR', 23, 'Used in parts of the Philippines', '1700', '171F', 5888, 5919, 'ISO 15924'), 
    ('Tagb', 'Tagbanwa', 'Tagbanwa', 'LtR', 18, 'Used in parts of the Philippines', '1760', '177F', 5984, 6015, 'ISO 15924'), 
    ('Tale', 'Tai Le', 'Tai Le', 'LtR', 35, 'Used in parts of China', '1950', '197F', 6480, 6527, 'ISO 15924'), 
    ('Lana', 'Tai Tham', 'Tai Tham (Lanna)', 'LtR', 127, 'Used in parts of Thailand', '1A20', '1AAF', 6688, 6831, 'ISO 15924'), 
    ('Tavt', 'Tai Viet', 'Tai Viet', 'LtR', 72, 'Used in parts of Thailand', 'AA80', 'AADF', 43648, 43743, 'ISO 15924'), 
    ('Taml', 'Tamil', 'Tamil', 'LtR', 123, 'Used in parts of India', '0B80', '0BFF', 2944, 3071, 'ISO 15924'), 
    ('Telu', 'Telugu', 'Telugu', 'LtR', 100, 'Used in parts of India', '0C00', '0C7F', 3072, 3199, 'ISO 15924'), 
    ('Thaa', 'Thaana', 'Thaana', 'RtL', 50, 'Used in the Maldives', '0780', '07BF', 1920, 1983, 'ISO 15924'), 
    ('Thai', 'Thai', 'Thai', 'LtR', 86, '', '0E00', '0E7F', 3584, 3711, 'ISO 15924'), 
    ('Tibt', 'Tibetan', 'Tibetan', 'LtR', 207, '', '0F00', '0FFF', 3840, 4095, 'ISO 15924'), 
    ('Cans', 'Canadian Aboriginal', 'Unified Canadian Aboriginal Syllabics', 'LtR', 726, 'Used in Inuit and related languages', '1400', '167F', 5120, 5759, 'ISO 15924'), 
    ('Wara', 'Warang Citi', 'Warang Citi (Varang Kshiti)', 'LtR', 84, 'Used in parts of India', '118A0', '118FF', 71840, 71935, 'ISO 15924'), 
    ('Yiii', 'Yi', 'Yi', 'LtR', 1220, 'Used in parts of China', 'A000', 'A48F', 40960, 42127, 'ISO 15924'),
    ('Latn, Jpan', 'Latin - Japanese mix', 'Latin - Japanese mix', null, null, 'Latin characters mixed with one or more of Han, Hiragana, or Katakana', '', '', 0, 0, 'imp_ror'), 
    ('Latn, Cyrl', 'Latin - Cyrillic mix', 'Latin - Cyrillic mix', null, null, 'Latin characters mixed with Cyrillic', '', '', 0, 0, 'imp_ror'), 
    ('Latn, Hani', 'Latin - Hani mix', 'Latin - Hani mix', null, null, 'Latin characters mixed with Hani (usually Chinese Hanzi)', '', '', 0, 0, 'imp_ror'), 
    ('Latn, Hang', 'Latin - Hangul mix', 'Latin - Hangul mix', null, null, 'Latin characters mixed with Hangul', '', '', 0, 0, 'imp_ror'), 
    ('Latn, Grek', 'Latin - Greek mix', 'Latin - Greek mix', null, null, 'Latin characters mixed with Greek', '', '', 0, 0, 'imp_ror'), 
    ('Latn, Deva', 'Latin - Devanagari mix', 'Latin - Devanagari mix', null, null, 'Latin characters mixed with Devanagari (usualy Hindi)', '', '', 0, 0, 'imp_ror'), 
    ('Latn, Geor', 'Latin - Georgian mix', 'Latin - Georgian mix', null, null, 'Latin characters mixed with Georgian', '', '', 0, 0, 'imp_ror'), 
    ('Deva, Beng', 'Devanagari - Bengali mix', 'Devanagari - Bengali mix', null, null, 'Devanagari characters mixed with Bengali', '', '', 0, 0, 'imp_ror'), 
    ('Hani, Hang', 'Hani - Hangul mix', 'Hani - Hangul mix', null, null, 'Hani (Hanja) characters mixed with Korean Hangul', '', '', 0, 0, 'imp_ror');"#
}


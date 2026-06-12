
pub fn get_sql<'a>() -> &'a str {

    r#"SET client_min_messages TO WARNING; 

    drop table if exists lup.lang_codes;
    create table lup.lang_codes (
        code            varchar     not null primary key
      , marc_code       varchar
      , name            varchar
      , source          varchar
    );
    
    
    insert into lup.lang_codes(code, marc_code, name, source) values
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
        ('lv', 'lav', 'Latvian', 'ISO 639-1'), ('mi', 'mao', 'Maori', 'ISO 639-1'), ('mk', 'mac', 'Macedonian', 'ISO 639-1');
    
    
    insert into lup.lang_codes(code, marc_code, name, source) values
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
        ('zu', 'zul', 'Zulu', 'ISO 639-1');
    
    
    insert into lup.lang_codes(code, marc_code, name, source) values
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
        ('tl', 'tgl', 'Tagalog', 'ISO 639-1'), ('ug', 'uig', 'Uighur', 'ISO 639-1');
    
    SET client_min_messages TO NOTICE;"#
}
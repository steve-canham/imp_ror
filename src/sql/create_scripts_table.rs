
pub fn get_sql<'a>() -> &'a str {

    r#"SET client_min_messages TO WARNING; 

    drop table if exists lup.lang_scripts;
    create table lup.lang_scripts (
        code            varchar     not null primary key
      , unicode_name    varchar
      , iso_name        varchar
      , dir             varchar
      , chars           int
      , notes           varchar
      , hex_start       varchar
      , hex_end         varchar
      , ascii_start     int
      , ascii_end       int
      , source          varchar
    );
    
    
    insert into lup.lang_scripts(code, unicode_name, iso_name, dir, chars, notes, hex_start, hex_end, ascii_start, ascii_end, source) 
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
        ('Deva', 'Devanagari', 'Devanagari (Nagari)', 'LtR', 154, 'Used in parts of India, including for Hindi and Marathi', '0900', '097F', 2304, 2431, 'ISO 15924');
    
    
    insert into lup.lang_scripts(code, unicode_name, iso_name, dir, chars, notes, hex_start, hex_end, ascii_start, ascii_end, source) 
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
        ('Mend', 'Mende Kikakui', 'Mende Kikakui', 'RtL', 213, 'Used  in Sierra Leone', '1E800', '1E8DF', 124928, 125151, 'ISO 15924');
    
    
    insert into lup.lang_scripts(code, unicode_name, iso_name, dir, chars, notes, hex_start, hex_end, ascii_start, ascii_end, source) 
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
        ('Hani, Hang', 'Hani - Hangul mix', 'Hani - Hangul mix', null, null, 'Hani (Hanja) characters mixed with Korean Hangul', '', '', 0, 0, 'imp_ror');

    SET client_min_messages TO NOTICE;"#
}
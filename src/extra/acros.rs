//use sqlx::{Pool, Postgres};
//use crate::AppError;
//use log::info;


/* 

update orgs.ror_names
set lang_code = null,
lang_source = null
where name_type = 10
and lang_source in ('derived acro', 'mono lang org')


select * 
from orgs.ror_names
where name_type = 10
and lang_code is not null
and lang_source <> 'cm_brand'
and lang_source <> 'ror'
and lang_source <> 'script_auto'



----------------------------------------------------
-- get current acronyms
----------------------------------------------------

drop table if exists orgs.ror_acronyms;
create table orgs.ror_acronyms (
     id  varchar
   , name varchar
   , name_to_match varchar
   , der_acro_comp varchar
   , lang_code varchar
   , source_name varchar
   , country_code varchar
);
create index ror_acrs_id on orgs.ror_acronyms(id);
create index ror_acrs_ntm on orgs.ror_acronyms(name_to_match);



insert into orgs.ror_acronyms (id, name, name_to_match, der_acro_comp, country_code)
select n.id, n.name, n.name_to_match, n.name_to_match, ro.country_code
from orgs.ror_names n
inner join orgs.ror_orgs ro 
on n.id = ro.id
where n.name_type = 10
and n.lang_code is null;

--43048

-- remove spaces and dashes to make matching easier

update orgs.ror_acronyms
set der_acro_comp = replace(der_acro_comp, '-', '')
where der_acro_comp like '%-%';

--795

update orgs.ror_acronyms
set der_acro_comp = replace(der_acro_comp, ' ', '')
where der_acro_comp like '% %';

--1430



----------------------------------------------------
-- get derived acronyms
----------------------------------------------------

-- about 165 duplicates of names_to_match in orgs.ror_names
-- therefore select distinct to get an acro base table

drop table if exists orgs.acro_base1;
create table orgs.acro_base1 as
select distinct n.id, n.name_to_match
from orgs.ror_names  n
inner join orgs.ror_acronyms a
on n.id = a.id;

--115641

-- replace hyphens with spaces to split the words

update orgs.acro_base1
set name_to_match = replace(name_to_match, '-', ' ')
where name_to_match like '%-%';

--4941

-- but this will also cause duplication...

drop table if exists orgs.acro_base2;
create table orgs.acro_base2 as
select distinct id, name_to_match
from orgs.acro_base1;

--115610

-- get the individual words in each name_to_match

drop table if exists orgs.ror_acro_words;
create table orgs.ror_acro_words (
     id int not null generated always as identity
   , ror_id  varchar
   , acro_base varchar
   , word varchar
);
create index ror_acro_words_id on orgs.ror_acro_words(id, acro_base);


insert into orgs.ror_acro_words (ror_id, acro_base, word)
select id, name_to_match, regexp_split_to_table(name_to_match, '\s+') as word
	 from orgs.ror_derived_acronyms;

-- 363088

-- construct table to hold derived acronyms and fill it with 
-- all current non acronym names that have one or more linked acronyms

drop table if exists orgs.ror_derived_acronyms;
create table orgs.ror_derived_acronyms (
     id  varchar
   , name varchar
   , name_to_match varchar
   , lang_code varchar
   , acro_base varchar
   , der_acro varchar
   , der_acro_wo_of varchar
   , der_acro_wo_ofand varchar
   , der_acro_wo_allsw varchar
);
create index ror_der_acrs_id on orgs.ror_derived_acronyms(id);
create index ror_der_acrs_ntm on orgs.ror_derived_acronyms(name_to_match);


insert into orgs.ror_derived_acronyms (id, name, name_to_match, lang_code, acro_base)
select n.id, n.name, n.name_to_match, n.lang_code, n.name_to_match
from orgs.ror_names n
inner join 
	(select distinct id from orgs.ror_acronyms) a
on n.id = a.id
where name_type <> 10
and script_code = 'Latn';

--64907

-- recombine the letters to create a full derived acronym value

update orgs.ror_derived_acronyms a
set der_acro = f.ac
from (  
        select ror_id, acro_base, string_agg(substring(word, 1, 1), '' order by id) as ac
		from orgs.ror_acro_words
		group by ror_id, acro_base ) f
where a.id = f.ror_id
and a.acro_base = f.acro_base;

--64838

-- where der_acro is only one letter cannot be an acronym
-- as all are at least two letters

delete from orgs.ror_derived_acronyms 
where length(der_acro) = 1

-- 671

-- remove 'of' and 'the' from listing of name words

delete from orgs.ror_acro_words
where word in ('of', 'the', 'de', 'des', 'du', 'la', 'le', 'les', 'los', 'der', 'del', 'di', 'el', 'za');

-- about 35337 go

-- also need to remove initial l' and d' from (mostly french) words

update orgs.ror_acro_words
set word = substring(word, 3)
where word like 'l’%'

--880

update orgs.ror_acro_words
set word = substring(word, 3)
where word like 'd’%'

--1200

update orgs.ror_derived_acronyms a
set der_acro_wo_of = f.ac
from (  
        select ror_id, acro_base, string_agg(substring(word, 1, 1), '' order by id) as ac
		from orgs.ror_acro_words
		group by ror_id, acro_base ) f
where a.id = f.ror_id
and a.acro_base = f.acro_base;

--64167

-- remove 'and' and '&' from listing of name words

delete from orgs.ror_acro_words
where word in ('and', '&', 'et', 'e', 'und', 'i');

--14258

update orgs.ror_derived_acronyms a
set der_acro_wo_ofand = f.ac
from (  
        select ror_id, acro_base, string_agg(substring(word, 1, 1), '' order by id) as ac
		from orgs.ror_acro_words
		group by ror_id, acro_base ) f
where a.id = f.ror_id
and a.acro_base = f.acro_base;

--64167

-- remove other stop words from listing of name words

delete from orgs.ror_acro_words
where word in ('for', 'für', 'in', 'en', 'y', 'on', 'a', 'v', 'pour', 'per', 'sur', 'à', 'voor', 'o', '/');

-- 12328 go

update orgs.ror_derived_acronyms a
set der_acro_wo_allsw = f.ac
from (  
        select ror_id, acro_base, string_agg(substring(word, 1, 1), '' order by id) as ac
		from orgs.ror_acro_words
		group by ror_id, acro_base ) f
where a.id = f.ror_id
and a.acro_base = f.acro_base;


----------------------------------------------------
-- apply derived acronyms
----------------------------------------------------


update orgs.ror_acronyms y
set lang_code = langs,
source_name = source
from
	(select id, name_to_match, string_agg(source, ', ') as source, string_agg(lang_code, ', ') as langs
	from 
		(select a.id, a.name, a.name_to_match, d.lang_code, d.name_to_match as source
		from orgs.ror_acronyms a
		inner join orgs.ror_derived_acronyms d
		on a.id = d.id
		and a.der_acro_comp = d.der_acro_wo_allsw) as w
	group by id, name_to_match) x
where y.id = x.id
and y.name_to_match = x.name_to_match

-- 26394

update orgs.ror_acronyms y
set lang_code = langs,
source_name = source
from
	(select id, name_to_match, string_agg(source, ', ') as source, string_agg(lang_code, ', ') as langs
	from 
		(select a.id, a.name, a.name_to_match, d.lang_code, d.name_to_match as source
		from orgs.ror_acronyms a
		inner join orgs.ror_derived_acronyms d
		on a.id = d.id
		and a.der_acro_comp = d.der_acro_wo_ofand
		and a.lang_code is null) as w
	group by id, name_to_match) x
where y.id = x.id
and y.name_to_match = x.name_to_match

--282

update orgs.ror_acronyms y
set lang_code = langs,
source_name = source
from
	(select id, name_to_match, string_agg(source, ', ') as source, string_agg(lang_code, ', ') as langs
	from 
		(select a.id, a.name, a.name_to_match, d.lang_code, d.name_to_match as source
		from orgs.ror_acronyms a
		inner join orgs.ror_derived_acronyms d
		on a.id = d.id
		and a.der_acro_comp = d.der_acro_wo_of
		and a.lang_code is null) as w
	group by id, name_to_match) x
where y.id = x.id
and y.name_to_match = x.name_to_match

--149


update orgs.ror_acronyms y
set lang_code = langs,
source_name = source
from
	(select id, name_to_match, string_agg(source, ', ') as source, string_agg(lang_code, ', ') as langs
	from 
		(select a.id, a.name, a.name_to_match, d.lang_code, d.name_to_match as source
		from orgs.ror_acronyms a
		inner join orgs.ror_derived_acronyms d
		on a.id = d.id
		and a.der_acro_comp = d.der_acro
		and a.lang_code is null) as w
	group by id, name_to_match) x
where y.id = x.id
and y.name_to_match = x.name_to_match

--498


update orgs.ror_names n
set lang_code = a.lang_code,
lang_source = 'derived acro'
from
orgs.ror_acronyms a 
where n.id = a.id
and n.name_to_match = a.name_to_match
and n.name_type = 10
and n.lang_code is null
and a.lang_code is not null

--27,323

select count(*)
from orgs.ror_acronyms a
where lang_code is  null

--15,725



-- for argentinian orgs
-- those with CONICET are spanish
-- and those beginning with 'UN' (Universidad Nacional)



-- get orgs with single language non acronym names
-- list the ids and language
   
update orgs.ror_acronyms y
set lang_code = x.lang_code
from
   (select distinct n.id, n.lang_code from orgs.ror_names n
   inner join
		(select id, count(distinct lang_code) from orgs.ror_names
		where name_type <> 10
		group by id
		having count(distinct lang_code) = 1) s
   on n.id = s.id
   where n.name_type <> 10
   and lang_code <> 'cm') x 
where y.id = x.id
and y.lang_code is null


select count(*)
from orgs.ror_acronyms
where lang_code is not null


select y.*, x.lang_code from orgs.ror_acronyms y
inner join 
	(select distinct n.id, n.lang_code 
	from 
	orgs.ror_names n
	inner join 
			(select distinct id
			from orgs.ror_names
			where name_type <> 10
			and lang_code <> 'cm'
			group by id
			having count(distinct lang_code) = 1) d
	on n.id = d.id
	where lang_code is not null) x
on y.id = x.id
where y.lang_code is null
order by y.id
		
--8366


select  y.*, x.lang_code from orgs.ror_acronyms y
inner join 
	(select distinct n.id, n.lang_code 
	from 
	orgs.ror_names n
	inner join 
			(select distinct id
			from orgs.ror_names
			where name_type <> 10
			and lang_code <> 'cm'
			group by id
			having count(lang_code) = 1) d
	on n.id = d.id		
	where lang_code is not null) x
on y.id = x.id	
where y.lang_code is null
order by y.id


--6332


update orgs.ror_names n
set lang_code = a.lang_code,
lang_source = 'mono lang org'
from
orgs.ror_acronyms a 
where n.id = a.id
and n.name_to_match = a.name_to_match
and n.name_type = 10
and n.lang_code is null
and a.lang_code is not null


select * from orgs.ror_acronyms a
left join 
    (select * from orgs.ror_names
	where name_type = 10
	and lang_source = 'mono lang org') m
on a.id = m.id
and a.name_to_match = m.name_to_match
where a.source_name is null
and a.lang_code is not null
and m.id is null
order by a.id


select distinct id 
from orgs.ror_names
where name_type = 10
and lang_code is null


select rn.*, ro.country_code from orgs.ror_names rn 
inner join  
		(select distinct id from orgs.ror_names 
		where lang_code is null) m
on rn.id= m.id
inner join orgs.ror_orgs ro 
on rn.id = ro.id
order by country_code, id, name_type



-- for argentinian orgs
-- those with CONICET are spanish
-- and those beginning with 'UN' (Universidad Nacional)

-- for brazil if they start with UF (Universidade Federale)
-- they are portuguese


-- for UK, Australia, US - make them english
-- also Chinese, Japanes (Korean? ) names

-- almost all arab countries

-- Some african countries topo


-- if the derived acronym is part of the actual acronym ??
-- pre-expand US, NHS so that they come back to the original (otherwise they become U, N etc.)



*/

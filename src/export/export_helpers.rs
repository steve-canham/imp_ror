use sqlx::{Pool, Postgres};
use crate::AppError;

#[derive(sqlx::FromRow)]
struct CPars {
    cname: String,
    colname: String,
}

pub async fn set_up_country_grid(pool: &Pool<Postgres>) -> Result<(), AppError> {

    // Create a temp table with the countries listed as being in the top 25
    
    let sql = r#"drop table if exists smm.temp_clist;
                create table smm.temp_clist (
                    id int primary key generated always as identity,
                    cname varchar,
                    colname varchar
                );

                insert into smm.temp_clist (cname, colname)
                select entity,
                lower(replace(entity, ' ', '_'))
                from smm.ranked_distributions
                where dist_type = 3
                and entity <> 'Remaining countries'
                group by entity
                order by sum(number) desc;

                insert into smm.temp_clist (cname, colname)
                values ('Remaining_countries', 'remaining_countries');"#;
    
    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Use that to create the sql that will generate the countries grid table
    // and create that table

    let sql = r#"select string_agg(' '||colname||'  float', ',') 
                    from (select colname from smm.temp_clist order by id)"#;

    let mid_string: String = sqlx::query_scalar(sql).fetch_one(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    
    let start_sql = r#"drop table if exists smm.countries_grid; 
                create table smm.countries_grid (vcode  varchar, vdate  date,"#;
    let sql = format!("{} {});", start_sql, mid_string);

    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql))?;

    // Add a row to the countries_grid table for each version

    let sql = r#"insert into smm.countries_grid (vcode, vdate)
                       select vcode, vdate from smm.version_summaries
                       where vcode <> 'v1.57'
                       order by vcode"#;
    sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    // Fill in the grid values
    // Obtain the colum / country names from the temp clist table
    // Construct the sqwl for each country to update the grid
    
    let sql = r#"select cname, colname from smm.temp_clist"#;
    let cpars: Vec<CPars> = sqlx::query_as(&sql).fetch_all(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    for cp in cpars {

        let sql = format!(r#"update smm.countries_grid cg
            set {} = round(rd.pc_of_base_set::numeric, 2)
            from smm.ranked_distributions rd
            where cg.vcode = rd.vcode
            and rd.entity = '{}'"#, cp.colname, cp.cname);

        sqlx::raw_sql(&sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;
    }

    // Drop the temporary country grid table

    let sql = "drop table smm.temp_clist"; 
    sqlx::raw_sql(sql).execute(pool)
        .await.map_err(|e| AppError::SqlxError(e, sql.to_string()))?;

    Ok(())

}




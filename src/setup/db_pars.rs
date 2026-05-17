use crate::err::AppError;
use crate::setup::config_reader::DB_PARS;
use std::time::Duration;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, PgPool};
use sqlx::ConnectOptions;

// Only a single database and database pool is used within the sytem

pub async fn get_db_pool() -> Result<PgPool, AppError> {

    // Use the static DB_PARS objecu to get the connection string
    // Use the string to set up a connection options object and change
    // the time threshold for warnings. Set up a DB pool option and
    // connect using the connection options object.

    let db_pars = DB_PARS.get()
        .ok_or_else(|| AppError::MissingDBParameters())?;
    let db_conn_string = format!("postgres://{}:{}@{}:{}/{}",
        db_pars.db_user, db_pars.db_password, db_pars.db_host, db_pars.db_port, db_pars.db_name);
    let mut conn_object: PgConnectOptions = db_conn_string.parse()
        .map_err(|e| AppError::DBPoolError("Problem with parsing conection string".to_string(), e))?;
    conn_object = conn_object.log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(3));

    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(conn_object).await
        .map_err(|e| AppError::DBPoolError(format!("Problem with connecting to database {} and obtaining Pool", db_pars.db_name), e))
}

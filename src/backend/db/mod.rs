pub mod init;
pub mod project_managing;

use dioxus::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn connection() -> Result<SqlitePool, ServerFnError> {
    let options = SqliteConnectOptions::from_str("sqlite://projectimer.db")
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .create_if_missing(true)
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
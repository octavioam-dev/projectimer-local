use dioxus::prelude::*;
use crate::backend::db::connection as db;

// ================================================================
//                         INIT FUNCTION
// ================================================================
pub async fn init_db() -> Result<(), ServerFnError> {
    let pool = db().await?;
    // ----------------------------------------------------------------
    //                     CREATE INDIVIDUAL TABLES
    // ----------------------------------------------------------------
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            email TEXT UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tags (
            tag TEXT PRIMARY KEY UNIQUE
        );"
    ).execute(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    // ----------------------------------------------------------------
    //                       CREATE 1-infg TABLES
    // ----------------------------------------------------------------
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            revision TEXT,
            client_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
        );"
    ).execute(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS times (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            ti DATETIME NOT NULL,
            tf DATETIME NOT NULL,
            notes TEXT,
            user_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );"
    ).execute(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    // ----------------------------------------------------------------
    //                       CREATE infg-infg TABLES
    // ----------------------------------------------------------------
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS time_tags (
            time_id INTEGER NOT NULL,
            tag TEXT NOT NULL,
            PRIMARY KEY (time_id, tag),
            FOREIGN KEY (time_id) REFERENCES times(id) ON DELETE CASCADE,
            FOREIGN KEY (tag) REFERENCES tags(tag) ON DELETE CASCADE
        );"
    ).execute(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
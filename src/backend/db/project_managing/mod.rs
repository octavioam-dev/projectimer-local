use dioxus::prelude::*;
use crate::backend::db::connection as db;
use crate::backend::data::{Client, Project, Tag};

pub async fn fetch_clients() -> Result<Vec<Client>, ServerFnError> {
    let pool = db().await?;
    let clients = sqlx::query_as::<_, Client>(
        "SELECT id, name, email, created_at FROM clients"
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(clients)
}

pub fn mark_clients_change() {
    dioxus::document::eval("localStorage.setItem('clients_change', 'true');");
}

/*pub async fn fetch_projects() -> Result<Vec<Project>, ServerFnError> {
    let pool = db().await?;
    let projects = sqlx::query_as::<_, Project>(
        "SELECT
            projects.id,
            projects.name,
            projects.client_id,
            clients.name AS client_name,
            projects.created_at
         FROM projects
         INNER JOIN clients ON clients.id = projects.client_id"
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(projects)
}*/
pub async fn fetch_projects_by_client(client_id: i64) -> Result<Vec<Project>, ServerFnError> {
    let pool = db().await?;
    let projects = sqlx::query_as::<_, Project>(
        "SELECT
            projects.id,
            projects.name,
            projects.client_id,
            projects.revision,
            clients.name AS client_name,
            projects.created_at
         FROM projects
         INNER JOIN clients ON clients.id = projects.client_id
         WHERE projects.client_id = ?"
    )
        .bind(client_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(projects)
}

pub fn mark_projects_changed(client_id: i64) {
    dioxus::document::eval(&format!(
        r#"
        const key = 'projects_changed_client_ids';
        const set = new Set(JSON.parse(localStorage.getItem(key) || '[]'));
        set.add({client_id});
        localStorage.setItem(key, JSON.stringify([...set]));
        "#
    ));
}

pub async fn fetch_tags() -> Result<std::collections::HashSet<Tag>, ServerFnError> {
    let pool = db().await?;
    let tags = sqlx::query_as::<_, Tag>("SELECT tag FROM tags")
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(tags.into_iter().collect::<std::collections::HashSet<Tag>>())
}
use dioxus::prelude::*;

pub(crate) async fn project_flag_set_for_client(client_id: i64) -> bool {
    let mut eval = document::eval(&format!(
        r#"
        const key = 'projects_changed_client_ids';
        const set = new Set(JSON.parse(localStorage.getItem(key) || '[]'));
        const wasChanged = set.has({client_id});
        localStorage.setItem(key, '[]');
        dioxus.send(wasChanged);
        "#
    ));
    eval.recv::<bool>().await.unwrap_or(false)
}
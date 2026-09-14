use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::backend::data::{Client, Project};
use crate::backend::db::project_managing::{fetch_clients, fetch_projects_by_client};

use icons::{DioxusIcon, IconType};
use crate::backend::helpers::project_flag_set_for_client;
use crate::ui::button::{Button, ButtonVariant};
use crate::components::combobox::GenericCombobox;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum CountDirection {
    Up,
    Down
}
#[component]
pub(crate) fn ClockWindow() -> Element {
    let mut count:Signal<u8> = use_signal(|| 0);

    let selected_client: Signal<Option<Client>> = use_signal(|| None);
    let mut selected_project: Signal<Option<Project>> = use_signal(|| None);

    let mut clients_resource = use_resource(move || async move {
        fetch_clients().await.unwrap_or_default()
    });
    let mut projects_resource = use_resource(move || async move {
        match selected_client() {
            Some(client) => fetch_projects_by_client(client.id).await.unwrap_or_default(),
            None => Vec::new(),
        }
    });
    use_effect(move || {
        selected_client(); // establish reactive dependency
        selected_project.set(None);
    });

    // After any project list refresh, keep the selection if it still exists, else clear it.
    use_effect(move || {
        let projects = projects_resource.value().read().clone().unwrap_or_default();
        if let Some(current) = selected_project() {
            if !projects.iter().any(|p| p.id == current.id) {
                selected_project.set(None);
            }
        }
    });

    // Watches window focus + a localStorage flag, and tells Rust when to refetch.
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
        (function() {
            const CLIENTS_KEY = 'clients_changed';
            const PROJECTS_KEY = 'projects_changed_client_ids';

            if (localStorage.getItem(CLIENTS_KEY) === null) {
                localStorage.setItem(CLIENTS_KEY, 'false');
            }
            if (localStorage.getItem(PROJECTS_KEY) === null) {
                localStorage.setItem(PROJECTS_KEY, '[]');
            }

            const checkClients = (force) => {
                const changed = localStorage.getItem(CLIENTS_KEY) === 'true';
                if (changed) localStorage.setItem(CLIENTS_KEY, 'false');
                if (changed || force) dioxus.send('clients');
            };

            // Unconditional refresh on app open.
            checkClients(true);
            dioxus.send('focus');

            window.addEventListener('focus', () => {
                checkClients(false);
                dioxus.send('focus');
            });
        })();
        "#,
        );

        loop {
            match eval.recv::<String>().await {
                Ok(msg) => match msg.as_str() {
                    "clients" => {
                        clients_resource.restart();
                    }
                    "focus" => {
                        if let Some(client) = selected_client() {
                            if project_flag_set_for_client(client.id).await {
                                projects_resource.restart();
                            }
                        }
                    }
                    _ => {}
                },
                Err(_) => break,
            }
        }
    });

    let clients = clients_resource.value().read().clone().unwrap_or_default();
    let projects = projects_resource.value().read().clone().unwrap_or_default();

    rsx! {
        GenericCombobox::<Client> {
            data: clients,
            placeholder: "Select a client".to_string(),
            selected_item: selected_client,
        }
         GenericCombobox::<Project> {
            data: projects,
            placeholder: "Select a project".to_string(),
            selected_item: selected_project,
        }
        Button {
            variant: ButtonVariant::Secondary,
            onclick: move |_| async move {
                if let Ok(new_count) = server_count(CountDirection::Down, count()).await {
                    count.set(new_count);
                };
            },
            DioxusIcon { icon: IconType::Minus, class: None }
        },
    }
}

#[server]
async fn server_count(direction: CountDirection, number: u8) -> Result<u8, ServerFnError> {
    match direction {
        CountDirection::Up => Ok(if number < u8::MAX { number + 1 } else { number }),
        CountDirection::Down => Ok(if number > 0 { number - 1 } else { number }),
    }
}
use dioxus::prelude::*;
use std::collections::HashSet;
use std::time::Duration;
use dioxus::logger::tracing;
use crate::components::combobox::GenericCombobox;
use crate::backend::data::{Client, Project, Tag};
use crate::backend::db::project_managing::{fetch_clients, fetch_projects_by_client, fetch_tags};
use crate::ui::multi_select::{MultiSelect, MultiSelectContent, MultiSelectGroup, MultiSelectItem, MultiSelectOption, MultiSelectTrigger, MultiSelectValue, };
use crate::backend::helpers::project_flag_set_for_client;
use crate::components::multiselect::GenericMultiSelect;

const TIMER_ACCENT: &str = "#F2A93B";
const LAP_SECONDS: u64 = 1800;
const TOTAL_TICKS: u64 = 12;

fn format_elapsed(total_seconds: u64) -> String {
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[component]
pub fn ClockWindow() -> Element {
    let mut selected_client: Signal<Option<Client>> = use_signal(|| None);
    let mut selected_project: Signal<Option<Project>> = use_signal(|| None);
    let mut selected_tags: Signal<HashSet<Tag>> = use_signal(HashSet::new);

    let mut clients_resource = use_resource(move || async move {
        fetch_clients().await.unwrap_or_default()
    });

    let mut projects_resource = use_resource(move || async move {
        match selected_client() {
            Some(client) => fetch_projects_by_client(client.id).await.unwrap_or_default(),
            None => Vec::new(),
        }
    });

    let mut tags_resource = use_resource(move || async move {
        fetch_tags().await.unwrap_or_default()
    });

    let mut tags_cache = use_signal(|| HashSet::<Tag>::new());
    use_effect(move || {
        if let Some(tags) = tags_resource.value().read().clone() {
            tags_cache.set(tags);
        }
    });

    let clients = clients_resource.value().read().clone().unwrap_or_default();
    let projects = projects_resource.value().read().clone().unwrap_or_default();
    let tags: HashSet<Tag> = tags_cache();

    //tracing::info!("tags count: {}", tags.len());

    use_effect(move || {
        selected_client();
        selected_project.set(None);
    });

    use_effect(move || {
        let projects = projects_resource.value().read().clone().unwrap_or_default();
        if let Some(current) = selected_project() {
            if !projects.iter().any(|p| p.id == current.id) {
                selected_project.set(None);
            }
        }
    });

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
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
        (function() {
            dioxus.send('focus');
            window.addEventListener('focus', () => {
                dioxus.send('focus');
            });
        })();
        "#,
        );

        loop {
            match eval.recv::<String>().await {
                Ok(_) => {
                    tags_resource.restart();
                }
                Err(_) => break,
            }
        }
    });
    use_effect(move || {
        let current = selected_tags();
        let retained: HashSet<Tag> = current
            .into_iter()
            .filter(|t| tags_cache().contains(t))
            .collect();
        if retained != selected_tags() {
            selected_tags.set(retained);
        }
    });

    let mut is_running = use_signal(|| false);
    let mut elapsed_seconds = use_signal(|| 0u64);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if is_running() {
                elapsed_seconds.set(elapsed_seconds() + 1);
            }
        }
    });

    let is_disabled = selected_client().is_none() || selected_project().is_none();

    let toggle_tracking = move |_| {
        if is_disabled {
            return;
        }
        if is_running() {
            is_running.set(false);
            elapsed_seconds.set(0);
        } else {
            is_running.set(true);
        }
    };

    let status_text = if is_disabled {
        "Select a client and project"
    } else if is_running() {
        "Tracking time"
    } else {
        "Ready to start"
    };

    // --- Ring + tick geometry ---
    let radius = 78.0_f64;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let laps_completed = (elapsed_seconds() / LAP_SECONDS).min(TOTAL_TICKS);
    let progress_fraction = (elapsed_seconds() % LAP_SECONDS) as f64 / LAP_SECONDS as f64;
    let dash_offset = circumference * (1.0 - progress_fraction);
    let track_color = "var(--color-border)";

    let tick_positions: Vec<(f64, f64, bool)> = (0..TOTAL_TICKS)
        .map(|i| {
            let angle_deg = i as f64 * (360.0 / TOTAL_TICKS as f64);
            let angle_rad = (angle_deg - 90.0).to_radians();
            let tick_radius = 69.0;
            let x = 90.0 + tick_radius * angle_rad.cos();
            let y = 90.0 + tick_radius * angle_rad.sin();
            (x, y, i < laps_completed)
        })
        .collect();

    let button_style = if is_disabled {
        "background-color: var(--color-muted);".to_string()
    } else if is_running() {
        format!(
            "background-color: var(--color-card); border: 2px solid {TIMER_ACCENT}; box-shadow: 0 0 0 6px rgba(242,169,59,0.14);"
        )
    } else {
        format!(
            "background-color: {TIMER_ACCENT}; box-shadow: 0 10px 28px -10px rgba(242,169,59,0.55);"
        )
    };

    let button_class = if is_disabled {
        "relative flex items-center justify-center rounded-full transition-all duration-200 cursor-not-allowed"
    } else {
        "relative flex items-center justify-center rounded-full transition-all duration-200 hover:brightness-105 active:scale-[0.97] focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-background"
    };

    rsx! {
        div { class: "flex flex-col w-[300px] h-[450px] bg-background px-6 py-7 gap-3",

            h1 { class: "text-lg font-semibold text-foreground text-center tracking-tight",
                "ProjecTimer"
            }

            div { class: "flex-1 flex flex-col items-center justify-center gap-3",
                div { class: "relative", style: "width: 180px; height: 180px;",
                    svg {
                        width: "180",
                        height: "180",
                        view_box: "0 0 180 180",

                        circle {
                            cx: "90", cy: "90", r: "{radius}",
                            fill: "none",
                            stroke: "{track_color}",
                            stroke_width: "6",
                        }
                        circle {
                            cx: "90", cy: "90", r: "{radius}",
                            fill: "none",
                            stroke: "{TIMER_ACCENT}",
                            stroke_width: "6",
                            stroke_linecap: "round",
                            stroke_dasharray: "{circumference}",
                            stroke_dashoffset: "{dash_offset}",
                            transform: "rotate(-90 90 90)",
                            style: "transition: stroke-dashoffset 1s linear;",
                        }
                        for (x, y, lit) in tick_positions.iter() {
                            circle {
                                cx: "{x}", cy: "{y}", r: "3.5",
                                fill: if *lit { TIMER_ACCENT } else { track_color },
                            }
                        }
                    }

                    div { class: "absolute inset-0 flex items-center justify-center",
                        button {
                            class: "{button_class}",
                            style: "width: 120px; height: 120px; {button_style}",
                            disabled: is_disabled,
                            onclick: toggle_tracking,
                            if is_running() {
                                span { class: "text-xl font-semibold tabular-nums", style: "color: {TIMER_ACCENT};",
                                    "{format_elapsed(elapsed_seconds())}"
                                }
                            } else if is_disabled {
                                icons::Play { class: "size-9 text-muted-foreground" }
                            } else {
                                icons::Play { class: "size-9 text-[#171310]" }
                            }
                        }
                    }
                }

                p { class: "text-xs text-muted-foreground",
                    "{status_text}"
                }
            }

            div { class: "flex gap-2 w-full",
                GenericCombobox::<Client> {
                    data: clients,
                    placeholder: "Client".to_string(),
                    selected_item: selected_client,
                    trigger_class: "flex-1 min-w-0",
                }
                GenericCombobox::<Project> {
                    data: projects,
                    placeholder: "Project".to_string(),
                    selected_item: selected_project,
                    trigger_class: "flex-1 min-w-0",
                }
            }
            div { class: "flex gap-2 w-full",
                GenericMultiSelect::<Tag> {
                    list: tags,
                    placeholder: "Select tags".to_string(),
                    selected: selected_tags,
                }
            }
        }
    }
}
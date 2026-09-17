mod components;
//dx serve
mod backend;
mod ui;
#[cfg(test)]
mod render_probe;
#[cfg(feature = "desktop")]
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

mod clock_window;
pub mod hooks;
pub mod constants;

use clock_window::ClockWindow;

use crate::constants::CLOCK_WINDOW_DIMENSIONS;
use crate::backend::db::init::init_db;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    //#[layout(Navbar)]
    #[route("/")]
    ClockWindow {},
    /*#[route("/blog/:id")]
    Blog { id: i32 },*/
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const DX_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    #[cfg(feature = "desktop")]
    {
        let config = Config::new().with_window(
            WindowBuilder::new()
                .with_title("ProjecTimer")
                .with_inner_size(LogicalSize::new(CLOCK_WINDOW_DIMENSIONS.width, CLOCK_WINDOW_DIMENSIONS.height)),
        );
        dioxus::LaunchBuilder::desktop()
            .with_cfg(config)
            .launch(App);
    }
    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_effect(move || {
        spawn(async move {
            if let Err(e) = init_db().await {
                error!("failed to init db: {e}");
            }
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: DX_THEME_CSS }
        Router::<Route> {}
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        "Home"
    }
}

/*/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}*/

/*/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[post("/api/echo")]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(format!("Echo: {input}"))
}*/

#![allow(non_snake_case)]

use dioxus_router::prelude::*;

use dioxus::prelude::*;
use log::LevelFilter;

use anyhow::anyhow;
use url::Url;

pub static BASE_API_URL: &str = "https://matrix.radical.directory/_matrix/client/v3/publicRooms";

#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse {
    pub chunk: Vec<Room>,
    pub next_batch: String,
    pub prev_batch: String,
    pub total_room_count_estimate: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct Room {
    pub avatar_url: String,
    pub guest_can_join: bool,
    pub join_rule: String,
    pub name: String,
    pub num_joined_members: i32,
    pub room_id: String,
    pub room_type: String,
    pub topic: String,
    pub world_readable: bool,
}

pub async fn get_public_rooms() -> Result<ApiResponse, reqwest::Error> {
    let url = format!("{}", BASE_API_URL);
    reqwest::get(&url).await?.json::<ApiResponse>().await
}

async fn login() -> Result<(), anyhow::Error> {
    let homeserver_url = Url::parse("http://matrix.radical.directory")?;
    let client = Client::new(homeserver_url).await?;

    // First, let's figure out what login types are supported by the homeserver.
    let mut choices = Vec::new();
    let login_types = client.matrix_auth().get_login_types().await?.flows;

    for login_type in login_types {
        match login_type {
                LoginType::Password(_) => {
                    choices.push(LoginChoice::Password)
                }
                LoginType::Sso(sso) => {
                    if sso.identity_providers.is_empty() {
                        choices.push(LoginChoice::Sso)
                    } else {
                        choices.extend(sso.identity_providers.into_iter().map(LoginChoice::SsoIdp))
                    }
                }
                // This is used for SSO, so it's not a separate choice.
                LoginType::Token(_) |
                // This is only for application services, ignore it here.
                LoginType::ApplicationService(_) => {},
                // We don't support unknown login types.
                _ => {},
            }
    }

    match choices.len() {
        0 => {
            return Err(anyhow!(
                "Homeserver login types incompatible with this client"
            ))
        }
        1 => choices[0].login(&client).await?,
        _ => offer_choices_and_login(&client, choices).await?,
    }
}

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    println!("app");

    //let rooms = use_future(cx, (), |_| get_public_rooms());

    // match rooms.value() {
    //     Some(Ok(rooms)) => {
    //         println!("{:?}", rooms);
    //     }
    //     Some(Err(e)) => {
    //         println!("{:?}", e);
    //     }
    //     None => {
    //         println!("None");
    //     }
    // }

    render! {
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

#[inline_props]
fn Blog(cx: Scope, id: i32) -> Element {
    render! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[inline_props]
fn Home(cx: Scope) -> Element {
    let mut count = use_state(cx, || 0);

    cx.render(rsx! {
        Link {
            to: Route::Blog {
                id: *count.get()
            },
            "Go to blog"
        }
        div {
            h1 { "High-Five counter: {count}" }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }

        }
    })
}

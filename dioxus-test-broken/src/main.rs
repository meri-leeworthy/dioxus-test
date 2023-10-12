#![allow(non_snake_case)]
#![allow(dead_code)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use log::LevelFilter;
use matrix_sdk::{
    ruma::api::client::session::get_login_types::v3::{IdentityProvider, LoginType},
    Client,
};
use std::fmt::{self, Error};
use url::Url;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub async fn login() -> Result<JsValue, JsError> {
    let homeserver_url = Url::parse("http://matrix.radical.directory")?;
    let client = Client::new(homeserver_url).await?;

    // First, let's figure out what login types are supported by the homeserver.
    let mut choices = Vec::new();
    let login_types = client.get_login_types().await?.flows;

    for login_type in login_types {
        match login_type {
            LoginType::Password(_) => choices.push(LoginChoice::Password),
            LoginType::Sso(sso) => {
                if sso.identity_providers.is_empty() {
                    choices.push(LoginChoice::Sso)
                } else {
                    choices.extend(sso.identity_providers.into_iter().map(LoginChoice::SsoIdp))
                }
            }
            // This is used for SSO, so it's not a separate choice.
            LoginType::Token(_) => {}
            // We don't support unknown login types.
            _ => {}
        }
    }

    match choices.len() {
        0 => {
            return Err(JsError::new(&format!(
                "Homeserver login types incompatible with this client"
            )))
        }
        1 => return Ok("One login type found".to_string().into()), //choices[0].login(&client).await?,
        _ => return Ok("Several login types found".to_string().into()), // offer_choices_and_login(&client, choices).await?,
    };
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
    // let login_types = use_future(cx, (), |_| login());
    let login_types: Option<Result<String, Error>> = Some(Ok("test".to_string()));

    cx.render(match login_types {
        Some(Ok(login_types)) => {
            println!("{:?}", login_types);
            rsx! {
                p { "Login types: {login_types}" }
            }
        }
        Some(Err(e)) => {
            println!("{:?}", e);
            rsx! {
                p { "Error: {e}" }
            }
        }
        None => {
            println!("None");
            rsx! {
                p { "Loading..." }
            }
        }
    })
}

enum LoginChoice {
    /// Login with username and password.
    Password,

    /// Login with SSO.
    Sso,

    /// Login with a specific SSO identity provider.
    SsoIdp(IdentityProvider),
}

impl LoginChoice {
    /// Login with this login choice.
    async fn login(&self, _client: &Client) -> anyhow::Result<()> {
        match self {
            LoginChoice::Password => Ok(()), //login_with_password(client).await,
            LoginChoice::Sso => Ok(()),      //login_with_sso(client, None).await,
            LoginChoice::SsoIdp(_idp) => Ok(()), //login_with_sso(client, Some(idp)).await,
        }
    }
}

impl fmt::Display for LoginChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoginChoice::Password => write!(f, "Username and password"),
            LoginChoice::Sso => write!(f, "SSO"),
            LoginChoice::SsoIdp(idp) => write!(f, "SSO via {}", idp.name),
        }
    }
}

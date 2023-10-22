mod db;
mod discord_webhook;
mod fcm_wrapper;
mod notification_types;
mod szkolny_api;
mod szkolny_fcm;

use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize)]
struct GeneralConfig {
    db_path: String,
}

#[derive(Deserialize)]
struct DiscordConfig {
    webhook_url: String,
}

#[derive(Deserialize)]
struct SzkolnyConfig {
    api_key: String,
    fcm_sender_id: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct LibrusConfig {
    teams: Vec<String>,
    subjects: HashMap<String, String>,
    teachers: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct Config {
    general: GeneralConfig,
    discord: DiscordConfig,
    szkolny: SzkolnyConfig,
    #[allow(dead_code)]
    librus: LibrusConfig,
}

fn load_config() -> Config {
    let config_file = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_file).unwrap();
    config
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() {
    println!("Starting app...");

    print!(" > Loading config... ");
    let config = load_config();
    println!("✓");

    print!(" > Connecting to database... ");
    let database = db::connect(PathBuf::from(config.general.db_path)).unwrap();
    println!("✓");

    let mut default_headers = HeaderMap::new();
    default_headers.insert("X-ApiKey", config.szkolny.api_key.parse().unwrap());

    let http_client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(default_headers)
        .build()
        .unwrap();

    let fcm_registration: fcm_push_listener::Registration =
        match db::get_data(&database, "fcm_registration").unwrap() {
            Some(registration) => registration,
            None => {
                print!(" > Registering with FCM... ");
                let registration = szkolny_fcm::register(&config.szkolny.fcm_sender_id)
                    .await
                    .unwrap();
                db::set_data(&database, "fcm_registration", &registration).unwrap();
                println!("✓");
                registration
            }
        };

    println!("FCM token: {}", fcm_registration.fcm_token);

    if let Some(browser_id) = db::get_data_raw(&database, "browser_id").unwrap() {
        println!(" > Contacting api.szkolny.eu...");
        szkolny_api::print_registered_devices(
            &http_client,
            &String::from_utf8_lossy(&browser_id).to_string(),
        )
        .await;
        println!(
            "Pair token: {}",
            String::from_utf8_lossy(&db::get_data_raw(&database, "pair_token").unwrap().unwrap())
                .to_string()
        );
        println!(
            "Browser ID: {}",
            String::from_utf8_lossy(&db::get_data_raw(&database, "browser_id").unwrap().unwrap())
                .to_string()
        );
    } else {
        println!(" > Registering with Szkolny.eu webPush API... ");
        let (browser_id, pair_token) =
            szkolny_api::register_browser(&http_client, &fcm_registration.fcm_token).await;

        db::set_data_raw(&database, "browser_id", browser_id.as_bytes().to_vec()).unwrap();
        db::set_data_raw(&database, "pair_token", pair_token.as_bytes().to_vec()).unwrap();
    }

    println!("Starting FCM listener...");
    szkolny_fcm::run(
        fcm_registration,
        database,
        config.discord.webhook_url,
        &http_client,
        &config.librus,
    )
    .await;
}

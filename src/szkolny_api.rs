use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBrowserBody {
    action: String,
    push_token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBrowserResponseDataBrowser {
    browser_id: String,
    pair_token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBrowserResponseData {
    browser: RegisterBrowserResponseDataBrowser,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBrowserResponse {
    data: RegisterBrowserResponseData,
}

pub async fn register_browser(client: &reqwest::Client, fcm_token: &String) -> (String, String) {
    let response = client
        .post("https://api.szkolny.eu/webPush")
        .json(&RegisterBrowserBody {
            action: "registerBrowser".to_owned(),
            push_token: fcm_token.to_string(),
        })
        .send()
        .await
        .unwrap();

    let data: RegisterBrowserResponse = response.json().await.unwrap();

    println!("Browser ID: {}", data.data.browser.browser_id);
    println!("Pair token: {}", data.data.browser.pair_token);

    (data.data.browser.browser_id, data.data.browser.pair_token)
}

pub async fn print_registered_devices(client: &reqwest::Client, browser_id: &String) {
    let response = client
        .post("https://api.szkolny.eu/webPush")
        .json(&HashMap::from([
            ("action", "listDevices"),
            ("browserId", &browser_id),
        ]))
        .send()
        .await
        .unwrap();
    println!("Paired devices: {}", response.text().await.unwrap());
}

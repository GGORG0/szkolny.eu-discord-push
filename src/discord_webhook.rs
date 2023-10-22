use std::error::Error;

use discord_message::DiscordMessage;
use serde::Deserialize;
use url::Url;

use crate::{notification_types, LibrusConfig};

#[derive(Deserialize)]
struct SzkolnyNotification {
    #[serde(rename = "type")]
    notification_type: String,
    title: String,
    message: String,
}

pub async fn process_message(
    json: String,
    webhook_url: String,
    client: &reqwest::Client,
    librus_config: &LibrusConfig,
) -> Result<(), Box<dyn Error>> {
    let fcm_message: serde_json::Value = serde_json::from_str(&json)?;

    let szkolny_notification: SzkolnyNotification =
        serde_json::from_value(fcm_message["data"].clone())?;

    if szkolny_notification.notification_type == "syncNotify" {
        return Ok(());
    }

    let processed = notification_types::process_notification(&fcm_message["data"], &librus_config);

    let discord_message = DiscordMessage {
        avatar_url: None,
        username: None,
        content: "".to_owned(),
        embeds: vec![discord_message::Embed {
            author: processed.author.map(|author| discord_message::EmbedAuthor {
                name: author,
                url: None,
                icon_url: None,
            }),

            title: szkolny_notification.message,
            description: processed.description.unwrap_or("".to_owned()),
            color: Some(0x02a0e9),
            footer: Some(discord_message::EmbedFooter {
                text: format!(
                    "{} / {}",
                    szkolny_notification.title, szkolny_notification.notification_type
                ),
                icon_url: Some(Url::parse("https://szkolny.eu/images/logo.png").unwrap()),
            }),
            fields: Some(
                processed
                    .fields
                    .into_iter()
                    .map(|field| discord_message::EmbedField {
                        title: field.name,
                        value: field.value,
                        inline: true,
                    })
                    .collect(),
            ),
            ..Default::default()
        }],
    };

    send_message(discord_message, webhook_url, client).await?;

    Ok(())
}

async fn send_message(
    message: DiscordMessage,
    webhook_url: String,
    client: &reqwest::Client,
) -> Result<(), reqwest::Error> {
    let mut retries = 0;
    loop {
        match client
            .post(&webhook_url)
            .header("Content-Type", "application/json")
            .body(message.to_json().unwrap())
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                eprintln!("  -> Failed to send message to Discord! {}", e);
                if retries >= 5 {
                    return Err(e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                retries += 1;
            }
        }
    }
}

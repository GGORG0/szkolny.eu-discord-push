use fcm_push_listener::Registration;
use futures::StreamExt;

use crate::{db, discord_webhook, fcm_wrapper::FcmMessageStream, LibrusConfig};

pub async fn run(
    registration: Registration,
    database: rusqlite::Connection,
    webhook_url: String,
    client: &reqwest::Client,
    librus_config: &LibrusConfig,
) {
    let notifications = db::get_notifications(&database).unwrap();

    let mut message_stream = FcmMessageStream::new(registration, notifications)
        .await
        .unwrap();

    println!(" > Listening for messages...");

    while let Some(message) = message_stream.next().await {
        println!("  -> Message JSON: {}", message.payload_json);

        if let Err(_) = discord_webhook::process_message(
            message.payload_json,
            webhook_url.clone(),
            client,
            librus_config,
        )
        .await
        {
            continue;
        }

        db::add_notification(&database, &message.persistent_id.as_ref().unwrap()).unwrap();
    }

    eprintln!("FCM message stream ended!");
}

pub async fn register(sender_id: &str) -> Result<Registration, fcm_push_listener::Error> {
    let registration = fcm_push_listener::register(sender_id).await?;

    Ok(registration)
}

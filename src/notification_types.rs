use time::{Date, Month, Time};

use crate::LibrusConfig;

pub fn process_notification(
    notification: &serde_json::Value,
    librus_config: &LibrusConfig,
) -> NotificationEmbed {
    let notification_type = notification["type"].as_str().unwrap_or("null");

    let processor: Box<dyn NotificationProcessor> = match notification_type {
        "sharedEvent" => Box::new(shared_event_notification::SharedEventProcessor {}),
        "unsharedEvent" => Box::new(unshared_event_notification::UnsharedEventProcessor {}),
        _ => Box::new(other_notification::OtherNotificationProcessor {}),
    };

    processor.process(notification, librus_config)
}

pub struct NotificationEmbedField {
    pub name: String,
    pub value: String,
}

pub struct NotificationEmbed {
    pub author: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<NotificationEmbedField>,
}

trait NotificationProcessor {
    fn process(
        &self,
        notification: &serde_json::Value,
        librus_config: &LibrusConfig,
    ) -> NotificationEmbed;
}

fn szkolny_date_convert(date: u64) -> Date {
    let year = (date / 10000) as i32;
    let month = ((date % 10000) / 100) as u8;
    let day = (date % 100) as u8;

    Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap()
}

fn szkolny_time_convert(time: u64) -> Time {
    let hour = (time / 10000) as u8;
    let minute = ((time % 10000) / 100) as u8;
    let second = (time % 100) as u8;

    Time::from_hms(hour, minute, second).unwrap()
}

mod shared_event_notification {
    use super::*;

    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SzkolnyEvent {
        #[serde(rename = "type")]
        event_type: i32,
        subject_id: i32,
        shared_by_name: String,
        teacher_id: i32,
        team_code: String,
        topic: String,
        start_time: Option<u64>,
        id: u64,
        event_date: u64,
    }

    pub struct SharedEventProcessor {}

    impl NotificationProcessor for SharedEventProcessor {
        fn process(
            &self,
            notification: &serde_json::Value,
            librus_config: &LibrusConfig,
        ) -> NotificationEmbed {
            let event: SzkolnyEvent =
                serde_json::from_str(notification["event"].as_str().unwrap()).unwrap();

            let embed = NotificationEmbed {
                author: Some(event.shared_by_name),
                description: Some(event.topic),
                fields: vec![
                    NotificationEmbedField {
                        name: "Grupa".to_owned(),
                        value: event.team_code.to_string(),
                    },
                    NotificationEmbedField {
                        name: "Przedmiot".to_owned(),
                        value: if event.subject_id != -1 {
                            librus_config.subjects[&event.subject_id.to_string()].clone()
                        } else {
                            "Brak przedmiotu".to_owned()
                        },
                    },
                    NotificationEmbedField {
                        name: "Nauczyciel".to_owned(),
                        value: if event.teacher_id != -1 {
                            librus_config.teachers[&event.teacher_id.to_string()].clone()
                        } else {
                            "Brak nauczyciela".to_owned()
                        },
                    },
                    NotificationEmbedField {
                        name: "Data".to_owned(),
                        value: szkolny_date_convert(event.event_date).to_string(),
                    },
                    NotificationEmbedField {
                        name: "Godzina".to_owned(),
                        value: match event.start_time {
                            Some(time) => szkolny_time_convert(time).to_string(),
                            None => "Cały dzień".to_string(),
                        },
                    },
                    NotificationEmbedField {
                        name: "Typ".to_owned(),
                        value: event.event_type.to_string(),
                    },
                    NotificationEmbedField {
                        name: "ID".to_owned(),
                        value: event.id.to_string(),
                    },
                ],
            };

            embed
        }
    }
}

mod unshared_event_notification {
    use super::*;

    pub struct UnsharedEventProcessor {}

    impl NotificationProcessor for UnsharedEventProcessor {
        fn process(
            &self,
            notification: &serde_json::Value,
            _librus_config: &LibrusConfig,
        ) -> NotificationEmbed {
            let embed = NotificationEmbed {
                author: None,
                description: None,
                fields: vec![
                    NotificationEmbedField {
                        name: "Grupa".to_owned(),
                        value: notification["unshareTeamCode"].as_str().unwrap().to_owned(),
                    },
                    NotificationEmbedField {
                        name: "ID".to_owned(),
                        value: notification["eventId"].as_str().unwrap().to_owned(),
                    },
                ],
            };

            embed
        }
    }
}

mod other_notification {
    use super::*;

    pub struct OtherNotificationProcessor {}

    impl NotificationProcessor for OtherNotificationProcessor {
        fn process(
            &self,
            notification: &serde_json::Value,
            _librus_config: &LibrusConfig,
        ) -> NotificationEmbed {
            let embed = NotificationEmbed {
                author: None,
                description: Some(format!("```json\n{}\n```", notification.to_string())),
                fields: vec![],
            };

            embed
        }
    }
}

use crate::models::EventChange;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::json;

pub async fn send_notification(webhook_url: &str, changes: &[EventChange]) -> Result<()> {
    let client = reqwest::Client::new();

    let mut embeds = Vec::new();

    for change in changes {
        let embed = match change {
            EventChange::Added(event) => {
                let start_dt = DateTime::<Utc>::from_timestamp(event.start, 0).unwrap_or_default();
                let end_dt = DateTime::<Utc>::from_timestamp(event.end, 0).unwrap_or_default();

                json!({
                    "title": format!("✅ New Event: {}", event.title),
                    "color": 3066993,
                    "fields": [
                        {
                            "name": "Room",
                            "value": &event.room,
                            "inline": true
                        },
                        {
                            "name": "Instructor",
                            "value": &event.instructor,
                            "inline": true
                        },
                        {
                            "name": "Start",
                            "value": start_dt.format("%Y-%m-%d %H:%M UTC").to_string(),
                            "inline": false
                        },
                        {
                            "name": "End",
                            "value": end_dt.format("%Y-%m-%d %H:%M UTC").to_string(),
                            "inline": false
                        }
                    ]
                })
            }
            EventChange::Modified {
                old: _,
                new,
                changes: change_list,
            } => {
                let start_dt = DateTime::<Utc>::from_timestamp(new.start, 0).unwrap_or_default();

                json!({
                    "title": format!("🔄 Event Modified: {}", new.title),
                    "color": 15105570,
                    "description": format!("**Changes:**\n{}", change_list.join("\n")),
                    "fields": [
                        {
                            "name": "Room",
                            "value": &new.room,
                            "inline": true
                        },
                        {
                            "name": "Date",
                            "value": start_dt.format("%Y-%m-%d %H:%M UTC").to_string(),
                            "inline": true
                        }
                    ]
                })
            }
            EventChange::Deleted(event) => {
                let start_dt = DateTime::<Utc>::from_timestamp(event.start, 0).unwrap_or_default();

                json!({
                    "title": format!("❌ Event Deleted: {}", event.title),
                    "color": 15158332,
                    "fields": [
                        {
                            "name": "Room",
                            "value": &event.room,
                            "inline": true
                        },
                        {
                            "name": "Date",
                            "value": start_dt.format("%Y-%m-%d %H:%M UTC").to_string(),
                            "inline": true
                        }
                    ]
                })
            }
        };

        embeds.push(embed);
    }

    for chunk in embeds.chunks(10) {
        let payload = json!({
            "embeds": chunk
        });

        let response = client.post(webhook_url).json(&payload).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Discord webhook failed: HTTP {}", response.status());
        }
    }

    Ok(())
}

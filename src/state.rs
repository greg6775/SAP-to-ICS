use crate::ics_generator;
use crate::models::{EventChange, SAPEvent};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct AppState {
    pub sap_url: String,
    pub cookie: String,
    pub discord_webhook: Option<String>,
    events: RwLock<HashMap<String, SAPEvent>>,
}

impl AppState {
    pub fn new(sap_url: String, cookie: String, discord_webhook: Option<String>) -> Self {
        Self {
            sap_url,
            cookie,
            discord_webhook,
            events: RwLock::new(HashMap::new()),
        }
    }

    pub async fn update_events(&self, new_events: Vec<SAPEvent>) -> Vec<EventChange> {
        let mut events = self.events.write().await;
        let mut changes = Vec::new();

        let new_events_map: HashMap<String, SAPEvent> = new_events
            .into_iter()
            .map(|e| (e.generate_uid(), e))
            .collect();

        let old_uids: Vec<String> = events.keys().cloned().collect();
        for uid in old_uids {
            if let Some(old_event) = events.get(&uid) {
                if let Some(new_event) = new_events_map.get(&uid) {
                    let change_list = old_event.compare(new_event);
                    if !change_list.is_empty() {
                        changes.push(EventChange::Modified {
                            old: Box::new(old_event.clone()),
                            new: Box::new(new_event.clone()),
                            changes: change_list,
                        });
                    }
                } else {
                    changes.push(EventChange::Deleted(Box::new(old_event.clone())));
                }
            }
        }

        for (uid, new_event) in &new_events_map {
            if !events.contains_key(uid) {
                changes.push(EventChange::Added(Box::new(new_event.clone())));
            }
        }

        *events = new_events_map;

        changes
    }

    pub async fn get_ics_calendar(&self) -> String {
        let events = self.events.read().await;
        let events_vec: Vec<SAPEvent> = events.values().cloned().collect();
        ics_generator::generate_ics(&events_vec)
    }
}

use crate::models::SAPEvent;
use chrono::{DateTime, Utc};
use icalendar::{Calendar, Component, Event, EventLike};

pub fn generate_ics(events: &[SAPEvent]) -> String {
    let mut calendar = Calendar::new().name("SAP Campus Dual Schedule").done();

    for sap_event in events {
        let start_dt = DateTime::<Utc>::from_timestamp(sap_event.start, 0).unwrap_or_default();
        let end_dt = DateTime::<Utc>::from_timestamp(sap_event.end, 0).unwrap_or_default();

        let event = Event::new()
            .uid(&sap_event.generate_uid())
            .summary(&sap_event.title)
            .description(&format!(
                "{}\n\nRoom: {}\nInstructor: {}\n{}",
                sap_event.description,
                sap_event.room,
                sap_event.instructor,
                if !sap_event.remarks.is_empty() {
                    format!("Remarks: {}", sap_event.remarks)
                } else {
                    String::new()
                }
            ))
            .location(&sap_event.room)
            .starts(start_dt)
            .ends(end_dt)
            .done();

        calendar.push(event);
    }

    calendar.to_string()
}

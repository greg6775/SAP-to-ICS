use crate::encoding_fix::fix_mojibake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SAPEvent {
    pub title: String,
    pub start: i64,
    pub end: i64,
    pub description: String,
    pub room: String,
    pub instructor: String,
    pub remarks: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventChange {
    Added(Box<SAPEvent>),
    Modified {
        old: Box<SAPEvent>,
        new: Box<SAPEvent>,
        changes: Vec<String>,
    },
    Deleted(Box<SAPEvent>),
}

impl SAPEvent {
    pub fn fix_encoding(mut self) -> Self {
        self.title = fix_mojibake(&self.title);
        self.description = fix_mojibake(&self.description);
        self.room = fix_mojibake(&self.room);
        self.instructor = fix_mojibake(&self.instructor);
        self.remarks = fix_mojibake(&self.remarks);
        self
    }

    pub fn generate_uid(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.title.as_bytes());
        hasher.update(self.start.to_string().as_bytes());
        hasher.update(self.end.to_string().as_bytes());
        hasher.update(self.room.as_bytes());
        let result = hasher.finalize();
        format!("sap-{}", hex::encode(&result[..8]))
    }

    pub fn compare(&self, other: &SAPEvent) -> Vec<String> {
        let mut changes = Vec::new();

        if self.title != other.title {
            changes.push(format!("Title: '{}' -> '{}'", self.title, other.title));
        }
        if self.start != other.start {
            changes.push("Start time changed".to_string());
        }
        if self.end != other.end {
            changes.push("End time changed".to_string());
        }
        if self.room != other.room {
            changes.push(format!("Room: '{}' -> '{}'", self.room, other.room));
        }
        if self.instructor != other.instructor {
            changes.push(format!(
                "Instructor: '{}' -> '{}'",
                self.instructor, other.instructor
            ));
        }
        if self.remarks != other.remarks {
            changes.push(format!(
                "Remarks: '{}' -> '{}'",
                self.remarks, other.remarks
            ));
        }

        changes
    }
}

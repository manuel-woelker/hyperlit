pub use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct LastModificationInfo {
    pub date: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

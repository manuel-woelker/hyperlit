use crate::document_info::DocumentInfo;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct SiteInfo {
    pub title: String,
    pub documents: Vec<DocumentInfo>,
}

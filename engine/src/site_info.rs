use crate::document_info::DocumentInfo;
use hyperlit_base::shared_string::SharedString;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct SiteInfo {
    pub title: SharedString,
    pub documents: Vec<DocumentInfo>,
}

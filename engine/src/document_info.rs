use hyperlit_base::shared_string::SharedString;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct DocumentInfo {
    pub id: SharedString,
    pub title: SharedString,
}

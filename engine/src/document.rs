use hyperlit_base::shared_string::SharedString;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Document {
    pub id: SharedString,
    pub title: SharedString,
    pub markdown: SharedString,
}

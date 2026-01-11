use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub markdown: String,
}

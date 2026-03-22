#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub content_length: usize,
    pub content_type: Option<String>,
    pub version: String,
    pub body: Option<Vec<u8>>,
}
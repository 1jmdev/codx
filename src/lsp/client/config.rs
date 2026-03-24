#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub language_id: String,
    pub command: String,
    pub args: Vec<String>,
}

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub language_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub initialization_options: Option<Value>,
}

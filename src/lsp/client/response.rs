use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

impl RpcResponse {
    pub fn into_result(self, method: &str) -> Result<Value, String> {
        if let Some(result) = self.result {
            return Ok(result);
        }
        if let Some(error) = self.error {
            return Err(format!("lsp request {method} failed: {error}"));
        }
        Err(format!("lsp request {method} returned no result"))
    }
}

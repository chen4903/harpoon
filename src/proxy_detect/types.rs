use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    Eip1167,
    Eip1967Direct,
    Eip1967Beacon,
    Eip1822,
    Eip2535Diamond,
    Eip897,
    OpenZeppelin,
    Safe,
    Comptroller,
    BatchRelayer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProxyResult {
    Single {
        target: String,
        #[serde(rename = "type")]
        proxy_type: ProxyType,
        immutable: bool,
    },
    Diamond {
        target: Vec<String>,
        #[serde(rename = "type")]
        proxy_type: ProxyType,
        immutable: bool,
    },
}

pub type BlockTag = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestArguments {
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

/// JSON-RPC request function trait for proxy detection
pub trait JsonRpcRequester: Clone {
    type Future: Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send;
    fn call(&self, args: RequestArguments) -> Self::Future;
}

// Auto-implement for any function matching the signature
impl<F, Fut> JsonRpcRequester for F
where
    F: Fn(RequestArguments) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send,
{
    type Future = Fut;
    fn call(&self, args: RequestArguments) -> Self::Future {
        self(args)
    }
}

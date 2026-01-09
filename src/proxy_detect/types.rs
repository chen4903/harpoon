use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

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
        target: Address,
        #[serde(rename = "type")]
        proxy_type: ProxyType,
        immutable: bool,
    },
    Diamond {
        target: Vec<Address>,
        #[serde(rename = "type")]
        proxy_type: ProxyType,
        immutable: bool,
    },
}

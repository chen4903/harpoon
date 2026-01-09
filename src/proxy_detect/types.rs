use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    NotProxy,
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

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ProxyType::NotProxy => write!(f, "NotProxy"),
            ProxyType::Eip1167 => write!(f, "Eip1167"),
            ProxyType::Eip1967Direct => write!(f, "Eip1967Direct"),
            ProxyType::Eip1967Beacon => write!(f, "Eip1967Beacon"),
            ProxyType::Eip1822 => write!(f, "Eip1822"),
            ProxyType::Eip2535Diamond => write!(f, "Eip2535Diamond"),
            ProxyType::Eip897 => write!(f, "Eip897"),
            ProxyType::OpenZeppelin => write!(f, "OpenZeppelin"),
            ProxyType::Safe => write!(f, "Safe"),
            ProxyType::Comptroller => write!(f, "Comptroller"),
            ProxyType::BatchRelayer => write!(f, "BatchRelayer"),
        }
    }
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

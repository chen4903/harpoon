pub mod detector;
pub mod eip1167;
pub mod read_string;
pub mod types;

pub use detector::detect_proxy;
pub use eip1167::parse_1167_bytecode;
pub use read_string::read_string;
pub use types::{JsonRpcRequester, ProxyResult, ProxyType, RequestArguments};

use crate::proxy_detect::{
    eip1167::parse_1167_bytecode,
    read_string::read_string,
    types::{JsonRpcRequester, ProxyResult, ProxyType, RequestArguments},
};
use serde_json::json;

// Storage slots for various proxy patterns
const EIP_1967_LOGIC_SLOT: &str = "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
const OPEN_ZEPPELIN_IMPLEMENTATION_SLOT: &str = "0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3";
const EIP_1822_LOGIC_SLOT: &str = "0xc5f16f0fcc639fa48a6947836d9850f504798523bf8c9a3a87d5876cf622bcf7";
const EIP_1967_BEACON_SLOT: &str = "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50";

// Function signatures (method IDs) for proxy detection
const EIP_897_IMPLEMENTATION: &str = "0x5c60da1b00000000000000000000000000000000000000000000000000000000";
const EIP_897_PROXY_TYPE: &str = "0x4555d5c900000000000000000000000000000000000000000000000000000000";
const EIP_1967_BEACON_IMPLEMENTATION: &str = "0x5c60da1b00000000000000000000000000000000000000000000000000000000";
const EIP_1967_BEACON_CHILD_IMPLEMENTATION: &str = "0xda52571600000000000000000000000000000000000000000000000000000000";
const SAFE_MASTER_COPY: &str = "0xa619486e00000000000000000000000000000000000000000000000000000000";
const COMPTROLLER_IMPLEMENTATION: &str = "0xbb82aa5e00000000000000000000000000000000000000000000000000000000";
const BATCH_RELAYER_VERSION: &str = "0x54fd4d5000000000000000000000000000000000000000000000000000000000";
const BATCH_RELAYER_GET_LIBRARY: &str = "0x7678922e00000000000000000000000000000000000000000000000000000000";
const EIP_2535_FACET_ADDRESSES: &str = "0x52ef6b2c00000000000000000000000000000000000000000000000000000000";

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

#[derive(Debug)]
pub struct DetectorError(String);

impl std::fmt::Display for DetectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DetectorError {}

/// Main proxy detection function
pub async fn detect_proxy<F>(proxy_address: &str, json_rpc_request: F, block_tag: Option<&str>) -> Option<ProxyResult>
where
    F: JsonRpcRequester,
{
    let block_tag = block_tag.unwrap_or("latest");

    // Try each detection method sequentially, return first successful one
    if let Ok(Some(result)) = detect_eip1167(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip1967_direct(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip1967_beacon(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_open_zeppelin(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip1822(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip897(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_safe(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_comptroller(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_batch_relayer(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip2535_diamond(proxy_address, json_rpc_request.clone(), block_tag).await {
        return Some(result);
    }

    None
}

// EIP-1167 Minimal Proxy Contract
async fn detect_eip1167<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let bytecode = json_rpc_request
        .call(RequestArguments {
            method: "eth_getCode".to_string(),
            params: vec![json!(proxy_address), json!(block_tag)],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let bytecode_str = bytecode.as_str().ok_or(DetectorError("Invalid bytecode".to_string()))?;
    let target = parse_1167_bytecode(bytecode_str).map_err(|_| DetectorError("Not EIP-1167".to_string()))?;

    let target = read_address(&target)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1167,
        immutable: true,
    }))
}

// EIP-1967 Direct Proxy
async fn detect_eip1967_direct<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let storage = json_rpc_request
        .call(RequestArguments {
            method: "eth_getStorageAt".to_string(),
            params: vec![json!(proxy_address), json!(EIP_1967_LOGIC_SLOT), json!(block_tag)],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let storage_str = storage.as_str().ok_or(DetectorError("Invalid storage".to_string()))?;
    let target = read_address(storage_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1967Direct,
        immutable: false,
    }))
}

// EIP-1967 Beacon Proxy
async fn detect_eip1967_beacon<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let storage = json_rpc_request
        .clone()
        .call(RequestArguments {
            method: "eth_getStorageAt".to_string(),
            params: vec![json!(proxy_address), json!(EIP_1967_BEACON_SLOT), json!(block_tag)],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let storage_str = storage.as_str().ok_or(DetectorError("Invalid storage".to_string()))?;
    let beacon_address = read_address(storage_str)?;

    // Try implementation() first, then childImplementation()
    let impl_result = json_rpc_request
        .clone()
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": beacon_address, "data": EIP_1967_BEACON_IMPLEMENTATION}),
                json!(block_tag),
            ],
        })
        .await;

    let impl_data = if impl_result.is_ok() {
        impl_result.unwrap()
    } else {
        json_rpc_request
            .call(RequestArguments {
                method: "eth_call".to_string(),
                params: vec![
                    json!({"to": beacon_address, "data": EIP_1967_BEACON_CHILD_IMPLEMENTATION}),
                    json!(block_tag),
                ],
            })
            .await
            .map_err(|e| DetectorError(e.to_string()))?
    };

    let impl_str = impl_data
        .as_str()
        .ok_or(DetectorError("Invalid implementation".to_string()))?;
    let target = read_address(impl_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1967Beacon,
        immutable: false,
    }))
}

// OpenZeppelin Proxy Pattern
async fn detect_open_zeppelin<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let storage = json_rpc_request
        .call(RequestArguments {
            method: "eth_getStorageAt".to_string(),
            params: vec![
                json!(proxy_address),
                json!(OPEN_ZEPPELIN_IMPLEMENTATION_SLOT),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let storage_str = storage.as_str().ok_or(DetectorError("Invalid storage".to_string()))?;
    let target = read_address(storage_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::OpenZeppelin,
        immutable: false,
    }))
}

// EIP-1822 Universal Upgradeable Proxy
async fn detect_eip1822<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let storage = json_rpc_request
        .call(RequestArguments {
            method: "eth_getStorageAt".to_string(),
            params: vec![json!(proxy_address), json!(EIP_1822_LOGIC_SLOT), json!(block_tag)],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let storage_str = storage.as_str().ok_or(DetectorError("Invalid storage".to_string()))?;
    let target = read_address(storage_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1822,
        immutable: false,
    }))
}

// EIP-897 DelegateProxy Pattern
async fn detect_eip897<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let impl_data = json_rpc_request
        .clone()
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": proxy_address, "data": EIP_897_IMPLEMENTATION}),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let impl_str = impl_data
        .as_str()
        .ok_or(DetectorError("Invalid implementation".to_string()))?;
    let target = read_address(impl_str)?;

    // Check if proxy is immutable
    let proxy_type_data = json_rpc_request
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": proxy_address, "data": EIP_897_PROXY_TYPE}),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let proxy_type_str = proxy_type_data.as_str().unwrap_or("");
    let immutable = proxy_type_str == "0x0000000000000000000000000000000000000000000000000000000000000001";

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip897,
        immutable,
    }))
}

// Safe Proxy Contract
async fn detect_safe<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let master_copy_data = json_rpc_request
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![json!({"to": proxy_address, "data": SAFE_MASTER_COPY}), json!(block_tag)],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let master_copy_str = master_copy_data
        .as_str()
        .ok_or(DetectorError("Invalid master copy".to_string()))?;
    let target = read_address(master_copy_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Safe,
        immutable: false,
    }))
}

// Comptroller Proxy
async fn detect_comptroller<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let impl_data = json_rpc_request
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": proxy_address, "data": COMPTROLLER_IMPLEMENTATION}),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let impl_str = impl_data
        .as_str()
        .ok_or(DetectorError("Invalid implementation".to_string()))?;
    let target = read_address(impl_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Comptroller,
        immutable: false,
    }))
}

// Balancer BatchRelayer
async fn detect_batch_relayer<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let version_data = json_rpc_request
        .clone()
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": proxy_address, "data": BATCH_RELAYER_VERSION}),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let version_str = version_data
        .as_str()
        .ok_or(DetectorError("Invalid version".to_string()))?;
    let version_json: serde_json::Value =
        serde_json::from_str(&read_string(version_str).map_err(|e| DetectorError(e.to_string()))?)
            .map_err(|e| DetectorError(e.to_string()))?;

    if version_json["name"].as_str() != Some("BatchRelayer") {
        return Err(DetectorError("Not a BatchRelayer".to_string()));
    }

    let library_data = json_rpc_request
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": proxy_address, "data": BATCH_RELAYER_GET_LIBRARY}),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let library_str = library_data
        .as_str()
        .ok_or(DetectorError("Invalid library".to_string()))?;
    let target = read_address(library_str)?;

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::BatchRelayer,
        immutable: true,
    }))
}

// EIP-2535 Diamond Proxy
async fn detect_eip2535_diamond<F>(
    proxy_address: &str,
    json_rpc_request: F,
    block_tag: &str,
) -> Result<Option<ProxyResult>, DetectorError>
where
    F: JsonRpcRequester,
{
    let facets_data = json_rpc_request
        .call(RequestArguments {
            method: "eth_call".to_string(),
            params: vec![
                json!({"to": proxy_address, "data": EIP_2535_FACET_ADDRESSES}),
                json!(block_tag),
            ],
        })
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let facets_str = facets_data
        .as_str()
        .ok_or(DetectorError("Invalid facets".to_string()))?;
    let target = read_address_array(facets_str)?;

    Ok(Some(ProxyResult::Diamond {
        target,
        proxy_type: ProxyType::Eip2535Diamond,
        immutable: false,
    }))
}

// Helper: read single address from hex string
fn read_address(value: &str) -> Result<String, DetectorError> {
    if value.is_empty() || value == "0x" {
        return Err(DetectorError("Invalid address value".to_string()));
    }

    let mut address = value.to_string();
    if address.len() == 66 {
        address = format!("0x{}", &address[26..]);
    }

    if address == ZERO_ADDRESS {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(address)
}

// Helper: read address array from ABI-encoded data
fn read_address_array(value: &str) -> Result<Vec<String>, DetectorError> {
    if !value.starts_with("0x") {
        return Err(DetectorError("Invalid hex-encoded value".to_string()));
    }

    let hex = &value[2..];
    if hex.len() < 64 {
        return Err(DetectorError("Insufficient data for address[]".to_string()));
    }

    let offset_bytes =
        u64::from_str_radix(&hex[0..64], 16).map_err(|_| DetectorError("Invalid dynamic offset".to_string()))?;
    let offset = (offset_bytes * 2) as usize;

    if hex.len() < offset + 64 {
        return Err(DetectorError("Invalid dynamic offset for address[]".to_string()));
    }

    let length = usize::from_str_radix(&hex[offset..offset + 64], 16)
        .map_err(|_| DetectorError("Invalid address[] length".to_string()))?;

    let mut addresses = Vec::new();
    let mut cursor = offset + 64;
    let needed = cursor + length * 64;

    if hex.len() < needed {
        return Err(DetectorError("Truncated address[] data".to_string()));
    }

    for _ in 0..length {
        let word = &hex[cursor..cursor + 64];
        cursor += 64;
        let address_hex = format!("0x{}", &word[24..]);
        if address_hex != ZERO_ADDRESS {
            addresses.push(address_hex);
        }
    }

    if addresses.is_empty() {
        return Err(DetectorError("Empty address[]".to_string()));
    }

    Ok(addresses)
}

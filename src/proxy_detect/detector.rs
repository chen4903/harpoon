use crate::proxy_detect::{
    eip1167::parse_1167_bytecode,
    read_string::read_string,
    types::{ProxyResult, ProxyType},
};
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::BlockId;
use alloy::rpc::types::TransactionRequest;

// Storage slots for various proxy patterns (as hex strings)
const EIP_1967_LOGIC_SLOT: &str = "360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
const OPEN_ZEPPELIN_IMPLEMENTATION_SLOT: &str = "7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3";
const EIP_1822_LOGIC_SLOT: &str = "c5f16f0fcc639fa48a6947836d9850f504798523bf8c9a3a87d5876cf622bcf7";
const EIP_1967_BEACON_SLOT: &str = "a3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50";

// Function signatures (method IDs) for proxy detection
const EIP_897_IMPLEMENTATION: &str = "5c60da1b";
const EIP_897_PROXY_TYPE: &str = "4555d5c9";
const EIP_1967_BEACON_IMPLEMENTATION: &str = "5c60da1b";
const EIP_1967_BEACON_CHILD_IMPLEMENTATION: &str = "da525716";
const SAFE_MASTER_COPY: &str = "a619486e";
const COMPTROLLER_IMPLEMENTATION: &str = "bb82aa5e";
const BATCH_RELAYER_VERSION: &str = "54fd4d50";
const BATCH_RELAYER_GET_LIBRARY: &str = "7678922e";
const EIP_2535_FACET_ADDRESSES: &str = "52ef6b2c";

#[derive(Debug)]
pub struct DetectorError(String);

impl std::fmt::Display for DetectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DetectorError {}

/// Main proxy detection function using alloy Provider
pub async fn detect_proxy(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: Option<BlockId>,
) -> Option<ProxyResult> {
    let block_id = block_id.unwrap_or(BlockId::latest());

    // Try each detection method sequentially, return first successful one
    if let Ok(Some(result)) = detect_eip1167(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip1967_direct(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip1967_beacon(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_open_zeppelin(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip1822(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip897(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_safe(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_comptroller(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_batch_relayer(provider, proxy_address, block_id).await {
        return Some(result);
    }

    if let Ok(Some(result)) = detect_eip2535_diamond(provider, proxy_address, block_id).await {
        return Some(result);
    }

    None
}

// Helper to create calldata from selector
fn make_calldata(selector: &str) -> Bytes {
    let bytes = hex::decode(selector).unwrap();
    Bytes::from(bytes)
}

// Helper to parse B256 from hex string
fn parse_slot(slot_hex: &str) -> FixedBytes<32> {
    let bytes = hex::decode(slot_hex).unwrap();
    FixedBytes::<32>::from_slice(&bytes)
}

// Helper to extract address from bytes
fn address_from_bytes(bytes: &[u8]) -> Address {
    if bytes.len() >= 20 {
        Address::from_slice(&bytes[bytes.len() - 20..])
    } else {
        Address::ZERO
    }
}

// EIP-1167 Minimal Proxy Contract
async fn detect_eip1167(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let bytecode = provider
        .get_code_at(proxy_address)
        .block_id(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let bytecode_hex = format!("0x{}", hex::encode(&bytecode));
    let target_hex = parse_1167_bytecode(&bytecode_hex).map_err(|_| DetectorError("Not EIP-1167".to_string()))?;

    let target = target_hex
        .parse::<Address>()
        .map_err(|_| DetectorError("Invalid address".to_string()))?;

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1167,
        immutable: true,
    }))
}

// EIP-1967 Direct Proxy
async fn detect_eip1967_direct(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let slot = parse_slot(EIP_1967_LOGIC_SLOT);
    let storage = provider
        .get_storage_at(proxy_address, slot.into())
        .block_id(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = Address::from_word(storage.into());

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1967Direct,
        immutable: false,
    }))
}

// EIP-1967 Beacon Proxy
async fn detect_eip1967_beacon(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let slot = parse_slot(EIP_1967_BEACON_SLOT);
    let storage = provider
        .get_storage_at(proxy_address, slot.into())
        .block_id(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let beacon_address = Address::from_word(storage.into());

    if beacon_address.is_zero() {
        return Err(DetectorError("Empty beacon address".to_string()));
    }

    // Try implementation() first
    let call_data = make_calldata(EIP_1967_BEACON_IMPLEMENTATION);
    let tx_req = alloy::rpc::types::TransactionRequest::default()
        .to(beacon_address)
        .input(call_data.into());

    let impl_result = provider.call(tx_req).await;

    let impl_bytes = if let Ok(bytes) = impl_result {
        bytes
    } else {
        // Try childImplementation()
        let call_data = make_calldata(EIP_1967_BEACON_CHILD_IMPLEMENTATION);
        let tx_req = alloy::rpc::types::TransactionRequest::default()
            .to(beacon_address)
            .input(call_data.into());

        provider.call(tx_req).await.map_err(|e| DetectorError(e.to_string()))?
    };

    let target = address_from_bytes(&impl_bytes);

    if target.is_zero() {
        return Err(DetectorError("Empty implementation address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1967Beacon,
        immutable: false,
    }))
}

// OpenZeppelin Proxy Pattern
async fn detect_open_zeppelin(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let slot = parse_slot(OPEN_ZEPPELIN_IMPLEMENTATION_SLOT);
    let storage = provider
        .get_storage_at(proxy_address, slot.into())
        .block_id(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = Address::from_word(storage.into());

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::OpenZeppelin,
        immutable: false,
    }))
}

// EIP-1822 Universal Upgradeable Proxy
async fn detect_eip1822(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let slot = parse_slot(EIP_1822_LOGIC_SLOT);
    let storage = provider
        .get_storage_at(proxy_address, slot.into())
        .block_id(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = Address::from_word(storage.into());

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip1822,
        immutable: false,
    }))
}

// EIP-897 DelegateProxy Pattern
async fn detect_eip897(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let call_data = make_calldata(EIP_897_IMPLEMENTATION);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let impl_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = address_from_bytes(&impl_bytes);

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    // Check if proxy is immutable
    let call_data = make_calldata(EIP_897_PROXY_TYPE);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let proxy_type_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    // Ensure slice is at most 32 bytes for U256 conversion
    let slice_len = proxy_type_bytes.len().min(32);
    let proxy_type_value = U256::from_be_slice(&proxy_type_bytes[proxy_type_bytes.len() - slice_len..]);
    let immutable = proxy_type_value == U256::from(1);

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Eip897,
        immutable,
    }))
}

// Safe Proxy Contract
async fn detect_safe(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let call_data = make_calldata(SAFE_MASTER_COPY);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let master_copy_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = address_from_bytes(&master_copy_bytes);

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Safe,
        immutable: false,
    }))
}

// Comptroller Proxy
async fn detect_comptroller(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let call_data = make_calldata(COMPTROLLER_IMPLEMENTATION);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let impl_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = address_from_bytes(&impl_bytes);

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::Comptroller,
        immutable: false,
    }))
}

// Balancer BatchRelayer
async fn detect_batch_relayer(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let call_data = make_calldata(BATCH_RELAYER_VERSION);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let version_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let version_hex = format!("0x{}", hex::encode(&version_bytes));
    let version_json: serde_json::Value =
        serde_json::from_str(&read_string(&version_hex).map_err(|e| DetectorError(e.to_string()))?)
            .map_err(|e| DetectorError(e.to_string()))?;

    if version_json["name"].as_str() != Some("BatchRelayer") {
        return Err(DetectorError("Not a BatchRelayer".to_string()));
    }

    let call_data = make_calldata(BATCH_RELAYER_GET_LIBRARY);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let library_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let target = address_from_bytes(&library_bytes);

    if target.is_zero() {
        return Err(DetectorError("Empty address".to_string()));
    }

    Ok(Some(ProxyResult::Single {
        target,
        proxy_type: ProxyType::BatchRelayer,
        immutable: true,
    }))
}

// EIP-2535 Diamond Proxy
async fn detect_eip2535_diamond(
    provider: &dyn Provider,
    proxy_address: Address,
    block_id: BlockId,
) -> Result<Option<ProxyResult>, DetectorError> {
    let call_data = make_calldata(EIP_2535_FACET_ADDRESSES);
    let tx_req = TransactionRequest::default().to(proxy_address).input(call_data.into());
    let facets_bytes = provider
        .call(tx_req)
        .block(block_id)
        .await
        .map_err(|e| DetectorError(e.to_string()))?;

    let facets_hex = format!("0x{}", hex::encode(&facets_bytes));
    let target = read_address_array(&facets_hex)?;

    Ok(Some(ProxyResult::Diamond {
        target,
        proxy_type: ProxyType::Eip2535Diamond,
        immutable: false,
    }))
}

// Helper: read address array from ABI-encoded data
fn read_address_array(value: &str) -> Result<Vec<Address>, DetectorError> {
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
        let address_hex = &word[24..];
        let address = Address::from_slice(&hex::decode(address_hex).unwrap());
        if !address.is_zero() {
            addresses.push(address);
        }
    }

    if addresses.is_empty() {
        return Err(DetectorError("Empty address[]".to_string()));
    }

    Ok(addresses)
}

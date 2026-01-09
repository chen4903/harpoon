# Proxy Detection Module

Detects and extracts implementation addresses from Ethereum proxy contracts using Alloy Provider.

## Supported Proxy Patterns

- **EIP-1167** - Minimal Proxy Contract (immutable)
- **EIP-1967** - Direct & Beacon Proxy
- **EIP-1822** - Universal Upgradeable Proxy (UUPS)
- **EIP-2535** - Diamond Proxy (multi-facet)
- **EIP-897** - DelegateProxy
- **OpenZeppelin** - Legacy proxy pattern
- **Gnosis Safe** - Safe proxy (masterCopy)
- **Compound** - Comptroller proxy
- **Balancer** - BatchRelayer

## Usage

```rust
use harpoon::proxy_detect::{detect_proxy, ProxyResult};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;
use alloy::rpc::types::BlockId;
use std::sync::Arc;

// Create provider
let provider = ProviderBuilder::new()
    .on_http("https://eth.llamarpc.com".parse().unwrap());
let provider: Arc<dyn Provider> = Arc::new(provider);

// Detect proxy
let proxy_address: Address = "0x...".parse().unwrap();
let result = detect_proxy(&*provider, proxy_address, None).await;

match result {
    Some(ProxyResult::Single { target, proxy_type, immutable }) => {
        println!("Implementation: {}", target);
        println!("Type: {:?}, Immutable: {}", proxy_type, immutable);
    }
    Some(ProxyResult::Diamond { target, .. }) => {
        println!("Facets: {:?}", target);
    }
    None => println!("Not a proxy contract"),
}
```

## Features

- Direct integration with Alloy's `Provider` trait
- Zero-copy parsing where possible
- Comprehensive error handling
- Type-safe proxy pattern identification  
- Supports custom block IDs for historical queries

## Testing

```bash
# Run tests (requires network access)
cargo test --test proxy_detect -- --show-output
```

Set `ETH_RPC_URL` environment variable to use a custom RPC endpoint.

## Implementation Note

Currently refactoring from custom JSON-RPC wrapper to native Alloy Provider API for better type safety and ergonomics.

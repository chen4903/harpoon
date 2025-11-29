use alloy::{
    network::{TransactionBuilder, eip2718::Encodable2718},
    primitives::{Address, address},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use harpoon::bloXroute_private_tx::{BloXrouteService, MevBuilder};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Load environment variables
    dotenv::from_filename("examples/.env").ok();

    // Get bloXroute auth header from environment
    let auth_header = env::var("BLOXROUTE_AUTH_HEADER").expect("BLOXROUTE_AUTH_HEADER not set");

    // Get private key from environment
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let signer: PrivateKeySigner = private_key.parse().expect("Invalid private key");

    println!("Signer address: {:#x}", signer.address());

    // Get RPC URL from environment
    let rpc_url = env::var("BSC_MAINNET_HTTP_RPC").unwrap_or_else(|_| "https://binance.llamarpc.com".to_string());

    // Create provider for querying state
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);

    // Create bloXroute service
    let service = BloXrouteService::new(auth_header, vec![signer.clone()], &provider);

    // Example 1: Send a simple transfer transaction with private tx
    println!("=== Example 1: Send a private transfer transaction ===");

    let to_address: Address = address!("0xee343B723F812925D0D17c79D22805b7Fcc8119a");

    // Get current gas price
    let gas_price = provider.get_gas_price().await?;
    println!("Current gas price: {} Gwei", gas_price / 1_000_000_000);

    // Create a simple transfer transaction
    let tx = TransactionRequest::default()
        .from(signer.address())
        .to(to_address)
        .value(alloy::primitives::U256::from(1u128)) // 1 wei
        .gas_limit(21000)
        .gas_price(gas_price);

    println!("Transaction details:");
    println!("  From: {:#x}", signer.address());
    println!("  To: {:#x}", to_address);
    println!("  Value: 1 wei");
    println!("  Gas limit: 21000");

    // Send the transaction with front-running protection using all MEV builders
    match service.send_transaction(tx, Some(vec![MevBuilder::All])).await {
        Ok(tx_hash) => {
            println!("✅ Transaction sent successfully!");
            println!("Transaction hash: {}", tx_hash);
            println!("View on BSCScan: https://bscscan.com/tx/{}", tx_hash);
        }
        Err(e) => {
            println!("❌ Failed to send transaction: {:#}", e);
        }
    }

    // sleep 10 seconds to avoid rate limit
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Example 2: Send a raw transaction
    println!("\n=== Example 2: Send a raw signed transaction ===");

    // Create another transaction
    let tx2 = TransactionRequest::default()
        .from(signer.address())
        .to(to_address)
        .value(alloy::primitives::U256::from(2u128)) // 2 wei
        .gas_limit(21000)
        .gas_price(gas_price);

    // Sign it manually
    let nonce = provider.get_transaction_count(signer.address()).await?;
    let mut tx2 = tx2;
    tx2.set_nonce(nonce + 1);

    let wallet = alloy::network::EthereumWallet::new(signer.clone());
    let signed_tx = tx2.build(&wallet).await?;
    let raw_tx = alloy::hex::encode(signed_tx.encoded_2718());

    println!("Sending raw transaction: 0x{}...", &raw_tx[..16]);

    // Send the raw transaction with bloXroute builder only
    match service.send_private_tx(raw_tx, Some(vec![MevBuilder::All])).await {
        Ok(tx_hash) => {
            println!("✅ Raw transaction sent successfully!");
            println!("Transaction hash: {}", tx_hash);
            println!("View on BSCScan: https://bscscan.com/tx/{}", tx_hash);
        }
        Err(e) => {
            println!("❌ Failed to send raw transaction: {:#}", e);
        }
    }

    println!("\n=== Example completed ===");

    Ok(())
}

/*

2025-11-20 21:46:28  INFO Signer address: 0x015b2e92dc6192cae3b64c96c0ed0984531787c3
2025-11-20 21:46:28  INFO === Example 1: Send a private transfer transaction ===
2025-11-20 21:46:30  INFO Current gas price: 1 Gwei
2025-11-20 21:46:30  INFO Transaction details:
2025-11-20 21:46:30  INFO   From: 0x015b2e92dc6192cae3b64c96c0ed0984531787c3
2025-11-20 21:46:30  INFO   To: 0xee343b723f812925d0d17c79d22805b7fcc8119a
2025-11-20 21:46:30  INFO   Value: 1 wei
2025-11-20 21:46:30  INFO   Gas limit: 21000
2025-11-20 21:46:30  INFO auto-filled nonce address=0x015b2e92dc6192cae3b64c96c0ed0984531787c3 nonce=5
2025-11-20 21:46:30  INFO sending private transaction method="bsc_private_tx" mev_builders=Some([All])
2025-11-20 21:46:30  INFO serialized request request={
  "jsonrpc": "2.0",
  "id": "1",
  "method": "bsc_private_tx",
  "params": {
    "transaction": "f86305843b9aca0082520894ee343b723f812925d0d17c79d22805b7fcc8119a01801ba067f520646af3e9d2ec7bcffe583dca9171585a121d3c0046e53babb1ee856568a06ad51b029e2c7bf4348f5f10c0b23a7a4e135e04c2122fdb8cc00b45b712bc65",
    "mev_builders": [
      "all"
    ]
  }
}
2025-11-20 21:46:31  INFO received bloXroute API response body={"id":"1","result":{"txHash":"1f629ffa4c99f5795356a9fa13d22cacbadaa4b0e5c2d9dd7e39a73861c2399a"},"jsonrpc":"2.0"}

2025-11-20 21:46:31  INFO private transaction submitted successfully tx_hash=1f629ffa4c99f5795356a9fa13d22cacbadaa4b0e5c2d9dd7e39a73861c2399a
2025-11-20 21:46:31  INFO ✅ Transaction sent successfully!
2025-11-20 21:46:31  INFO Transaction hash: 1f629ffa4c99f5795356a9fa13d22cacbadaa4b0e5c2d9dd7e39a73861c2399a
2025-11-20 21:46:31  INFO View on BSCScan: https://bscscan.com/tx/1f629ffa4c99f5795356a9fa13d22cacbadaa4b0e5c2d9dd7e39a73861c2399a
2025-11-20 21:46:41  INFO
=== Example 2: Send a raw signed transaction ===
2025-11-20 21:46:41  INFO Sending raw transaction: 0xf86308843b9aca00...
2025-11-20 21:46:41  INFO sending private transaction method="bsc_private_tx" mev_builders=Some([All])
2025-11-20 21:46:41  INFO serialized request request={
  "jsonrpc": "2.0",
  "id": "1",
  "method": "bsc_private_tx",
  "params": {
    "transaction": "f86308843b9aca0082520894ee343b723f812925d0d17c79d22805b7fcc8119a02801ca006cf081cdf7164cb7a4224c40ac09e4a43a02a342c4f8cf24d9d8ccc38198bffa00f6b883c5435f14fba1034313a88d07c19ebaf3309cae6c34f51c7d0faf28312",
    "mev_builders": [
      "all"
    ]
  }
}
2025-11-20 21:46:41  INFO received bloXroute API response body={"id":"1","result":{"txHash":"85618a2a2b51bb49b436c808dbc6a3af9c643ae70c4d5ed8f9af5265c6bf997b"},"jsonrpc":"2.0"}

2025-11-20 21:46:41  INFO private transaction submitted successfully tx_hash=85618a2a2b51bb49b436c808dbc6a3af9c643ae70c4d5ed8f9af5265c6bf997b
2025-11-20 21:46:41  INFO ✅ Raw transaction sent successfully!
2025-11-20 21:46:41  INFO Transaction hash: 85618a2a2b51bb49b436c808dbc6a3af9c643ae70c4d5ed8f9af5265c6bf997b
2025-11-20 21:46:41  INFO View on BSCScan: https://bscscan.com/tx/85618a2a2b51bb49b436c808dbc6a3af9c643ae70c4d5ed8f9af5265c6bf997b
2025-11-20 21:46:41  INFO
=== Example completed ===

*/

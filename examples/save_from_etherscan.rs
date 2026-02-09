use alloy::primitives::address;
use anyhow::Result;
use harpoon::save_from_etherscan::{etherscan::EtherscanClient, foundry::FoundryProject};
use std::{env, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::from_filename("examples/.env").ok();

    let chain_id = 1;
    let address = address!("0xdac17f958d2ee523a2206206994597c13d831ec7");
    let api_key = env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY not set");
    let output = PathBuf::from("cache/");

    // Create Etherscan client
    let client = EtherscanClient::new(chain_id, api_key)?;

    // Fetch contract source code
    println!("🌐 Fetching source code from Etherscan...");
    let contract_data = client.fetch_contract_info(&address.to_string()).await?;

    // Save contract (one-stop method: creates project, initializes structure, saves all files)
    if contract_data.is_verified {
        let project_path = FoundryProject::save_as_foundry(&output, &contract_data)?;
        println!("📂 Location: {}", project_path.display());
        println!("✨ Done! Contract source code saved successfully.");
    } else {
        println!("❌ Contract is not verified");
    }

    Ok(())
}

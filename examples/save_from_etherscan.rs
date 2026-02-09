use alloy::primitives::address;
use anyhow::Result;
use harpoon::save_from_etherscan::{etherscan::EtherscanClient, foundry::FoundryProject};
use std::{env, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::from_filename("examples/.env").ok();

    let chain_id = 1;
    let address = address!("0x6f288fb580b7969aC4638345FEd4bF4b3F0B2c8D");
    let api_key = env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY not set");
    let output = PathBuf::from("cache/");

    // Create Etherscan client
    let client = EtherscanClient::new(chain_id, api_key)?;

    // Fetch contract source code
    println!("🌐 Fetching source code from Etherscan...");
    let contract_data = client.fetch_contract_info(&address.to_string()).await?;

    // Save contract (one-stop method: creates project, initializes structure, saves all files)
    let project_path = FoundryProject::save_as_foundry(&output, &contract_data)?;

    println!("✨ Done! Contract source code saved successfully.");
    println!("📂 Location: {}", project_path.display());

    Ok(())
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a contract source file from Etherscan
#[derive(Debug, Deserialize, Serialize)]
pub struct SourceFile {
    pub content: String,
}

/// Contract data including sources and metadata
#[derive(Debug)]
pub struct ContractInfo {
    pub is_verified: bool,
    /// Handles both single-file and multi-file contracts
    pub sources: HashMap<String, SourceFile>,
    pub source_code: String,
    pub contract_name: String,
    pub abi: String,
    pub compiler_version: String,
    pub optimization_used: String,
    pub runs: String,
    pub constructor_arguments: String,
    pub evm_version: String,
    pub library: String,
    pub license_type: String,
    pub proxy: String,
    pub implementation: String,
    pub swarm_source: String,
}

/// Etherscan API response for contract source code
#[derive(Debug, Deserialize)]
struct EtherscanResponse {
    status: String,
    message: String,
    result: Vec<ContractResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ContractResult {
    pub source_code: String,
    #[serde(rename = "ABI")]
    pub abi: String,
    pub contract_name: String,
    pub compiler_version: String,
    pub optimization_used: String,
    pub runs: String,
    pub constructor_arguments: String,
    #[serde(rename = "EVMVersion")]
    pub evm_version: String,
    pub library: String,
    pub license_type: String,
    pub proxy: String,
    pub implementation: String,
    pub swarm_source: String,
}

/// Configuration for different chain explorers
pub struct ChainConfig {
    pub api_url: String,
    pub api_key: String,
    pub chain_id: u64,
}

impl ChainConfig {
    /// Get chain configuration based on chain_id
    /// Uses Etherscan API V2 format
    pub fn from_chain_id(chain_id: u64, api_key: String) -> Result<Self> {
        // Validate chain_id is supported
        let supported_chains = [
            1, 11155111, 17000, 560048, // Ethereum & Testnets
            56, 97, // BNB Smart Chain
            137, 80002, // Polygon
            8453, 84532, // Base
            42161, 421614, // Arbitrum
            59144, 59141, // Linea
            81457, 168587773, // Blast
            10, 11155420, // OP Mainnet
            43114, 43113, // Avalanche
            199, 1029, // BitTorrent Chain
            42220, 11142220, // Celo
            252, 2523, // Fraxtal
            100,  // Gnosis
            5000, 5003, // Mantle
            4352, 43521, // Memecore
            1284, 1285, 1287, // Moonbeam/Moonriver
            204, 5611, // opBNB
            534352, 534351, // Scroll
            167000, 167013, // Taiko
            50, 51, // XDC
            33139, 33111, // ApeChain
            480, 4801, // World
            146, 14601, // Sonic
            130, 1301, // Unichain
            2741, 11124, // Abstract
            80094, 80069, // Berachain
            1923, 1924, // Swellchain
            143, 10143, // Monad
            999,   // HyperEVM
            747474, 737373, // Katana
            1329, 1328, // Sei
            988, 2201, // Stable
            9745, 9746, // Plasma
        ];
        if !supported_chains.contains(&chain_id) {
            anyhow::bail!("Unsupported chain_id: {}", chain_id);
        }

        // Etherscan V2 uses a unified API endpoint
        let api_url = "https://api.etherscan.io/v2/api".to_string();

        Ok(Self {
            api_url,
            api_key,
            chain_id,
        })
    }
}

/// Etherscan API client
pub struct EtherscanClient {
    config: ChainConfig,
    client: reqwest::Client,
}

impl EtherscanClient {
    /// Create a new Etherscan client
    pub fn new(chain_id: u64, api_key: String) -> Result<Self> {
        let config = ChainConfig::from_chain_id(chain_id, api_key)?;
        let client = reqwest::Client::new();

        Ok(Self { config, client })
    }

    /// Fetch contract source code from Etherscan V2 API
    /// Returns (is_verified, contract_info)
    pub async fn fetch_contract_info(&self, contract_address: &str) -> Result<(bool, ContractInfo)> {
        let chain_id_str = self.config.chain_id.to_string();

        // Etherscan V2 API parameters
        let params = vec![
            ("chainid", chain_id_str.as_str()),
            ("module", "contract"),
            ("action", "getsourcecode"),
            ("address", contract_address),
            ("apikey", &self.config.api_key),
        ];

        let response = self
            .client
            .get(&self.config.api_url)
            .query(&params)
            .send()
            .await
            .context("Failed to send request to Etherscan")?;

        let etherscan_response: EtherscanResponse =
            response.json().await.context("Failed to parse Etherscan response")?;

        if etherscan_response.status != "1" {
            anyhow::bail!("Etherscan API error: {}", etherscan_response.message);
        }

        let contract_result = etherscan_response.result.first().context("No contract data found")?;

        let sources = self.parse_source_code(&contract_result.source_code, &contract_result.contract_name)?;

        // Check if contract is verified
        let is_verified = !contract_result.source_code.trim().is_empty()
            && contract_result.abi != "Contract source code not verified";

        let contract_info = ContractInfo {
            is_verified,
            sources,
            source_code: contract_result.source_code.clone(),
            contract_name: contract_result.contract_name.clone(),
            abi: contract_result.abi.clone(),
            compiler_version: contract_result.compiler_version.clone(),
            optimization_used: contract_result.optimization_used.clone(),
            runs: contract_result.runs.clone(),
            constructor_arguments: contract_result.constructor_arguments.clone(),
            evm_version: contract_result.evm_version.clone(),
            library: contract_result.library.clone(),
            license_type: contract_result.license_type.clone(),
            proxy: contract_result.proxy.clone(),
            implementation: contract_result.implementation.clone(),
            swarm_source: contract_result.swarm_source.clone(),
        };

        Ok((is_verified, contract_info))
    }

    /// Parse source code from Etherscan response
    /// Handles both single-file and multi-file contracts
    fn parse_source_code(&self, source_code: &str, contract_name: &str) -> Result<HashMap<String, SourceFile>> {
        let mut sources = HashMap::new();

        // Multi-file contract (starts with {{ and ends with }})
        if source_code.starts_with("{{") && source_code.ends_with("}}") {
            let json_str = &source_code[1..source_code.len() - 1];
            let parsed: serde_json::Value =
                serde_json::from_str(json_str).context("Failed to parse multi-file source code")?;

            if let Some(sources_obj) = parsed.get("sources").and_then(|s| s.as_object()) {
                for (file_path, file_data) in sources_obj {
                    if let Some(content) = file_data.get("content").and_then(|c| c.as_str()) {
                        sources.insert(
                            file_path.clone(),
                            SourceFile {
                                content: content.to_string(),
                            },
                        );
                    }
                }
            }
        } else if source_code.starts_with('{') {
            // Try parsing as JSON (alternative multi-file format)
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(source_code) {
                if let Some(sources_obj) = parsed.as_object() {
                    for (file_path, file_data) in sources_obj {
                        if let Some(content) = file_data.get("content").and_then(|c| c.as_str()) {
                            sources.insert(
                                file_path.clone(),
                                SourceFile {
                                    content: content.to_string(),
                                },
                            );
                        }
                    }
                }
            } else {
                // Single file contract
                sources.insert(
                    format!("{}.sol", contract_name),
                    SourceFile {
                        content: source_code.to_string(),
                    },
                );
            }
        } else {
            // Single file contract
            sources.insert(
                format!("{}.sol", contract_name),
                SourceFile {
                    content: source_code.to_string(),
                },
            );
        }

        if sources.is_empty() {
            anyhow::bail!("No source files found in contract data");
        }

        Ok(sources)
    }
}

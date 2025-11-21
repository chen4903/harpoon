use alloy::{
    hex,
    network::{EthereumWallet, Network, TransactionBuilder, eip2718::Encodable2718},
    primitives::Address,
    providers::Provider,
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MEV Builder options for BSC private transactions
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MevBuilder {
    /// bloXroute internal builder
    Bloxroute,
    /// All builders
    All,
    /// 48club builder
    #[serde(rename = "48club")]
    Club48,
    /// BlockRazor builder
    Blockrazor,
    /// JetBldr builder
    Jetbldr,
    /// NodeReal builder
    Nodereal,
}

/// Request parameters for bsc_private_tx
#[derive(Debug, Serialize)]
struct PrivateTxRequest {
    /// Raw transaction bytes without 0x prefix
    transaction: String,
    /// List of MEV builders that should receive the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    mev_builders: Option<Vec<MevBuilder>>,
}

/// JSON-RPC request structure
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: PrivateTxRequest,
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// bloXroute private transaction service
pub struct BloXrouteService {
    /// bloXroute API endpoint
    api_url: String,
    /// Authorization header value
    auth_header: String,
    /// HTTP client
    client: Client,
    /// Signers mapped by their addresses
    signers: HashMap<Address, EthereumWallet>,
}

impl BloXrouteService {
    /// Create a new bloXroute service
    pub fn new(auth_header: String, signers: Vec<PrivateKeySigner>) -> Self {
        let signers: HashMap<_, _> = signers
            .into_iter()
            .map(|s| (s.address(), EthereumWallet::new(s)))
            .collect();

        Self {
            api_url: "https://api.blxrbdn.com".to_string(),
            auth_header,
            client: Client::new(),
            signers,
        }
    }

    /// Create a new bloXroute service with custom API URL
    ///
    /// # Arguments
    /// * `api_url` - Custom bloXroute API endpoint
    /// * `auth_header` - Authorization header value
    /// * `signers` - List of private key signers
    pub fn with_url(api_url: String, auth_header: String, signers: Vec<PrivateKeySigner>) -> Self {
        let signers: HashMap<_, _> = signers
            .into_iter()
            .map(|s| (s.address(), EthereumWallet::new(s)))
            .collect();

        Self {
            api_url,
            auth_header,
            client: Client::new(),
            signers,
        }
    }

    async fn sign_transaction<P, N>(&self, tx: TransactionRequest, provider: &P) -> Result<String>
    where
        P: Provider<N>,
        N: Network,
    {
        let mut tx = tx;

        let account = tx.from.ok_or_else(|| eyre::eyre!("missing sender address"))?;

        let signer = self
            .signers
            .get(&account)
            .ok_or_else(|| eyre::eyre!("missing signer for {:#x}", account))?;

        // Get nonce if not set
        if tx.nonce.is_none() {
            let nonce = provider.get_transaction_count(account).await?;
            tx.set_nonce(nonce);
        }

        // Build and sign transaction
        let signed = tx.build(signer).await?;
        let encoded = signed.encoded_2718();

        // Return hex string without 0x prefix (as required by bloXroute API)
        Ok(hex::encode(&encoded))
    }

    /// Send a private transaction to bloXroute
    ///
    /// # Arguments
    /// * `raw_tx` - Raw transaction hex string (with or without 0x prefix)
    /// * `mev_builders` - Optional list of MEV builders (default: bloxroute builder)
    ///
    /// # Returns
    /// Transaction hash
    pub async fn send_private_tx(&self, raw_tx: String, mev_builders: Option<Vec<MevBuilder>>) -> Result<String> {
        // Remove 0x prefix if present
        let transaction = raw_tx.strip_prefix("0x").unwrap_or(&raw_tx).to_string();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "bsc_private_tx".to_string(),
            params: PrivateTxRequest {
                transaction,
                mev_builders,
            },
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .header("Authorization", &self.auth_header)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(eyre::eyre!("bloXroute API request failed: {}", body));
        }

        let json_response: JsonRpcResponse = serde_json::from_str(&body)?;

        if let Some(error) = json_response.error {
            return Err(eyre::eyre!("bloXroute API error [{}]: {}", error.code, error.message));
        }

        let result = json_response
            .result
            .ok_or_else(|| eyre::eyre!("missing result in response"))?;

        // Parse the result - it could be a string or an object
        let tx_hash = if result.is_string() {
            result.as_str().unwrap().to_string()
        } else if result.is_object() {
            // Try to extract tx_hash from object
            result
                .get("txHash")
                .or_else(|| result.get("tx_hash"))
                .or_else(|| result.get("hash"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| eyre::eyre!("cannot find transaction hash in result object: {:?}", result))?
                .to_string()
        } else {
            return Err(eyre::eyre!("unexpected result type: {:?}", result));
        };

        Ok(tx_hash)
    }

    /// Send a signed transaction with front-running protection
    pub async fn send_transaction<P, N>(
        &self,
        tx: TransactionRequest,
        provider: &P,
        mev_builders: Option<Vec<MevBuilder>>,
    ) -> Result<String>
    where
        P: Provider<N>,
        N: Network,
    {
        let raw_tx = self.sign_transaction(tx, provider).await?;
        self.send_private_tx(raw_tx, mev_builders).await
    }
}

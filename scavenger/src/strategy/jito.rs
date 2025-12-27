// use solana_sdk::pubkey::Pubkey;
use serde::Serialize;
use serde_json::json;
use reqwest::Client;
use std::error::Error;

pub struct JitoClient {
    pub client: Client,
    pub base_url: String,
}

#[derive(Serialize)]
struct BundleRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

impl JitoClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://mainnet.block-engine.jito.wtf/api/v1/bundles".to_string(),
        }
    }

    /// 发送 Jito Bundle (HTTP JSON-RPC)
    /// Docs: https://jito-labs.gitbook.io/mev/searcher-resources/json-rpc-api-reference/bundles/sendbundle
    pub async fn send_bundle(
        &self, 
        txs_base58: Vec<String>, 
        uuid: Option<String>
    ) -> Result<String, Box<dyn Error>> {
        let params = json!([
            txs_base58,
            uuid, // Optional UUID
            [],   // Optional: tipAccounts
            []    // Optional: regions
        ]);

        let req = BundleRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendBundle".to_string(),
            params: vec![params],
        };

        let res = self.client.post(&self.base_url)
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(format!("Jito API Error: {}", res.status()).into());
        }

        let body: serde_json::Value = res.json().await?;
        if let Some(err) = body.get("error") {
             return Err(format!("Jito RPC Error: {:?}", err).into());
        }

        if let Some(result) = body.get("result") {
            if let Some(bundle_id) = result.as_str() {
                return Ok(bundle_id.to_string());
            }
        }

        Ok("Bundle Sent (No ID Returned)".to_string())
    }
}

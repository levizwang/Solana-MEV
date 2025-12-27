use reqwest::Client;
use serde_json::{json, Value};
use log::{info, error};
use std::time::Duration;

pub struct JitoHttpClient {
    client: Client,
    url: String,
}

impl JitoHttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            // Jito Mainnet Block Engine
            url: "https://mainnet.block-engine.jito.wtf/api/v1/bundles".to_string(),
        }
    }

    pub async fn send_bundle(&self, txs_base58: Vec<String>) -> Result<String, String> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [
                txs_base58
            ]
        });

        match self.client.post(&self.url).json(&payload).send().await {
            Ok(resp) => {
                match resp.json::<Value>().await {
                    Ok(json) => {
                        if let Some(result) = json.get("result") {
                            if let Some(uuid) = result.as_str() {
                                return Ok(uuid.to_string());
                            }
                            // Sometimes result is just the bundle ID string directly or inside an object
                            return Ok(result.to_string());
                        } else if let Some(err) = json.get("error") {
                            return Err(format!("Jito Error: {:?}", err));
                        }
                        Ok("Bundle Sent (No ID returned)".to_string())
                    },
                    Err(e) => Err(format!("Failed to parse Jito response: {}", e)),
                }
            },
            Err(e) => Err(format!("HTTP Request Failed: {}", e)),
        }
    }
}

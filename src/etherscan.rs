use crate::models::{chain_id_to_name, ContractData, EtherscanResponse};
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::time::sleep;

pub struct EtherscanClient {
    api_key: String,
    client: reqwest::Client,
}

impl EtherscanClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_contract(
        &self,
        address: &str,
        chain_id: i32,
        protocol: Option<String>,
    ) -> Result<ContractData> {
        // Rate limit: 250ms between requests (Etherscan free tier: 5 req/sec)
        sleep(Duration::from_millis(250)).await;

        let url = format!(
            "https://api.etherscan.io/v2/api?chainid={}&module=contract&action=getsourcecode&address={}&apikey={}",
            chain_id, address, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to Etherscan")?;

        let data: EtherscanResponse = response
            .json()
            .await
            .context("Failed to parse Etherscan response")?;

        if data.status != "1" {
            anyhow::bail!("Etherscan API error: {}", data.message);
        }

        if data.result.is_empty() {
            anyhow::bail!("No contract found at address {}", address);
        }

        let contract = &data.result[0];

        // Detect if proxy
        let is_proxy = !contract.implementation.is_empty() && contract.implementation != "0x";
        let implementation_address = if is_proxy {
            Some(contract.implementation.clone())
        } else {
            None
        };

        // Detect contract type from name
        let contract_type = detect_contract_type(&contract.contract_name);

        Ok(ContractData {
            address: address.to_lowercase(),
            chain: chain_id_to_name(chain_id),
            chain_id,
            name: contract.contract_name.clone(),
            symbol: None, // Will be populated by RPC if needed
            source_code: contract.source_code.clone(),
            abi: contract.abi.clone(),
            is_proxy,
            implementation_address,
            protocol,
            contract_type,
            version: None,
        })
    }
}

fn detect_contract_type(name: &str) -> Option<String> {
    let name_lower = name.to_lowercase();

    if name_lower.contains("proxy") {
        Some("Proxy".to_string())
    } else if name_lower.contains("router") {
        Some("Router".to_string())
    } else if name_lower.contains("factory") {
        Some("Factory".to_string())
    } else if name_lower.contains("pool") {
        Some("Pool".to_string())
    } else if name_lower.contains("vault") {
        Some("Vault".to_string())
    } else if name_lower.contains("token") {
        Some("Token".to_string())
    } else {
        None
    }
}

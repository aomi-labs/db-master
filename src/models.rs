use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractData {
    pub address: String,
    pub chain: String,
    pub chain_id: i32,
    pub name: String,
    pub symbol: Option<String>,
    pub source_code: String,
    pub abi: String,
    pub is_proxy: bool,
    pub implementation_address: Option<String>,
    pub protocol: Option<String>,
    pub contract_type: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub address: String,
    pub chain: String,
    pub chain_id: i32,
    pub name: String,
    pub symbol: Option<String>,
    pub is_proxy: bool,
    pub implementation_address: Option<String>,
    pub protocol: Option<String>,
    pub contract_type: Option<String>,
    pub version: Option<String>,
}

impl From<ContractData> for ContractMetadata {
    fn from(contract: ContractData) -> Self {
        ContractMetadata {
            address: contract.address,
            chain: contract.chain,
            chain_id: contract.chain_id,
            name: contract.name,
            symbol: contract.symbol,
            is_proxy: contract.is_proxy,
            implementation_address: contract.implementation_address,
            protocol: contract.protocol,
            contract_type: contract.contract_type,
            version: contract.version,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub message: String,
    pub result: Vec<EtherscanContract>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EtherscanContract {
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

#[derive(Debug)]
pub struct CuratedAddress {
    pub address: String,
    pub chain_id: i32,
    pub protocol: Option<String>,
}

impl CuratedAddress {
    pub fn from_line(line: &str) -> Option<Self> {
        // Skip comments and empty lines
        let line = line.split('#').next()?.trim();
        if line.is_empty() {
            return None;
        }

        // Parse: address,chain_id,protocol
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            return None;
        }

        Some(CuratedAddress {
            address: parts[0].to_string(),
            chain_id: parts[1].parse().ok()?,
            protocol: parts.get(2).map(|s| s.to_string()),
        })
    }
}

pub fn chain_id_to_name(chain_id: i32) -> String {
    match chain_id {
        1 => "ethereum".to_string(),
        10 => "optimism".to_string(),
        42161 => "arbitrum".to_string(),
        8453 => "base".to_string(),
        137 => "polygon".to_string(),
        _ => format!("chain_{}", chain_id),
    }
}

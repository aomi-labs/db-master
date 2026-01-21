use crate::models::ContractData;
use anyhow::Result;
use csv::{Reader, Writer};
use std::fs::File;
use std::path::Path;

pub fn write_contracts_to_csv(contracts: &[ContractData], output_path: &str) -> Result<()> {
    let mut writer = Writer::from_path(output_path)?;

    for contract in contracts {
        writer.serialize(contract)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn read_contracts_from_csv(input_path: &str) -> Result<Vec<ContractData>> {
    let mut contracts = Vec::new();
    let mut reader = Reader::from_path(input_path)?;

    for result in reader.deserialize() {
        let contract: ContractData = result?;
        contracts.push(contract);
    }

    Ok(contracts)
}

pub fn append_contract_to_csv(contract: &ContractData, output_path: &str) -> Result<()> {
    let file_exists = Path::new(output_path).exists();

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(output_path)?;

    let mut writer = Writer::from_writer(file);

    // Write header if file is new
    if !file_exists {
        writer.serialize(contract)?;
    } else {
        // Skip header for existing files
        writer.serialize(contract)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn csv_exists_with_data(path: &str) -> bool {
    if let Ok(file) = File::open(path) {
        let mut reader = Reader::from_reader(file);
        return reader.records().count() > 0;
    }
    false
}

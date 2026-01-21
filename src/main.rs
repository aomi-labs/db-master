mod csv_handler;
mod db_importer;
mod etherscan;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use models::CuratedAddress;
use std::fs;

#[derive(Parser)]
#[command(name = "contract-csv-tool")]
#[command(about = "Fetch contract data from Etherscan and manage CSV datasets", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch contracts from Etherscan and save to CSV
    Fetch {
        /// Input file with curated addresses
        #[arg(short, long, default_value = "curated-addresses.txt")]
        input: String,

        /// Output CSV file
        #[arg(short, long, default_value = "contracts.csv")]
        output: String,

        /// Etherscan API key (or set ETHERSCAN_API_KEY env var)
        #[arg(short, long)]
        api_key: Option<String>,
    },

    /// Fetch contracts from Etherscan and import directly to database (no CSV)
    FetchToDb {
        /// Input file with curated addresses
        #[arg(short, long, default_value = "curated-addresses.txt")]
        input: String,

        /// Etherscan API key (or set ETHERSCAN_API_KEY env var)
        #[arg(short, long)]
        api_key: Option<String>,

        /// Database URL (or set DATABASE_URL env var)
        #[arg(short, long)]
        database_url: Option<String>,

        /// Batch size for database inserts (default: 50)
        #[arg(short, long, default_value = "50")]
        batch_size: usize,
    },

    /// Import contracts from CSV to database
    Import {
        /// Input CSV file
        #[arg(short, long, default_value = "contracts.csv")]
        input: String,

        /// Database URL (or set DATABASE_URL env var)
        #[arg(short, long)]
        database_url: Option<String>,
    },

    /// Show statistics about CSV file
    Stats {
        /// Input CSV file
        #[arg(short, long, default_value = "contracts.csv")]
        input: String,
    },

    /// Import from metadata CSV by fetching source/ABI from Etherscan
    FetchFromMetadataCsv {
        /// Input metadata CSV file
        #[arg(short, long, default_value = "contracts-metadata.csv")]
        input: String,

        /// Etherscan API key (or set ETHERSCAN_API_KEY env var)
        #[arg(short, long)]
        api_key: Option<String>,

        /// Database URL (or set DATABASE_URL env var)
        #[arg(short, long)]
        database_url: Option<String>,

        /// Batch size for database inserts (default: 50)
        #[arg(short, long, default_value = "50")]
        batch_size: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch { input, output, api_key } => {
            fetch_command(input, output, api_key).await?;
        }
        Commands::FetchToDb { input, api_key, database_url, batch_size } => {
            fetch_to_db_command(input, api_key, database_url, batch_size).await?;
        }
        Commands::Import { input, database_url } => {
            import_command(input, database_url).await?;
        }
        Commands::Stats { input } => {
            stats_command(input)?;
        }
        Commands::FetchFromMetadataCsv { input, api_key, database_url, batch_size } => {
            fetch_from_metadata_csv_command(input, api_key, database_url, batch_size).await?;
        }
    }

    Ok(())
}

async fn fetch_command(input: String, output: String, api_key: Option<String>) -> Result<()> {
    let api_key = api_key
        .or_else(|| std::env::var("ETHERSCAN_API_KEY").ok())
        .expect("ETHERSCAN_API_KEY must be provided via --api-key or environment variable");

    println!("📖 Reading curated addresses from: {}", input);
    let content = fs::read_to_string(&input)?;

    let addresses: Vec<CuratedAddress> = content
        .lines()
        .filter_map(CuratedAddress::from_line)
        .collect();

    println!("✓ Found {} addresses to fetch\n", addresses.len());

    let client = etherscan::EtherscanClient::new(api_key);

    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );

    let mut contracts = Vec::new();

    for addr in addresses {
        pb.set_message(format!("Fetching {}", addr.address));

        match client
            .fetch_contract(&addr.address, addr.chain_id, addr.protocol)
            .await
        {
            Ok(contract) => {
                pb.println(format!(
                    "✓ {} - {}",
                    contract.name, contract.address
                ));
                contracts.push(contract);
            }
            Err(e) => {
                pb.println(format!("✗ {} - Error: {}", addr.address, e));
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done!");

    println!("\n💾 Writing {} contracts to: {}", contracts.len(), output);
    csv_handler::write_contracts_to_csv(&contracts, &output)?;

    println!("✅ Success! {} contracts saved to {}", contracts.len(), output);

    Ok(())
}

async fn fetch_to_db_command(
    input: String,
    api_key: Option<String>,
    database_url: Option<String>,
    batch_size: usize,
) -> Result<()> {
    let api_key = api_key
        .or_else(|| std::env::var("ETHERSCAN_API_KEY").ok())
        .expect("ETHERSCAN_API_KEY must be provided via --api-key or environment variable");

    let database_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be provided via --database-url or environment variable");

    println!("📖 Reading curated addresses from: {}", input);
    let content = fs::read_to_string(&input)?;

    let addresses: Vec<CuratedAddress> = content
        .lines()
        .filter_map(CuratedAddress::from_line)
        .collect();

    println!("✓ Found {} addresses to fetch", addresses.len());
    println!("💾 Fetching and importing directly to database...\n");

    let client = etherscan::EtherscanClient::new(api_key);

    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );

    let mut batch = Vec::new();
    let mut total_imported = 0;

    for addr in addresses {
        pb.set_message(format!("Fetching {}", addr.address));

        match client
            .fetch_contract(&addr.address, addr.chain_id, addr.protocol)
            .await
        {
            Ok(contract) => {
                batch.push(contract);

                // Import batch when it reaches the specified size
                if batch.len() >= batch_size {
                    let imported = db_importer::import_contracts_to_db(&batch, &database_url).await?;
                    total_imported += imported;
                    pb.println(format!("💾 Imported batch of {} contracts", imported));
                    batch.clear();
                }
            }
            Err(e) => {
                pb.println(format!("✗ {} - Error: {}", addr.address, e));
            }
        }

        pb.inc(1);
    }

    // Import remaining contracts
    if !batch.is_empty() {
        let imported = db_importer::import_contracts_to_db(&batch, &database_url).await?;
        total_imported += imported;
        pb.println(format!("💾 Imported final batch of {} contracts", imported));
    }

    pb.finish_with_message("Done!");

    println!("\n✅ Success! Imported {} contracts to database", total_imported);

    Ok(())
}

async fn import_command(input: String, database_url: Option<String>) -> Result<()> {
    let database_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be provided via --database-url or environment variable");

    println!("📖 Reading contracts from: {}", input);
    let contracts = csv_handler::read_contracts_from_csv(&input)?;

    println!("✓ Found {} contracts in CSV", contracts.len());
    println!("💾 Importing to database...\n");

    let imported = db_importer::import_contracts_to_db(&contracts, &database_url).await?;

    println!("\n✅ Success! Imported {} contracts to database", imported);

    Ok(())
}

fn stats_command(input: String) -> Result<()> {
    println!("📊 Reading statistics from: {}", input);
    let contracts = csv_handler::read_contracts_from_csv(&input)?;

    let total = contracts.len();
    let with_symbol = contracts.iter().filter(|c| c.symbol.is_some()).count();
    let proxies = contracts.iter().filter(|c| c.is_proxy).count();
    let with_protocol = contracts.iter().filter(|c| c.protocol.is_some()).count();

    println!("\n📈 Contract Statistics:");
    println!("  Total contracts: {}", total);
    println!("  With symbols:    {}", with_symbol);
    println!("  Proxies:         {}", proxies);
    println!("  With protocol:   {}", with_protocol);

    // Group by protocol
    let mut protocols: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for contract in &contracts {
        if let Some(ref proto) = contract.protocol {
            *protocols.entry(proto.clone()).or_insert(0) += 1;
        }
    }

    if !protocols.is_empty() {
        println!("\n📦 By Protocol:");
        let mut sorted: Vec<_> = protocols.iter().collect();
        sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for (proto, count) in sorted {
            println!("  {}: {}", proto, count);
        }
    }

    // Group by chain
    let mut chains: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
    for contract in &contracts {
        *chains.entry(contract.chain_id).or_insert(0) += 1;
    }

    println!("\n🔗 By Chain:");
    let mut sorted: Vec<_> = chains.iter().collect();
    sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (chain_id, count) in sorted {
        println!("  {} ({}): {}", models::chain_id_to_name(*chain_id), chain_id, count);
    }

    Ok(())
}

async fn fetch_from_metadata_csv_command(
    input: String,
    api_key: Option<String>,
    database_url: Option<String>,
    batch_size: usize,
) -> Result<()> {
    let api_key = api_key
        .or_else(|| std::env::var("ETHERSCAN_API_KEY").ok())
        .expect("ETHERSCAN_API_KEY must be provided via --api-key or environment variable");

    let database_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be provided via --database-url or environment variable");

    println!("📖 Reading metadata CSV from: {}", input);

    // Read CSV file
    let mut rdr = csv::Reader::from_path(&input)?;
    let mut addresses = Vec::new();

    for result in rdr.records() {
        let record = result?;

        // CSV format: address,chain,chain_id,name,symbol,is_proxy,implementation_address,protocol,contract_type,version,created_at,updated_at
        let address = record.get(0).unwrap_or("").to_string();
        let chain_id: i32 = record.get(2).unwrap_or("1").parse().unwrap_or(1);
        let protocol = record.get(7).map(|s| s.to_string());

        if !address.is_empty() && address.starts_with("0x") {
            addresses.push(CuratedAddress {
                address,
                chain_id,
                protocol,
            });
        }
    }

    println!("✓ Found {} addresses to fetch", addresses.len());
    println!("💾 Fetching from Etherscan and importing to database...\n");

    let client = etherscan::EtherscanClient::new(api_key);

    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );

    let mut batch = Vec::new();
    let mut total_imported = 0;

    for addr in addresses {
        pb.set_message(format!("Fetching {}", addr.address));

        match client
            .fetch_contract(&addr.address, addr.chain_id, addr.protocol)
            .await
        {
            Ok(contract) => {
                pb.println(format!("✓ Imported: {} ({})", contract.name, contract.address));
                batch.push(contract);

                // Import batch when it reaches the specified size
                if batch.len() >= batch_size {
                    let imported = db_importer::import_contracts_to_db(&batch, &database_url).await?;
                    total_imported += imported;
                    batch.clear();
                }
            }
            Err(e) => {
                pb.println(format!("✗ {} - Error: {}", addr.address, e));
            }
        }

        pb.inc(1);
    }

    // Import remaining contracts
    if !batch.is_empty() {
        let imported = db_importer::import_contracts_to_db(&batch, &database_url).await?;
        total_imported += imported;
    }

    pb.finish_with_message("Done!");

    println!("\n✅ Success! Imported {} contracts to database", total_imported);

    Ok(())
}

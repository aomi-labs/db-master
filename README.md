# Contract CSV Tool

A Rust CLI tool to fetch smart contract data from Etherscan and manage it as portable CSV datasets.

## Philosophy

**Simple & Portable:** Curate contract addresses → Fetch from Etherscan → Store as CSV → Import anywhere

## Features

- ✅ Fetch contract source code and ABI from Etherscan
- ✅ Store data in portable CSV format
- ✅ Import CSV to PostgreSQL database
- ✅ Version control friendly
- ✅ No flattening - stores original Etherscan format
- ✅ Progress bars and statistics

## Installation

```bash
cd /Users/kevin/foameo/contract-csv-tool
cargo build --release
```

## Setup

1. **Create `.env` file:**
```bash
cp .env.example .env
# Edit .env and add your keys
```

2. **Configure your curated addresses:**

Edit `curated-addresses.txt`:
```
# Format: address,chain_id,protocol

# Stablecoins
0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48,1,Circle    # USDC
0x6b175474e89094c44da98b954eedeac495271d0f,1,MakerDAO  # DAI

# Add more addresses...
```

## Usage

### 1. Fetch Contracts from Etherscan

```bash
# Fetch all curated addresses and save to CSV
cargo run -- fetch

# Custom input/output
cargo run -- fetch --input my-addresses.txt --output my-contracts.csv

# With API key override
cargo run -- fetch --api-key YOUR_KEY_HERE
```

**Output:** `contracts.csv` with all contract data

### 2. Import CSV to Database

```bash
# Import contracts.csv to PostgreSQL
cargo run -- import

# Custom CSV file
cargo run -- import --input my-contracts.csv

# With database URL override
cargo run -- import --database-url "postgresql://user@localhost/db"
```

### 3. View Statistics

```bash
# Show stats about your CSV
cargo run -- stats

# Custom CSV
cargo run -- stats --input my-contracts.csv
```

**Example output:**
```
📊 Reading statistics from: contracts.csv

📈 Contract Statistics:
  Total contracts: 18
  With symbols:    3
  Proxies:         5
  With protocol:   18

📦 By Protocol:
  Uniswap: 7
  Circle: 1
  MakerDAO: 1
  ...

🔗 By Chain:
  ethereum (1): 18
```

## Workflow

```
┌─────────────────────┐
│ curated-addresses   │  ← Manually curate addresses
│  .txt               │
└──────────┬──────────┘
           │
           ▼
    cargo run -- fetch
           │
           ▼
┌─────────────────────┐
│  contracts.csv      │  ← Portable dataset (commit to git!)
└──────────┬──────────┘
           │
           ▼
   cargo run -- import
           │
           ▼
┌─────────────────────┐
│  PostgreSQL DB      │  ← Query with your Rust search API
└─────────────────────┘
```

## CSV Format

```csv
address,chain,chain_id,name,symbol,source_code,abi,is_proxy,implementation_address,protocol,contract_type,version
0xa0b86...eb48,ethereum,1,FiatTokenProxy,USDC,"pragma solidity...","[{...}]",true,0x43506...02dd,Circle,Proxy,
```

## Benefits

### ✅ Portable
- CSV works with any database (PostgreSQL, MySQL, SQLite)
- Share datasets with team/community
- No vendor lock-in

### ✅ Version Controlled
- Track your curated list over time
- Git-friendly format
- Easy to review changes

### ✅ Reproducible
- CSV is your source of truth
- Can recreate database anytime
- Consistent across environments

### ✅ Simple
- No complex dependencies
- Clear separation: fetch → store → import
- Easy to debug

## Commands Reference

### fetch
```bash
cargo run -- fetch [OPTIONS]

OPTIONS:
  -i, --input <FILE>        Input addresses file [default: curated-addresses.txt]
  -o, --output <FILE>       Output CSV file [default: contracts.csv]
  -a, --api-key <KEY>       Etherscan API key (or set ETHERSCAN_API_KEY)
```

### import
```bash
cargo run -- import [OPTIONS]

OPTIONS:
  -i, --input <FILE>            Input CSV file [default: contracts.csv]
  -d, --database-url <URL>      Database URL (or set DATABASE_URL)
```

### stats
```bash
cargo run -- stats [OPTIONS]

OPTIONS:
  -i, --input <FILE>        Input CSV file [default: contracts.csv]
```

## Database Schema

The tool expects this PostgreSQL schema:

```sql
CREATE TABLE contracts (
    address TEXT NOT NULL,
    chain TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    source_code TEXT NOT NULL,
    abi TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT 'Unknown',
    symbol TEXT,
    is_proxy BOOLEAN NOT NULL DEFAULT false,
    implementation_address TEXT,
    protocol TEXT,
    contract_type TEXT,
    version TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (chain_id, address)
);
```

## Environment Variables

```bash
ETHERSCAN_API_KEY=your_key_here       # Get from https://etherscan.io/apis
DATABASE_URL=postgresql://...         # PostgreSQL connection string
```

## Rate Limits

- Etherscan free tier: 5 requests/second
- Tool automatically adds 250ms delay between requests
- Upgrade to premium for faster fetching

## Tips

1. **Start small:** Add a few addresses, test the workflow
2. **Git commit CSV:** Track your dataset over time
3. **Multiple datasets:** Create different address lists for different purposes
4. **Backup:** Keep CSV files - they're your source of truth
5. **Share:** Export CSV and share with team

## Example Workflow

```bash
# 1. Curate addresses
echo "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48,1,Circle" > addresses.txt

# 2. Fetch from Etherscan
cargo run -- fetch --input addresses.txt --output usdc.csv

# 3. Review CSV
cat usdc.csv

# 4. Import to database
cargo run -- import --input usdc.csv

# 5. Commit to git
git add usdc.csv
git commit -m "Add USDC contract"
```

## License

MIT

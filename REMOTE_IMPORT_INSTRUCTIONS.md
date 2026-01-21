# Remote Database Import Instructions

This guide explains how to import contract data to a remote PostgreSQL database using the metadata CSV file.

## What You Need

1. The `contracts-metadata.csv` file (295KB)
2. The compiled binary `contract-csv-tool`
3. An Etherscan API key
4. Database connection URL

## Files to Transfer to Remote Machine

```bash
# Transfer these files to your remote machine:
- target/release/contract-csv-tool  # The compiled binary
- contracts-metadata.csv            # The metadata file
```

## On the Remote Machine

### 1. Make the Binary Executable

```bash
chmod +x contract-csv-tool
```

### 2. Set Environment Variables (Optional)

```bash
export ETHERSCAN_API_KEY="your-etherscan-api-key"
export DATABASE_URL="postgresql://user:password@host:5432/database"
```

### 3. Run the Import

**Using environment variables:**
```bash
./contract-csv-tool fetch-from-metadata-csv \
  --input contracts-metadata.csv
```

**Using command-line arguments:**
```bash
./contract-csv-tool fetch-from-metadata-csv \
  --input contracts-metadata.csv \
  --api-key "your-etherscan-api-key" \
  --database-url "postgresql://user:password@host:5432/database"
```

**With custom batch size:**
```bash
./contract-csv-tool fetch-from-metadata-csv \
  --input contracts-metadata.csv \
  --batch-size 100
```

## How It Works

1. Reads the metadata CSV file (2,394 contracts)
2. For each contract:
   - Fetches full source code and ABI from Etherscan API
   - Rate-limited to 5 requests/second (250ms delay)
3. Imports contracts in batches (default: 50 at a time)
4. Displays progress with a progress bar

## Expected Time

- **2,394 contracts** × **0.25 seconds** = ~10 minutes
- Plus database insert time

## Database Schema

The tool expects a PostgreSQL table named `contracts` with this schema:

```sql
CREATE TABLE IF NOT EXISTS contracts (
    address VARCHAR(42) PRIMARY KEY,
    chain VARCHAR(50) NOT NULL,
    chain_id INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL,
    symbol VARCHAR(50),
    source_code TEXT NOT NULL,
    abi TEXT NOT NULL,
    is_proxy BOOLEAN NOT NULL DEFAULT FALSE,
    implementation_address VARCHAR(42),
    protocol VARCHAR(100),
    contract_type VARCHAR(100),
    version VARCHAR(50),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_contracts_chain_id ON contracts(chain_id);
CREATE INDEX IF NOT EXISTS idx_contracts_protocol ON contracts(protocol);
CREATE INDEX IF NOT EXISTS idx_contracts_is_proxy ON contracts(is_proxy);
```

## Troubleshooting

### Binary Won't Run
```bash
# Check architecture
file contract-csv-tool

# If it's the wrong architecture, you'll need to cross-compile
# or compile directly on the remote machine
```

### Compile on Remote Machine
If the binary doesn't work, compile directly on the remote machine:

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Transfer just the source code
scp -r /path/to/contract-csv-tool remote:/path/to/

# On remote machine
cd contract-csv-tool
cargo build --release
./target/release/contract-csv-tool fetch-from-metadata-csv --input contracts-metadata.csv
```

### Etherscan Rate Limiting
If you hit rate limits:
- The tool already includes 250ms delay between requests
- Use an Etherscan Pro API key for higher limits
- Reduce batch size if needed

### Database Connection Issues
```bash
# Test connection first
psql "postgresql://user:password@host:5432/database" -c "SELECT version();"
```

## Example Output

```
📖 Reading metadata CSV from: contracts-metadata.csv
✓ Found 2394 addresses to fetch
💾 Fetching from Etherscan and importing to database...

[00:02:15] ████████████░░░░░░░░░░░░░░░░ 542/2394 Fetching 0x...
✓ Imported: UniswapV2Router02 (0x7a250d5630b4cf539739df2c5dacb4c659f2488d)
✓ Imported: WETH9 (0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2)
...

✅ Success! Imported 2394 contracts to database
```

## Notes

- The CSV contains metadata only (no source/ABI)
- Source code and ABI are fetched fresh from Etherscan
- This ensures you have the latest verified contract data
- Failed contracts are logged but don't stop the process

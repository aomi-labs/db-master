#!/bin/bash
# Seed database with contract data (idempotent)
# Usage: DATABASE_URL=... ./scripts/seed.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

if [ -z "${DATABASE_URL:-}" ]; then
    echo "❌ ERROR: DATABASE_URL is required"
    exit 1
fi

echo "🌱 Seeding database..."
echo "   Target: ${DATABASE_URL%%@*}@****"

# Check if contracts.csv exists
if [ ! -f "contracts.csv" ]; then
    echo "⚠️  No contracts.csv found, skipping contract import"
    exit 0
fi

# Count existing contracts
EXISTING=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM contracts;" 2>/dev/null | tr -d ' ')
echo "   📊 Existing contracts: $EXISTING"

echo "   📥 Importing from contracts.csv..."

# All in one psql session using heredoc with \copy
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'EOSQL'
-- Create staging table (regular table, not temp, so \copy works)
DROP TABLE IF EXISTS _contracts_staging;
CREATE TABLE _contracts_staging (
    address TEXT,
    chain TEXT,
    chain_id INTEGER,
    name TEXT,
    symbol TEXT,
    source_code TEXT,
    abi TEXT,
    is_proxy TEXT,
    implementation_address TEXT,
    protocol TEXT,
    contract_type TEXT,
    version TEXT
);
EOSQL

# Import CSV using \copy (must be separate command)
psql "$DATABASE_URL" -c "\COPY _contracts_staging(address,chain,chain_id,name,symbol,source_code,abi,is_proxy,implementation_address,protocol,contract_type,version) FROM 'contracts.csv' WITH (FORMAT csv, HEADER true);"

# Upsert and cleanup
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'EOSQL'
-- Upsert into main table (idempotent)
INSERT INTO contracts (
    address, chain, chain_id, name, symbol, source_code, abi, 
    is_proxy, implementation_address, protocol, contract_type, version
)
SELECT 
    address, chain, chain_id, 
    COALESCE(name, 'Unknown'),
    symbol,
    COALESCE(source_code, ''),
    COALESCE(abi, '[]'),
    COALESCE(is_proxy, 'false')::BOOLEAN,
    implementation_address,
    protocol,
    contract_type,
    version
FROM _contracts_staging
WHERE address IS NOT NULL AND chain_id IS NOT NULL
ON CONFLICT (chain_id, address) DO UPDATE SET
    name = COALESCE(EXCLUDED.name, contracts.name),
    symbol = COALESCE(EXCLUDED.symbol, contracts.symbol),
    source_code = COALESCE(EXCLUDED.source_code, contracts.source_code),
    abi = COALESCE(EXCLUDED.abi, contracts.abi),
    is_proxy = COALESCE(EXCLUDED.is_proxy, contracts.is_proxy),
    implementation_address = COALESCE(EXCLUDED.implementation_address, contracts.implementation_address),
    protocol = COALESCE(EXCLUDED.protocol, contracts.protocol),
    contract_type = COALESCE(EXCLUDED.contract_type, contracts.contract_type),
    version = COALESCE(EXCLUDED.version, contracts.version),
    updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT;

-- Cleanup staging table
DROP TABLE _contracts_staging;
EOSQL

# Count after import
AFTER=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM contracts;" 2>/dev/null | tr -d ' ')
echo "   📊 Contracts after seed: $AFTER"

echo "✅ Seeding complete!"

# Aomi Database Master

Central repository for Aomi database schema, migrations, and seed data.

## Overview

This repo is the **source of truth** for the Aomi database schema. It manages:

- **Migrations** - Schema changes applied to the database
- **Seeds** - Contract data and other seed data
- **Scripts** - Helper scripts for DB operations

## Database

**Production**: DigitalOcean Managed PostgreSQL (SFO3)
- VPC endpoint: `private-aomi-prod-db-postgresql-sfo3-*.e.db.ondigitalocean.com:25060`
- Public endpoint: `aomi-prod-db-postgresql-sfo3-*.e.db.ondigitalocean.com:25060`

## Quick Start

### Manual Migration

```bash
# Set database URL
export DATABASE_URL="postgresql://user:pass@host:port/dbname?sslmode=require"

# Run migrations
./scripts/migrate.sh

# Seed contract data
./scripts/seed.sh
```

### Automatic Deployment

Pushing to `main` triggers GitHub Actions to:
1. Run migrations
2. Seed contract data

## Schema

### Tables

| Table | Description |
|-------|-------------|
| `contracts` | Smart contract metadata and ABIs |
| `transaction_records` | Transaction fetch tracking |
| `transactions` | Individual blockchain transactions |
| `users` | User accounts (wallet public_key) |
| `sessions` | Chat sessions |
| `messages` | Chat and agent messages |
| `api_keys` | API key → namespace access |
| `signup_challenges` | Wallet signature challenges |

### Entity Relationships

```
users (public_key) ──┐
                     ├──> sessions ──> messages
api_keys ────────────┘         │
                               └──> signup_challenges

contracts ──> transaction_records ──> transactions
```

## Contract Data Tools

The original contract CSV tool is preserved for fetching contract data from Etherscan:

```bash
# Fetch contracts from Etherscan
cargo run -- fetch

# Import to database (done automatically via seed.sh)
cargo run -- import --database-url "$DATABASE_URL"
```

## Directory Structure

```
db-master/
├── .github/workflows/  # CI/CD
│   └── db-deploy.yml   # Auto-deploy on push to main
├── migrations/         # SQL migrations (applied in order)
│   └── 001_initial_schema.sql
├── scripts/            # Helper scripts
│   ├── migrate.sh      # Run migrations (idempotent)
│   └── seed.sh         # Seed data (idempotent)
├── src/                # Rust contract fetcher
├── contracts.csv       # Contract seed data
└── README.md
```

## GitHub Secrets Required

| Secret | Description |
|--------|-------------|
| `DATABASE_URL` | PostgreSQL connection string (public endpoint with SSL) |

Example:
```
postgresql://doadmin:PASSWORD@aomi-prod-db-postgresql-sfo3-do-user-XXXXX-0.e.db.ondigitalocean.com:25060/defaultdb?sslmode=require
```

## Adding Migrations

1. Create a new file in `migrations/` with format `NNN_description.sql`
2. Write idempotent SQL (use `IF NOT EXISTS`, `ON CONFLICT`, etc.)
3. Push to `main` - GitHub Actions will apply it

## Related Repos

- [product-mono](https://github.com/aomi-labs/product-mono) - Main application (uses this schema)

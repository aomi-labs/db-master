use crate::models::ContractData;
use anyhow::Result;
use sqlx::postgres::PgPool;

pub async fn import_contracts_to_db(
    contracts: &[ContractData],
    database_url: &str,
) -> Result<usize> {
    let pool = PgPool::connect(database_url).await?;

    let mut imported = 0;

    for contract in contracts {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO contracts (
                address, chain, chain_id, source_code, abi, name, symbol,
                is_proxy, implementation_address, protocol, contract_type, version,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (chain_id, address) DO UPDATE SET
                source_code = EXCLUDED.source_code,
                abi = EXCLUDED.abi,
                name = EXCLUDED.name,
                symbol = EXCLUDED.symbol,
                is_proxy = EXCLUDED.is_proxy,
                implementation_address = EXCLUDED.implementation_address,
                protocol = EXCLUDED.protocol,
                contract_type = EXCLUDED.contract_type,
                version = EXCLUDED.version,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&contract.address)
        .bind(&contract.chain)
        .bind(contract.chain_id)
        .bind(&contract.source_code)
        .bind(&contract.abi)
        .bind(&contract.name)
        .bind(&contract.symbol)
        .bind(contract.is_proxy)
        .bind(&contract.implementation_address)
        .bind(&contract.protocol)
        .bind(&contract.contract_type)
        .bind(&contract.version)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        match result {
            Ok(_) => {
                println!("✓ Imported: {} ({})", contract.name, contract.address);
                imported += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to import {}: {}", contract.address, e);
            }
        }
    }

    pool.close().await;

    Ok(imported)
}

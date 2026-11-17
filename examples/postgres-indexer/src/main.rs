use std::sync::Arc;

use shipstern::{bs58, config::ShipsternConfig, Handler, HandlerResult, Pipeline, Runtime};
use shipstern_core::instruction::InstructionUpdate;
use shipstern_spl_token_parser::{instruction::Instruction, InstructionParser, TokenProgram};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};
use tokio_postgres::{Client, NoTls};

/// How far behind the checkpoint to resume after a restart. Updates finish out
/// of order across the worker pool, so slots just below the checkpoint may not
/// be written yet. Re-reading them is safe because every write is an upsert.
const RESUME_MARGIN_SLOTS: i64 = 300;

/// Writes SPL Token transfers to Postgres. Safe to run twice on the same update.
#[derive(Debug)]
struct TransfersToPostgres {
    db: Arc<Client>,
}

impl Handler<TokenProgram, InstructionUpdate> for TransfersToPostgres {
    async fn handle(&self, value: &TokenProgram, raw: &InstructionUpdate) -> HandlerResult<()> {
        let (source, destination, amount) = match value.instruction.as_ref() {
            Some(Instruction::Transfer(t)) => (t.accounts.source, t.accounts.destination, t.args.amount),
            Some(Instruction::TransferChecked(t)) => {
                (t.accounts.source, t.accounts.destination, t.args.amount)
            },
            _ => return Ok(()),
        };

        let signature = bs58::encode(&raw.shared.signature).into_string();
        let ix_path = raw
            .path
            .as_slice()
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(".");
        let slot = raw.shared.slot as i64;

        // Same row on a repeat delivery: the primary key turns it into a no-op.
        self.db
            .execute(
                "INSERT INTO token_transfers (signature, ix_path, slot, source, destination, amount)
                 VALUES ($1, $2, $3, $4, $5, $6::text::numeric)
                 ON CONFLICT (signature, ix_path) DO NOTHING",
                &[
                    &signature,
                    &ix_path,
                    &slot,
                    &source.to_string(),
                    &destination.to_string(),
                    &amount.to_string(),
                ],
            )
            .await?;

        // Only ever moves forward, whatever order updates finish in.
        self.db
            .execute(
                "INSERT INTO indexer_checkpoint (id, slot) VALUES (1, $1)
                 ON CONFLICT (id) DO UPDATE SET slot = GREATEST(indexer_checkpoint.slot, EXCLUDED.slot)",
                &[&slot],
            )
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    // The connection object does the I/O; it must run for the client to work.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(%e, "Postgres connection closed");
        }
    });

    client.batch_execute(include_str!("../schema.sql")).await?;

    let config = std::fs::read_to_string("Shipstern.toml")?;
    // Only the message: the error's Debug form holds the whole file, x-token included.
    let mut config: ShipsternConfig<YellowstoneGrpcConfig> =
        toml::from_str(&config).map_err(|e| format!("Error parsing config: {}", e.message()))?;

    let checkpoint = client
        .query_opt("SELECT slot FROM indexer_checkpoint WHERE id = 1", &[])
        .await?
        .map(|row| row.get::<_, i64>(0));

    if let Some(slot) = checkpoint {
        let from = (slot - RESUME_MARGIN_SLOTS).max(0) as u64;
        tracing::info!(checkpoint = slot, from, "Resuming");
        config.source.from_slot = Some(from);
    }

    let handler = TransfersToPostgres { db: Arc::new(client) };

    Runtime::builder()
        .instruction(Pipeline::new(InstructionParser, [handler]))
        .build::<YellowstoneGrpcSource>(config)
        .run_async()
        .await;

    Ok(())
}

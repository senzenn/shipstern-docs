use shipstern::{bs58, Handler, HandlerResult, Pipeline};
use shipstern_core::{instruction::InstructionUpdate, AccountUpdate};
use shipstern_spl_token_parser::{instruction::Instruction, InstructionParser, TokenProgram};
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;

// #region logger
#[derive(Debug)]
pub struct Logger;

impl<V: std::fmt::Debug + Sync, R: Sync> shipstern::Handler<V, R> for Logger {
    async fn handle(&self, value: &V, _raw: &R) -> shipstern::HandlerResult<()> {
        tracing::info!(?value);
        Ok(())
    }
}
// #endregion logger

// #region token-transfers
#[derive(Debug)]
pub struct TokenTransfers;

impl Handler<TokenProgram, InstructionUpdate> for TokenTransfers {
    async fn handle(&self, value: &TokenProgram, _raw: &InstructionUpdate) -> HandlerResult<()> {
        let Some(ix) = value.instruction.as_ref() else {
            return Ok(());
        };

        match ix {
            Instruction::Transfer(t) => {
                tracing::info!(
                    source = %t.accounts.source,
                    destination = %t.accounts.destination,
                    amount = t.args.amount,
                    "transfer"
                );
            },
            Instruction::TransferChecked(t) => {
                tracing::info!(
                    mint = %t.accounts.mint,
                    amount = t.args.amount,
                    decimals = t.args.decimals,
                    "transfer checked"
                );
            },
            Instruction::MintTo(t) => {
                tracing::info!(mint = %t.accounts.mint, amount = t.args.amount, "mint to");
            },
            _ => {},
        }

        Ok(())
    }
}
// #endregion token-transfers

// #region instruction-context
#[derive(Debug)]
pub struct WithContext;

impl<V: std::fmt::Debug + Sync> Handler<V, InstructionUpdate> for WithContext {
    async fn handle(&self, value: &V, raw: &InstructionUpdate) -> HandlerResult<()> {
        let signature = bs58::encode(&raw.shared.signature).into_string();

        tracing::info!(slot = raw.shared.slot, %signature, path = ?raw.path, ?value);
        Ok(())
    }
}
// #endregion instruction-context

// #region account-context
#[derive(Debug)]
pub struct AccountContext;

impl<V: std::fmt::Debug + Sync> Handler<V, AccountUpdate> for AccountContext {
    async fn handle(&self, value: &V, raw: &AccountUpdate) -> HandlerResult<()> {
        let Some(info) = raw.account.as_ref() else {
            return Ok(());
        };

        let pubkey = bs58::encode(&info.pubkey).into_string();
        tracing::info!(slot = raw.slot, %pubkey, lamports = info.lamports, ?value);
        Ok(())
    }
}
// #endregion account-context

// #region forward
use std::time::Duration;

use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Forward(mpsc::Sender<String>);

impl<V: std::fmt::Debug + Sync, R: Sync> shipstern::Handler<V, R> for Forward {
    async fn handle(&self, value: &V, _raw: &R) -> shipstern::HandlerResult<()> {
        // Waits when the channel is full, which slows the pipeline down
        // instead of buffering without limit.
        self.0.send(format!("{value:?}")).await?;
        Ok(())
    }
}

pub async fn run_with_writer() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let config = std::fs::read_to_string("Shipstern.toml").expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    let (tx, mut rx) = mpsc::channel(1024);

    let writer = tokio::spawn(async move {
        while let Some(row) = rx.recv().await {
            // Database writes here keep storage I/O outside handler tasks.
            println!("{row}");
        }
    });

    shipstern::Runtime::builder()
        .instruction(Pipeline::new(InstructionParser, [Forward(tx)]))
        .build::<YellowstoneGrpcSource>(config)
        .run_async()
        .await;

    // The pipeline held the only sender, so the writer stops once the queue is empty.
    // Jobs abandoned by the drain timeout still hold it, so bound the wait.
    if tokio::time::timeout(Duration::from_secs(30), writer).await.is_err() {
        tracing::warn!("Writer did not drain within 30 seconds");
    }
}
// #endregion forward

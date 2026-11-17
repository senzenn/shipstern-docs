use std::str::FromStr as _;

use shipstern::{filter_pipeline::FilterPipeline, Handler, HandlerResult};
use shipstern_core::{instruction::InstructionUpdate, KeyBytes, Prefilter};
use shipstern_preconf_source::PreconfSource;
use shipstern_spl_token_parser::{InstructionParser, TokenProgram};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// The Orca Whirlpool program. A required key keeps the stream under the coverage share,
/// which the whole SPL Token program reaches within a minute.
const WHIRLPOOL: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Prints each SPL Token instruction of a preconf with its slot, arrival index and signature.
#[derive(Debug)]
struct PreconfLogger;

impl Handler<TokenProgram, InstructionUpdate> for PreconfLogger {
    async fn handle(&self, value: &TokenProgram, raw: &InstructionUpdate) -> HandlerResult<()> {
        let shared = &raw.shared;
        let signature = shipstern_core::bs58::encode(&shared.signature).into_string();
        println!(
            "slot={} index={} signature={signature} {value:?}",
            shared.slot, shared.txn_index
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = std::fs::read_to_string("Shipstern.toml").expect("Error reading config file");
    // The error Debug holds the whole file, x-token included, so print only its message.
    let config =
        toml::from_str(&config).unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    let whirlpool = KeyBytes::<32>::from_str(WHIRLPOOL).expect("Error parsing the Whirlpool key");

    shipstern::Runtime::builder()
        .instruction(FilterPipeline::new(
            InstructionParser,
            [PreconfLogger],
            Prefilter::builder().transaction_accounts([whirlpool]),
        ))
        .build::<PreconfSource>(config)
        .run_async()
        .await;
}

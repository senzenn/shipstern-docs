use std::path::PathBuf;

use clap::Parser;
use shipstern::filter_pipeline::FilterPipeline;
use shipstern_core::{AccountUpdate, CuckooAccounts, Prefilter, Pubkey};
use shipstern_spl_token_parser::AccountParser;
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,

    /// A file with one base58 account address per line.
    #[arg(long, short)]
    accounts: PathBuf,
}

/// Keeps the updates for accounts in the set and drops the rest.
#[derive(Debug)]
pub struct InSet(CuckooAccounts);

impl<V: std::fmt::Debug + Sync> shipstern::Handler<V, AccountUpdate> for InSet {
    async fn handle(&self, value: &V, input: &AccountUpdate) -> shipstern::HandlerResult<()> {
        let Some(key) = input
            .account
            .as_ref()
            .and_then(|account| Pubkey::try_from(account.pubkey.as_slice()).ok())
        else {
            return Ok(());
        };

        // The server can match an account outside the set.
        if !self.0.contains(&key) {
            eprintln!("{key} false match, dropped");
            return Ok(());
        }

        println!("{key} slot {} {value:?}", input.slot);
        Ok(())
    }
}

fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config, accounts } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    let accounts = std::fs::read_to_string(accounts).expect("Error reading accounts file");
    let accounts: CuckooAccounts = accounts
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.parse::<Pubkey>()
                .unwrap_or_else(|err| panic!("Invalid account address {line}: {err}"))
        })
        .collect();

    let prefilter = Prefilter::builder().account_cuckoo(accounts.iter().copied());

    shipstern::Runtime::builder()
        .account(FilterPipeline::new(
            AccountParser,
            [InSet(accounts)],
            prefilter,
        ))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}

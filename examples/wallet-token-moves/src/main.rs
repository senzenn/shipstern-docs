use std::path::PathBuf;

use clap::Parser;
use shipstern::{bs58, filter_pipeline::FilterPipeline, Handler, HandlerResult};
use shipstern_core::{instruction::InstructionUpdate, Prefilter, Pubkey, TokenAccounts};
use shipstern_spl_token_parser::{instruction::Instruction, InstructionParser, TokenProgram};
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,

    /// The wallet whose token moves to print.
    #[arg(long, short)]
    wallet: Pubkey,
}

/// Prints SPL Token transfers into and out of one wallet's token accounts.
#[derive(Debug)]
pub struct WalletMoves {
    wallet: String,
}

/// Owner and mint of a token account, read from the transaction's token balances.
fn owner_and_mint(raw: &InstructionUpdate, token_account: Pubkey) -> Option<(&str, &str)> {
    let shared = &raw.shared;

    shared
        .post_token_balances
        .iter()
        .chain(&shared.pre_token_balances)
        .find(|b| {
            shared
                .accounts
                .get(b.account_index)
                .is_ok_and(|key| key == token_account)
        })
        .map(|b| (b.owner.as_str(), b.mint.as_str()))
}

impl Handler<TokenProgram, InstructionUpdate> for WalletMoves {
    async fn handle(&self, value: &TokenProgram, raw: &InstructionUpdate) -> HandlerResult<()> {
        let (source, destination, amount) = match value.instruction.as_ref() {
            Some(Instruction::Transfer(t)) => {
                (t.accounts.source, t.accounts.destination, t.args.amount)
            }
            Some(Instruction::TransferChecked(t)) => {
                (t.accounts.source, t.accounts.destination, t.args.amount)
            }
            _ => return Ok(()),
        };

        let source = owner_and_mint(raw, source);
        let destination = owner_and_mint(raw, destination);
        let owned =
            |side: Option<(&str, &str)>| side.is_some_and(|(owner, _)| owner == self.wallet);

        // Other transfers in the same transaction move someone else's tokens.
        let direction = match (owned(source), owned(destination)) {
            (true, true) => "self",
            (true, false) => "out",
            (false, true) => "in",
            (false, false) => return Ok(()),
        };

        let mint = source
            .or(destination)
            .map_or("unknown mint", |(_, mint)| mint);
        let signature = bs58::encode(&raw.shared.signature).into_string();
        println!("{direction} {amount} of {mint} in {signature}");

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

    let Opts { config, wallet } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    let handler = WalletMoves {
        wallet: wallet.to_string(),
    };

    shipstern::Runtime::builder()
        .instruction(FilterPipeline::new(
            InstructionParser,
            [handler],
            Prefilter::builder()
                .transaction_accounts_include([wallet])
                .transaction_token_accounts(TokenAccounts::BalanceChanged),
        ))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}

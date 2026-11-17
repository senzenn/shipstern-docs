use std::{borrow::Cow, path::PathBuf};

use clap::Parser as _;
use shipstern_core::{
    bs58, AccountUpdate, ParseError, ParseResult, Parser, Prefilter, ProgramParser, Pubkey,
};
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// #region message
/// What the server sends to clients. The stream server needs a protobuf
/// message with a type name (`prost::Name`), because it wraps every value in
/// `google.protobuf.Any`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct TokenAccountUpdate {
    #[prost(string, tag = "1")]
    pub pubkey: String,
    #[prost(uint64, tag = "2")]
    pub slot: u64,
    #[prost(uint64, tag = "3")]
    pub lamports: u64,
    #[prost(uint64, tag = "4")]
    pub data_len: u64,
}

impl prost::Name for TokenAccountUpdate {
    const NAME: &'static str = "TokenAccountUpdate";
    const PACKAGE: &'static str = "my_indexer";
}
// #endregion message

// #region parser
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// A parser for accounts owned by the Token Program. The stream server keys
/// subscriptions by `program_id()`, so it must be a `ProgramParser`.
#[derive(Debug, Clone, Copy)]
pub struct TokenAccounts;

impl Parser for TokenAccounts {
    type Input = AccountUpdate;
    type Output = TokenAccountUpdate;

    fn id(&self) -> Cow<'static, str> { "my_indexer::TokenAccounts".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([self.program_id()])
            .build()
            .unwrap()
    }

    async fn parse(&self, update: &AccountUpdate) -> ParseResult<Self::Output> {
        let Some(info) = update.account.as_ref() else {
            return Err(ParseError::Filtered);
        };

        Ok(TokenAccountUpdate {
            pubkey: bs58::encode(&info.pubkey).into_string(),
            slot: update.slot,
            lamports: info.lamports,
            data_len: info.data.len() as u64,
        })
    }
}

impl ProgramParser for TokenAccounts {
    fn program_id(&self) -> Pubkey { TOKEN_PROGRAM.parse().unwrap() }
}
// #endregion parser

#[derive(clap::Parser)]
#[command(version, about = "Serve Token Program account updates over gRPC")]
struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

// #region server
fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    shipstern_stream::Server::builder()
        .account(TokenAccounts)
        .build::<YellowstoneGrpcSource>(config)
        .run();
}
// #endregion server

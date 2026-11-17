use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    future::Future,
    path::PathBuf,
};

use clap::Parser as _;
use shipstern::{
    bs58,
    config::ShipsternConfig,
    shipstern_core::{
        instruction::InstructionUpdate, ParseResult, Parser, Prefilter, ProgramParser,
        TransactionPrefilter,
    },
    Handler, HandlerResult, Pipeline, Runtime,
};
use shipstern_spl_token_extensions_parser::{
    InstructionParser as Token2022Ix, TokenExtensionProgram,
};
use shipstern_spl_token_parser::{InstructionParser as TokenIx, TokenProgram};
use shipstern_stake_pool_parser::{InstructionParser as StakePoolIx, StakePoolProgram};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

/// Puts the parser's program in the include set, not the required set. The runtime
/// merges the required sets of all instruction parsers, which requires every program.
#[derive(Debug, Clone, Copy)]
pub struct IncludeProgram<P>(P);

impl<P: ProgramParser<Input = InstructionUpdate>> Parser for IncludeProgram<P> {
    type Input = InstructionUpdate;
    type Output = P::Output;

    // All instruction parsers share one filter entry, so this ID selects no filter.
    fn id(&self) -> Cow<'static, str> {
        self.0.id()
    }

    fn prefilter(&self) -> Prefilter {
        Prefilter {
            transaction: Some(TransactionPrefilter {
                accounts_include: [self.0.program_id()].into(),
                ..TransactionPrefilter::default()
            }),
            ..Prefilter::default()
        }
    }

    fn parse(&self, ix: &InstructionUpdate) -> impl Future<Output = ParseResult<P::Output>> + Send {
        // The inner parser returns `Filtered` for the instructions of other programs.
        self.0.parse(ix)
    }
}

fn print(program: &str, raw: &InstructionUpdate, value: &impl Debug) {
    let signature = bs58::encode(&raw.shared.signature).into_string();
    println!("{program} {signature} ix {:?} - {value:?}", raw.path);
}

#[derive(Debug)]
pub struct TokenLog;

impl Handler<TokenProgram, InstructionUpdate> for TokenLog {
    async fn handle(&self, value: &TokenProgram, raw: &InstructionUpdate) -> HandlerResult<()> {
        print("spl-token", raw, value);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Token2022Log;

impl Handler<TokenExtensionProgram, InstructionUpdate> for Token2022Log {
    async fn handle(
        &self,
        value: &TokenExtensionProgram,
        raw: &InstructionUpdate,
    ) -> HandlerResult<()> {
        print("token-2022", raw, value);
        Ok(())
    }
}

#[derive(Debug)]
pub struct StakePoolLog;

impl Handler<StakePoolProgram, InstructionUpdate> for StakePoolLog {
    async fn handle(&self, value: &StakePoolProgram, raw: &InstructionUpdate) -> HandlerResult<()> {
        print("stake-pool", raw, value);
        Ok(())
    }
}

fn fail(context: &str, err: impl Display) -> ! {
    eprintln!("{context}: {err}");
    std::process::exit(1)
}

fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config)
        .unwrap_or_else(|err| fail("Error reading config file", err));
    // The Debug text of a TOML error holds the whole file, x-token included.
    let config: ShipsternConfig<YellowstoneGrpcConfig> = toml::from_str(&config)
        .unwrap_or_else(|err: toml::de::Error| fail("Error parsing config", err.message()));

    Runtime::builder()
        .instruction(Pipeline::new(IncludeProgram(TokenIx), [TokenLog]))
        .instruction(Pipeline::new(IncludeProgram(Token2022Ix), [Token2022Log]))
        .instruction(Pipeline::new(IncludeProgram(StakePoolIx), [StakePoolLog]))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use shipstern::{
        handler::BoxPipeline, instruction::InstructionPipeline, shipstern_core::GetPrefilter,
    };

    use super::*;

    fn merged(pipelines: Vec<BoxPipeline<'static, InstructionUpdate>>) -> TransactionPrefilter {
        let bundle = InstructionPipeline::new(pipelines).expect("one or more pipelines");
        bundle
            .prefilter()
            .transaction
            .expect("a transaction prefilter")
    }

    #[test]
    fn subscription_selects_any_program() {
        let programs: HashSet<_> = [
            TokenIx.program_id(),
            Token2022Ix.program_id(),
            StakePoolIx.program_id(),
        ]
        .into();

        let built_in = merged(vec![
            Box::new(Pipeline::new(TokenIx, [TokenLog])),
            Box::new(Pipeline::new(Token2022Ix, [Token2022Log])),
            Box::new(Pipeline::new(StakePoolIx, [StakePoolLog])),
        ]);
        assert_eq!(built_in.accounts_required, programs);
        assert!(built_in.accounts_include.is_empty());

        let wrapped = merged(vec![
            Box::new(Pipeline::new(IncludeProgram(TokenIx), [TokenLog])),
            Box::new(Pipeline::new(IncludeProgram(Token2022Ix), [Token2022Log])),
            Box::new(Pipeline::new(IncludeProgram(StakePoolIx), [StakePoolLog])),
        ]);
        assert!(wrapped.accounts_required.is_empty());
        assert_eq!(wrapped.accounts_include, programs);
        assert_eq!(wrapped.failed, Some(false));
    }
}

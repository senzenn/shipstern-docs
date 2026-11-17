// #region block-footer
use std::borrow::Cow;

use shipstern::{config::ShipsternConfig, Handler, HandlerResult, Pipeline, Runtime};
use shipstern_core::{BlockFooterUpdate, ParseResult, Parser, Prefilter};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};

#[derive(Debug, Clone, Copy)]
pub struct FooterParser;

impl Parser for FooterParser {
    type Input = BlockFooterUpdate;
    type Output = BlockFooterUpdate;

    fn id(&self) -> Cow<'static, str> { "my_crate::FooterParser".into() }

    // `false`: send the footer without its vote certificates.
    fn prefilter(&self) -> Prefilter { Prefilter::builder().block_footers(false).build().unwrap() }

    async fn parse(&self, footer: &BlockFooterUpdate) -> ParseResult<Self::Output> {
        Ok(footer.clone())
    }
}

#[derive(Debug)]
pub struct FooterLogger;

impl Handler<BlockFooterUpdate, BlockFooterUpdate> for FooterLogger {
    async fn handle(
        &self,
        footer: &BlockFooterUpdate,
        _raw: &BlockFooterUpdate,
    ) -> HandlerResult<()> {
        // Other updates of this block have the same (slot, bank_id).
        tracing::info!(
            slot = footer.slot,
            bank_id = footer.bank_id,
            producer_time_nanos = footer.block_producer_time_nanos,
            "block footer"
        );
        Ok(())
    }
}

pub fn run(config: ShipsternConfig<YellowstoneGrpcConfig>) {
    Runtime::builder()
        .block_footer(Pipeline::new(FooterParser, [FooterLogger]))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}
// #endregion block-footer

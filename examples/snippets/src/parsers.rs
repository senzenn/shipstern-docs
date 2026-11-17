// #region custom-parser
use std::borrow::Cow;

use shipstern_core::{ParseResult, Parser, Prefilter, SlotUpdate};

#[derive(Debug, Clone, Copy)]
pub struct MySlotParser;

impl Parser for MySlotParser {
    type Input = SlotUpdate;
    type Output = SlotUpdate;

    fn id(&self) -> Cow<'static, str> { "my_crate::MySlotParser".into() }

    fn prefilter(&self) -> Prefilter { Prefilter::builder().slots().build().unwrap() }

    async fn parse(&self, slot: &SlotUpdate) -> ParseResult<Self::Output> { Ok(slot.to_owned()) }
}
// #endregion custom-parser

//! Code fragments embedded in the docs with `<include>`. Each `// #region name`
//! block is one sample. Code outside the regions makes them compile, and the
//! test modules check claims the pages make.
#![allow(dead_code)]

pub mod block_footers;
pub mod filters;
pub mod handlers;
pub mod instruction_context;
pub mod parsers;
pub mod production;
pub mod token_extensions;

#[cfg(test)]
mod fixture_tests;

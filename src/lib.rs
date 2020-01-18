//! Tree printer.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::{
    config::EdgeConfig,
    item_writer::{ItemState, ItemStyle, ItemWriter, ItemWriterOptions},
    tree_printer::TreePrinter,
};

pub(crate) mod config;
pub(crate) mod item_writer;
mod tree_printer;

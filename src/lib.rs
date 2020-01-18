//! Tree printer.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::{
    item_writer::{EdgeConfig, ItemState, ItemWriter, ItemWriterOptions},
    tree_printer::TreePrinter,
};

pub(crate) mod item_writer;
mod tree_printer;

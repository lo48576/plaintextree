//! Tree printer.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::{
    config::{EdgeConfig, ItemStyle, TreeConfig, TreeConfigBuilder},
    item_writer::{ItemState, ItemWriter},
    tree_printer::TreePrinter,
};

pub(crate) mod config;
pub(crate) mod item_writer;
pub mod tree_printer;

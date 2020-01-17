//! Tree printer.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::item_writer::{ItemWriter, ItemWriterOptions};

mod item_writer;

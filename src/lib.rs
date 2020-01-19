//! Tree printer.
//!
//! # How to use
//!
//! 1.  Prepare a writer which implements [`std::fmt::Write`].
//! 2.  Prepare a [`TreeConfig`] using [`TreeConfigBuilder`], or use the default value.
//! 3.  Create a [`TreePrinter`].
//! 4.  [Open a new node][open_node] or [close a node][close_node] as you need.
//!     Specify [`ItemStyle`] for each node to control the visual styles.
//! 5.  Finally, [`TreePrinter::finalize()`] will finalize the tree printing.
//!     You get the inner writer as the retern value.
//!
//! # Examples
//!
//! ```
//! use plaintextree::{EdgeConfig, ItemStyle, TreeConfig, TreePrinter};
//!
//! let mut buf = ".\n".to_owned();
//! // You can pass `&mut String` as the inner writer.
//! let mut printer = TreePrinter::new(&mut buf, TreeConfig::new());
//! let edge = EdgeConfig::Ascii;
//!
//! printer.open_node(ItemStyle::non_last(edge.clone()), "foo")?;
//! printer.open_node(ItemStyle::non_last(edge.clone()), "bar")?;
//! printer.open_node(ItemStyle::last(edge.clone()), "baz\n\nmultiline support!")?;
//! printer.close_node()?; // baz
//! printer.close_node()?; // bar
//! printer.open_node(ItemStyle::last(edge.clone()), "qux")?;
//! printer.open_node(ItemStyle::last(edge.clone()), "quux")?;
//! printer.close_node()?; // quux
//! printer.close_node()?; // qux
//! printer.close_node()?; // foo
//! printer.open_node(ItemStyle::non_last(edge.clone()), "corge\n")?;
//! printer.close_node()?; // corge
//! printer.open_node(ItemStyle::last(edge.clone()), "grault")?;
//! printer.close_node()?; // grault
//!
//! // `finalize()` returns `Result<&mut String, _>` here.
//! let _ = printer.finalize()?;
//!
//! let expected = ".\n\
//!                 |-- foo\n\
//!                 |   |-- bar\n\
//!                 |   |   `-- baz\n\
//!                 |   |\n\
//!                 |   |       multiline support!\n\
//!                 |   `-- qux\n\
//!                 |       `-- quux\n\
//!                 |-- corge\n\
//!                 `-- grault\n";
//!
//! assert_eq!(buf, expected);
//! # Ok::<_, plaintextree::Error>(())
//! ```
//!
//! Unicode ruled line characters can also be used.
//!
//! ```
//! use plaintextree::{EdgeConfig, ItemStyle, TreeConfig, TreePrinter};
//!
//! // You can pass `String` directly as the inner writer.
//! let mut printer = TreePrinter::new(".\n".to_owned(), TreeConfig::new());
//! let edge = EdgeConfig::UnicodeSingleWidth;
//!
//! printer.open_node(ItemStyle::non_last(edge.clone()), "foo")?;
//! printer.open_node(ItemStyle::non_last(edge.clone()), "bar")?;
//! printer.close_node()?; // bar
//! printer.open_node(ItemStyle::last(edge.clone()), "baz")?;
//! printer.close_node()?; // baz
//! printer.close_node()?; // foo
//! printer.open_node(ItemStyle::last(edge.clone()), "qux")?;
//! printer.close_node()?; // qux
//!
//! // `finalize()` returns `Result<String, _>` here.
//! let got = printer.finalize()?;
//!
//! let expected = ".\n\
//!                 ├── foo\n\
//!                 │   ├── bar\n\
//!                 │   └── baz\n\
//!                 └── qux\n";
//!
//! assert_eq!(got, expected);
//! # Ok::<_, plaintextree::Error>(())
//! ```
//!
//! [`std::fmt::Write`]: https://doc.rust-lang.org/stable/std/fmt/trait.Write.html
//! [`ItemStyle`]: struct.ItemStyle.html
//! [`TreeConfig`]: struct.TreeConfig.html
//! [`TreeConfigBuilder`]: struct.TreeConfigBuilder.html
//! [`TreePrinter`]: struct.TreePrinter.html
//! [`TreePrinter::finalize()`]: struct.TreePrinter.html#method.finalize
//! [close_node]: struct.TreePrinter.html#method.close_node
//! [open_node]: struct.TreePrinter.html#method.open_node
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::{
    config::{unicode, EdgeConfig, ItemStyle, TreeConfig, TreeConfigBuilder},
    tree_printer::{Error, Result, TreePrinter},
};

pub(crate) mod config;
pub(crate) mod item_writer;
pub(crate) mod tree_printer;

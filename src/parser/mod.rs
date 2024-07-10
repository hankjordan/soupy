mod html;
#[cfg(feature = "xml")]
mod xml;

pub use html::*;
#[cfg(feature = "xml")]
pub use xml::*;

use crate::Node;

/// Used to convert a string into a [`Vec`] of nodes.
pub trait Parser<'a> {
    type Node: Node + 'a;
    /// The error thrown when parsing fails.
    type Error;

    /// Attempts to parse the text with the `Parser`.
    ///
    /// # Errors
    /// If the text has an invalid format.
    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error>;
}

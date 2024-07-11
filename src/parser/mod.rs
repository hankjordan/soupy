mod html;
#[cfg(feature = "xml")]
mod xml;

pub use html::*;
#[cfg(feature = "xml")]
pub use xml::*;

use crate::Node;

/// Used to convert a string into a [`Vec`] of nodes.
pub trait Parser {
    /// Input type.
    type Input;
    /// The node type.
    type Node: Node;
    /// The error thrown when parsing fails.
    type Error;

    /// Attempts to parse the input with the `Parser`.
    ///
    /// # Errors
    /// If the input has an invalid format.
    fn parse(input: Self::Input) -> Result<Vec<Self::Node>, Self::Error>;
}

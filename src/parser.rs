use crate::HTMLNode;

/// Used to convert a string into a [`Vec`] of nodes.
pub trait Parser<'a> {
    type Node: 'a;
    /// The error thrown when parsing fails.
    type Error;

    /// Attempts to parse the text with the `Parser`.
    ///
    /// # Errors
    /// If the text has an invalid format.
    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error>;
}

/// Default HTML parser.
///
/// Errors on malformed HTML.
#[derive(Clone, Debug)]
pub struct StrictHTMLParser;

impl<'a> Parser<'a> for StrictHTMLParser {
    type Node = HTMLNode<'a>;
    type Error = nom::Err<nom::error::Error<&'a str>>;

    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error> {
        crate::parse::parse(text).map(|r| r.1)
    }
}

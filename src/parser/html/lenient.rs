use std::convert::Infallible;

use crate::parser::Parser;

/// Lenient HTML parser.
///
/// Attempts to work through invalid HTML.
#[derive(Clone, Debug)]
pub struct LenientHTMLParser;

impl<'a> Parser<'a> for LenientHTMLParser {
    type Node = scraper::Node;
    type Error = Infallible;

    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error> {
        Ok(scraper::Html::parse_document(text)
            .tree
            .values()
            .cloned()
            .collect())
    }
}

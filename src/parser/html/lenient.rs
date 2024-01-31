use std::convert::Infallible;

use crate::{
    node::HTMLNode,
    parser::Parser,
};

/// Lenient HTML parser.
///
/// Attempts to work through invalid HTML.
#[derive(Clone, Debug)]
pub struct LenientHTMLParser;

impl<'a> Parser<'a> for LenientHTMLParser {
    type Text = scraper::StrTendril;
    type Node = HTMLNode<scraper::StrTendril>;
    type Error = Infallible;

    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error> {
        Ok(scraper::Html::parse_document(text)
            .tree
            .root()
            .children()
            .filter_map(|n| n.try_into().ok())
            .collect())
    }
}

#[allow(clippy::mutable_key_type)]
impl<'a> TryFrom<ego_tree::NodeRef<'a, scraper::Node>> for HTMLNode<scraper::StrTendril> {
    type Error = ();

    fn try_from(node: ego_tree::NodeRef<'a, scraper::Node>) -> Result<Self, Self::Error> {
        match node.value() {
            scraper::Node::Document
            | scraper::Node::Fragment
            | scraper::Node::ProcessingInstruction(_) => Err(()),
            scraper::Node::Doctype(doctype) => Ok(HTMLNode::Doctype(doctype.name.clone())),
            scraper::Node::Comment(comment) => Ok(HTMLNode::Comment(comment.comment.clone())),
            scraper::Node::Text(text) => Ok(HTMLNode::Text(text.text.clone())),
            scraper::Node::Element(element) => {
                let name = element.name().into();
                let attrs = element.attrs().map(|(k, v)| (k.into(), v.into())).collect();

                Ok(match element.name() {
                    "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link"
                    | "meta" | "source" | "track" | "wbr" => HTMLNode::Void { name, attrs },
                    _ => HTMLNode::Element {
                        name,
                        attrs,
                        children: node.children().filter_map(|e| e.try_into().ok()).collect(),
                    },
                })
            }
        }
    }
}

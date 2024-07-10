use std::collections::BTreeMap;

use xmltree::Namespace;

use crate::{
    parser::Parser,
    Node,
};

#[derive(Clone, Debug)]
pub struct XMLParser;

impl<'a> Parser<'a> for XMLParser {
    type Text = String;
    type Node = XMLNode;
    type Error = xmltree::ParseError;

    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error> {
        Ok(xmltree::Element::parse_all(text.as_bytes())?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

/// Represents an XML element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XMLElement {
    /// This elements prefix, if any
    pub prefix: Option<String>,

    /// This elements namespace, if any
    pub namespace: Option<String>,

    /// The full list of namespaces, if any
    ///
    /// The `Namespace` type is exported from the `xml-rs` crate.
    pub namespaces: Option<Namespace>,

    /// The name of the Element.  Does not include any namespace info
    pub name: String,

    /// The Element attributes
    ///
    /// By default, this is a `HashMap`, but if the optional "attribute-order" feature is enabled,
    /// this is an [IndexMap](https://docs.rs/indexmap/1.4.0/indexmap/), which will retain
    /// item insertion order.
    pub attributes: BTreeMap<String, String>,

    /// Children
    pub children: Vec<XMLNode>,
}

impl From<xmltree::Element> for XMLElement {
    fn from(value: xmltree::Element) -> Self {
        Self {
            prefix: value.prefix,
            namespace: value.namespace,
            namespaces: value.namespaces,
            name: value.name,
            attributes: value.attributes.into_iter().collect(),
            children: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XMLNode {
    Element(XMLElement),
    Comment(String),
    CData(String),
    Text(String),
    ProcessingInstruction(String, Option<String>),
}

impl From<xmltree::XMLNode> for XMLNode {
    fn from(value: xmltree::XMLNode) -> Self {
        match value {
            xmltree::XMLNode::Element(e) => XMLNode::Element(e.into()),
            xmltree::XMLNode::Comment(c) => XMLNode::Comment(c),
            xmltree::XMLNode::CData(d) => XMLNode::CData(d),
            xmltree::XMLNode::Text(t) => XMLNode::Text(t),
            xmltree::XMLNode::ProcessingInstruction(a, b) => XMLNode::ProcessingInstruction(a, b),
        }
    }
}

impl Node for XMLNode {
    type Text = String;

    fn name(&self) -> Option<&String> {
        match &self {
            XMLNode::Element(e) => Some(&e.name),
            _ => None,
        }
    }

    fn attrs(&self) -> Option<&BTreeMap<String, String>> {
        match &self {
            XMLNode::Element(e) => Some(&e.attributes),
            _ => None,
        }
    }
}

impl XMLNode {
    #[must_use]
    pub fn iter(&self) -> XMLNodeIter {
        let e = if let XMLNode::Element(e) = &self {
            Some(e.children.iter())
        } else {
            None
        };

        XMLNodeIter {
            iter: e.into_iter().flatten(),
        }
    }
}

pub struct XMLNodeIter<'a> {
    iter: std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, XMLNode>>>,
}

impl<'a> Iterator for XMLNodeIter<'a> {
    type Item = &'a XMLNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> IntoIterator for &'a XMLNode {
    type Item = &'a XMLNode;
    type IntoIter = XMLNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

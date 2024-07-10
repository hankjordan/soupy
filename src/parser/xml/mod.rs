use std::collections::BTreeMap;

use xmltree::Namespace;

use crate::{
    parser::Parser,
    Node,
};

#[derive(Clone, Debug)]
pub struct XMLParser;

impl<'a> Parser<'a> for XMLParser {
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
    /// Iterate over child nodes
    #[must_use]
    pub fn iter(&self) -> XMLNodeIter {
        XMLNodeIter {
            node: self,
            child: None,
            next: None,
        }
    }
}

/// An [`Iterator`] over an [`XMLNode`] and its children.
pub struct XMLNodeIter<'a> {
    node: &'a XMLNode,
    child: Option<Box<XMLNodeIter<'a>>>,
    next: Option<usize>,
}

impl<'a> Iterator for XMLNodeIter<'a> {
    type Item = &'a XMLNode;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(child) = self.child.as_mut() {
                if let Some(next) = child.next() {
                    return Some(next);
                }

                self.child = None;
            } else if let Some(next) = self.next {
                if let XMLNode::Element(XMLElement { children, .. }) = self.node {
                    if let Some(child) = children.get(next) {
                        self.child = Some(Box::new(child.into_iter()));
                        self.next = Some(next + 1);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                self.next = Some(0);
                return Some(self.node);
            }
        }
    }
}

impl<'a> IntoIterator for &'a XMLNode {
    type Item = &'a XMLNode;
    type IntoIter = XMLNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::*;

    const HELLO: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<root xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <simple>Here's some text</simple>
    <complex id="hello">
        <nested>Nested text!</nested>
        <example>More text</example>

        <tree depth="1">
            <tree depth="2">
                <tree depth="3">Tree text</tree>
            </tree>
        </tree>
    </complex>
</root>"#;

    #[test]
    fn test_iterator() {
        let soup = Soup::xml(HELLO).expect("Failed to parse XML");

        let complex = soup
            .tag("complex")
            .first()
            .expect("Could not find 'complex' tag")
            .deref()
            .clone();

        let mut nodes = complex.iter();

        // Node iterator must start with parent element, then recurse over children depth first.
        // TODO: replace with special trait?

        assert_eq!(nodes.next().unwrap().name(), Some(&"complex".into()));
    }
}

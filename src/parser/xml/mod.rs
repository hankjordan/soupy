use std::{
    collections::BTreeMap,
    io::Read,
    marker::PhantomData,
};

use xmltree::Namespace;

use crate::{
    parser::Parser,
    Node,
};

/// Default XML parser
///
/// Errors on malformed XML.
#[derive(Clone, Debug)]
pub struct XMLParser<R> {
    _marker: PhantomData<R>,
}

impl<R> Parser for XMLParser<R>
where
    R: Read,
{
    type Input = R;
    type Node = XMLNode;
    type Error = xmltree::ParseError;

    fn parse(reader: R) -> Result<Vec<Self::Node>, Self::Error> {
        Ok(xmltree::Element::parse_all(reader)?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

/// Represents an XML element
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct XMLElement {
    /// This elements prefix, if any
    pub prefix: Option<String>,

    /// This elements namespace, if any
    pub namespace: Option<String>,

    /// The full list of namespaces, if any
    pub namespaces: Option<Namespace>,

    /// The name of the Element
    pub name: String,

    /// The Element attributes
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

/// Represents an XML node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XMLNode {
    /// XML element which can contain children nodes
    Element(XMLElement),

    /// Comment
    Comment(String),

    /// CDATA
    CData(String),

    /// Text
    Text(String),

    /// Processing Instruction
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
        match self {
            XMLNode::Element(e) => Some(&e.name),
            _ => None,
        }
    }

    fn text(&self) -> Option<&String> {
        match self {
            XMLNode::Text(t) => Some(t),
            _ => None,
        }
    }

    fn attrs(&self) -> Option<&BTreeMap<String, String>> {
        match self {
            XMLNode::Element(e) => Some(&e.attributes),
            _ => None,
        }
    }

    fn children(&self) -> &[Self] {
        if let XMLNode::Element(e) = &self {
            e.children.as_slice()
        } else {
            &[]
        }
    }
}

impl XMLNode {
    /// Iterate over direct children
    pub fn iter(&self) -> std::slice::Iter<Self> {
        self.children().iter()
    }
}

impl<'a> IntoIterator for &'a XMLNode {
    type Item = &'a XMLNode;
    type IntoIter = std::slice::Iter<'a, XMLNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::*;

    const HELLO: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<root>
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

    <b>
        <a>Inner text</a>
    </b>

    <a>Outer text</a>
</root>"#;

    #[test]
    fn test_text() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let example = soup
            .tag("example")
            .first()
            .expect("Could not find 'example' tag");

        // TODO: Fix borrow lifetime issue here
        let child = example
            .children()
            .first()
            .expect("Could not find 'example' child node");

        assert_eq!(child.text(), Some(&"More text".into()));

        let root = soup.tag("root").first().expect("Could not find 'root' tag");

        assert_eq!(
            root.all_text(),
            "Here's some text\nNested text!\nMore text\nTree text\nInner text\nOuter text"
        );
    }

    #[test]
    fn test_tree_iter() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let complex = soup
            .tag("complex")
            .first()
            .expect("Could not find 'complex' tag")
            .deref()
            .clone();

        let mut nodes = complex.descendants();

        assert_eq!(nodes.next().unwrap().name(), Some(&"complex".into()));

        assert_eq!(
            nodes.next().unwrap(),
            &XMLNode::Element(XMLElement {
                name: "nested".into(),
                children: vec![XMLNode::Text("Nested text!".into())],
                ..Default::default()
            })
        );

        assert_eq!(nodes.next().unwrap(), &XMLNode::Text("Nested text!".into()));

        assert_eq!(
            nodes.next().unwrap(),
            &XMLNode::Element(XMLElement {
                name: "example".into(),
                children: vec![XMLNode::Text("More text".into())],
                ..Default::default()
            })
        );

        assert_eq!(nodes.next().unwrap(), &XMLNode::Text("More text".into()));
    }

    #[test]
    fn test_direct_iter() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let complex = soup
            .tag("complex")
            .first()
            .expect("Could not find 'complex' tag")
            .deref()
            .clone();

        let mut nodes = complex.into_iter();

        assert_eq!(
            nodes.next().unwrap(),
            &XMLNode::Element(XMLElement {
                name: "nested".into(),
                children: vec![XMLNode::Text("Nested text!".into())],
                ..Default::default()
            })
        );

        assert_eq!(
            nodes.next().unwrap(),
            &XMLNode::Element(XMLElement {
                name: "example".into(),
                children: vec![XMLNode::Text("More text".into())],
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_iter_order() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let soup = soup
            .tag("root")
            .first()
            .expect("Failed to find 'root' tag")
            .query();

        // By default, the data is searched recursively, depth-first.
        assert_eq!(
            soup.tag("a").first().map(|t| t.all_text()),
            Some("Inner text".into())
        );

        // Strict queries only match direct children, no recursion.
        assert_eq!(
            soup.strict().tag("a").first().map(|t| t.all_text()),
            Some("Outer text".into())
        );
    }
}

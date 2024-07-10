use std::collections::BTreeMap;

use crate::node::Node;

/// An HTML node
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HTMLNode<S> {
    /// A comment, like `<!-- ... -->`
    Comment(S),
    /// The doctype, like `<!DOCTYPE ...>`
    Doctype(S),
    /// A standard element, like `<p> ... </p>`
    Element {
        name: S,
        attrs: BTreeMap<S, S>,
        children: Vec<HTMLNode<S>>,
    },
    /// An element that contains code, like `<script> ... </script>`
    RawElement {
        name: S,
        attrs: BTreeMap<S, S>,
        content: S,
    },
    /// A void element that is unable to contain children, like `<input>`
    Void { name: S, attrs: BTreeMap<S, S> },
    /// Raw text
    Text(S),
}

impl<S> HTMLNode<S> {
    /// Iterate over child nodes
    pub fn iter(&self) -> HTMLNodeIter<S> {
        HTMLNodeIter {
            node: self,
            child: None,
            next: None,
        }
    }
}

impl<S> Node for HTMLNode<S> {
    type Text = S;

    #[must_use]
    fn name(&self) -> Option<&S> {
        match self {
            Self::Element { name, .. }
            | Self::RawElement { name, .. }
            | Self::Void { name, .. } => Some(name),
            _ => None,
        }
    }

    fn attrs(&self) -> Option<&BTreeMap<S, S>> {
        match self {
            Self::Element { attrs, .. }
            | Self::RawElement { attrs, .. }
            | Self::Void { attrs, .. } => Some(attrs),
            _ => None,
        }
    }
}

/// An [`Iterator`] over a [`Node`] and its children.
pub struct HTMLNodeIter<'a, S> {
    node: &'a HTMLNode<S>,
    child: Option<Box<HTMLNodeIter<'a, S>>>,
    next: Option<usize>,
}

impl<'a, S> Iterator for HTMLNodeIter<'a, S> {
    type Item = &'a HTMLNode<S>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(child) = self.child.as_mut() {
                if let Some(next) = child.next() {
                    return Some(next);
                }

                self.child = None;
            } else if let Some(next) = self.next {
                if let HTMLNode::Element { children, .. } = self.node {
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

impl<'a, S> IntoIterator for &'a HTMLNode<S> {
    type Item = &'a HTMLNode<S>;
    type IntoIter = HTMLNodeIter<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

use std::collections::BTreeMap;

/// An HTML node
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HTMLNode<'a> {
    /// A comment, like `<!-- ... -->`
    Comment(&'a str),
    /// The doctype, like `<!DOCTYPE ...>`
    Doctype(&'a str),
    /// A standard element, like `<p> ... </p>`
    Element {
        name: &'a str,
        attrs: BTreeMap<&'a str, &'a str>,
        children: Vec<HTMLNode<'a>>,
    },
    /// An element that contains code, like `<script> ... </script>`
    RawElement {
        name: &'a str,
        attrs: BTreeMap<&'a str, &'a str>,
        content: &'a str,
    },
    /// A void element that is unable to contain children, like `<input>`
    Void {
        name: &'a str,
        attrs: BTreeMap<&'a str, &'a str>,
    },
    /// Raw text
    Text(&'a str),
}

impl<'a> HTMLNode<'a> {
    /// Retrieves the name of the node
    ///
    /// Only returns [`Some`] for [`Node::Element`], [`Node::RawElement`], or [`Node::Void`].
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Element { name, .. }
            | Self::RawElement { name, .. }
            | Self::Void { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Returns the node's attributes as a [`BTreeMap`]
    #[must_use]
    pub fn attrs(&self) -> Option<&BTreeMap<&str, &str>> {
        match self {
            Self::Element { attrs, .. }
            | Self::RawElement { attrs, .. }
            | Self::Void { attrs, .. } => Some(attrs),
            _ => None,
        }
    }

    /// Looks for an attribute named `attr` and returns its value
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::new(r#"<div class="foo bar"></div>"#).unwrap();
    /// let div = soup.tag("div").first().expect("Couldn't find div");
    /// assert_eq!(div.get("class"), Some("foo bar"));
    /// ```
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs().and_then(|a| a.get(name)).copied()
    }
}

/// An [`Iterator`] over a [`Node`] and its children.
pub struct HTMLNodeIter<'a> {
    node: &'a HTMLNode<'a>,
    child: Option<Box<HTMLNodeIter<'a>>>,
    next: Option<usize>,
}

impl<'a> Iterator for HTMLNodeIter<'a> {
    type Item = &'a HTMLNode<'a>;

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

impl<'a> IntoIterator for &'a HTMLNode<'a> {
    type Item = &'a HTMLNode<'a>;
    type IntoIter = HTMLNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HTMLNodeIter {
            node: self,
            child: None,
            next: None,
        }
    }
}

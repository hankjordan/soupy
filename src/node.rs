use std::collections::BTreeMap;

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
    /// Retrieves the name of the node
    ///
    /// Only returns [`Some`] for [`Node::Element`], [`Node::RawElement`], or [`Node::Void`].
    #[must_use]
    pub fn name(&self) -> Option<&S> {
        match self {
            Self::Element { name, .. }
            | Self::RawElement { name, .. }
            | Self::Void { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Returns the node's attributes as a [`BTreeMap`]
    #[must_use]
    pub fn attrs(&self) -> Option<&BTreeMap<S, S>> {
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
    pub fn get<Q>(&self, name: &Q) -> Option<&S>
    where
        S: Ord + std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.attrs().and_then(|a| a.get(name))
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
        HTMLNodeIter {
            node: self,
            child: None,
            next: None,
        }
    }
}

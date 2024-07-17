use std::collections::BTreeMap;

/// Basic queryable unit of the data structure
pub trait Node: Sized {
    /// Type of text values returned
    type Text;

    /// Returns the name of the node
    fn name(&self) -> Option<&Self::Text>;

    /// Returns the direct text content of the node, if any
    fn text(&self) -> Option<&Self::Text>;

    /// Returns the node's attributes as a [`BTreeMap`]
    #[must_use]
    fn attrs(&self) -> Option<&BTreeMap<Self::Text, Self::Text>>;

    /// Looks for an attribute named `attr` and returns its value
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div class="foo bar"></div>"#).unwrap();
    /// let div = soup.tag("div").first().expect("Couldn't find div");
    /// assert_eq!(div.get("class"), Some(&"foo bar"));
    /// ```
    #[must_use]
    fn get<'a, Q>(&self, name: &'a Q) -> Option<&Self::Text>
    where
        Self::Text: Ord + From<&'a Q>,
        Q: ?Sized,
    {
        self.attrs().and_then(|a| a.get(&name.into()))
    }

    /// Direct children of the node
    fn children(&self) -> &[Self];

    /// Depth-first iterator over children of the node, including the root
    fn descendants(&self) -> NodeIter<Self> {
        NodeIter::tree(self)
    }

    /// Returns all text content contained within the node's tree
    fn all_text(&self) -> String
    where
        Self::Text: std::fmt::Display,
    {
        self.descendants()
            .filter_map(|n| n.text())
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub enum NodeIter<'x, N> {
    Direct {
        iter: std::slice::Iter<'x, N>,
    },
    Tree {
        node: &'x N,
        child: Option<Box<NodeIter<'x, N>>>,
        next: Option<usize>,
    },
}

impl<'x, N> NodeIter<'x, N>
where
    N: Node,
{
    pub(crate) fn direct(iter: std::slice::Iter<'x, N>) -> Self {
        Self::Direct { iter }
    }

    pub(crate) fn tree(node: &'x N) -> Self {
        Self::Tree {
            node,
            child: None,
            next: None,
        }
    }
}

impl<'x, N> Iterator for NodeIter<'x, N>
where
    N: Node,
{
    type Item = &'x N;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NodeIter::Direct { iter } => iter.next(),
            NodeIter::Tree { node, child, next } => loop {
                if let Some(c) = child.as_mut() {
                    if let Some(next) = c.next() {
                        return Some(next);
                    }

                    *child = None;
                } else if let Some(n) = next {
                    let children = node.children();

                    if let Some(c) = children.get(*n) {
                        *child = Some(Box::new(Self::tree(c)));
                        *next = Some(*n + 1);
                    } else {
                        return None;
                    }
                } else {
                    *next = Some(0);
                    return Some(*node);
                }
            },
        }
    }
}

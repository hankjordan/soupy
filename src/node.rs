use std::{
    collections::BTreeMap,
    marker::PhantomData,
};

pub trait Node: Sized {
    type Text;

    /// Returns the name of the node
    fn name(&self) -> Option<&Self::Text>;

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
    fn tree(&self) -> TreeIter<Self> {
        TreeIter::new(self)
    }
}

pub(crate) struct MapTreeIter<'a, I> {
    iter: I,
    _marker: PhantomData<&'a ()>,
}

impl<'a, I> MapTreeIter<'a, I> {
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            _marker: PhantomData,
        }
    }
}

impl<'a, I, N> Iterator for MapTreeIter<'a, I>
where
    N: 'a,
    I: Iterator<Item = &'a N>,
{
    type Item = TreeIter<'a, N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|n| TreeIter::new(n))
    }
}

pub struct TreeIter<'a, N> {
    node: &'a N,
    child: Option<Box<TreeIter<'a, N>>>,
    next: Option<usize>,
}

impl<'a, N> TreeIter<'a, N> {
    pub fn new(node: &'a N) -> Self {
        Self {
            node,
            child: None,
            next: None,
        }
    }
}

impl<'a, N> Iterator for TreeIter<'a, N>
where
    N: Node,
{
    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(child) = self.child.as_mut() {
                if let Some(next) = child.next() {
                    return Some(next);
                }

                self.child = None;
            } else if let Some(next) = self.next {
                let children = self.node.children();

                if let Some(child) = children.get(next) {
                    self.child = Some(Box::new(Self::new(child)));
                    self.next = Some(next + 1);
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

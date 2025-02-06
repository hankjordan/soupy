use crate::{
    filter::{
        And,
        Attr,
        Filter,
        Tag,
    },
    node::NodeIter,
    Node,
    Pattern,
    Soup,
};

/// A query for elements in [`Soup`](`crate::Soup`) matching the [`Filter`](`crate::filter::Filter`) `F`
#[derive(Debug)]
pub struct Query<'x, N, F> {
    soup: &'x Soup<N>,
    recursive: bool,
    filter: F,
}

impl<N, F> Copy for Query<'_, N, F> where F: Copy {}

impl<N, F> Clone for Query<'_, N, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            soup: self.soup,
            recursive: self.recursive,
            filter: self.filter.clone(),
        }
    }
}

/// Allows you to query for sub-elements matching the given [`Filter`](`crate::filter::Filter`)
#[allow(clippy::type_complexity)]
pub trait Queryable<'x>: Sized {
    /// Query node type
    type Node: Node;

    /// Already applied filter
    type Filter: Filter<Self::Node>;

    /// Allows the query to search the entire node tree
    fn recursive(self) -> Query<'x, Self::Node, Self::Filter>;

    /// Forces the query to only match direct children of the root node
    fn strict(self) -> Query<'x, Self::Node, Self::Filter>;

    /// Specifies a tag for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.tag("b").first().expect("Couldn't find tag 'b'");
    /// assert_eq!(result.get("id"), Some(&"bold-tag".into()));
    /// ```
    fn tag<T>(self, tag: T) -> Query<'x, Self::Node, And<Self::Filter, Tag<T>>>
    where
        T: Pattern<<Self::Node as Node>::Text>,
        Tag<T>: Filter<Self::Node>;

    /// Specifies an attribute name/value pair for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr("id", "bold-tag").first().expect("Couldn't find tag with id 'bold-tag'");
    /// assert_eq!(result.name(), Some(&"b".into()));
    fn attr<Q, V>(self, name: Q, value: V) -> Query<'x, Self::Node, And<Self::Filter, Attr<Q, V>>>
    where
        Q: Pattern<<Self::Node as Node>::Text>,
        V: Pattern<<Self::Node as Node>::Text>,
        Attr<Q, V>: Filter<Self::Node>;

    /// Searches for a tag that has an attribute with the specified name
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr_name("id").first().expect("Couldn't find element with an 'id'");
    /// assert_eq!(result.name(), Some(&"b".into()));
    /// ```
    fn attr_name<Q>(self, name: Q) -> Query<'x, Self::Node, And<Self::Filter, Attr<Q, bool>>>
    where
        Q: Pattern<<Self::Node as Node>::Text>,
        Attr<Q, bool>: Filter<Self::Node>,
    {
        self.attr(name, true)
    }

    /// Search for a node with any attribute with a value that matches the specified value
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr_value("bold-tag").first().expect("Couldn't find a tag with attribute value 'bold-tag'");
    /// assert_eq!(result.name(), Some(&"b".into()));
    /// ```
    fn attr_value<V>(self, value: V) -> Query<'x, Self::Node, And<Self::Filter, Attr<bool, V>>>
    where
        V: Pattern<<Self::Node as Node>::Text>,
        Attr<bool, V>: Filter<Self::Node>,
    {
        self.attr(true, value)
    }

    /// Specifies a class name for which to search
    ///
    /// NOTE: This is an *exact match*.
    /// If the element has classes other than the one you are searching for the filter will not match.
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section class="content"><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.class("content").first().expect("Couldn't find tag with class 'content'");
    /// assert_eq!(result.name(), Some(&"section".into()));
    fn class<C>(self, class: C) -> Query<'x, Self::Node, And<Self::Filter, Attr<&'static str, C>>>
    where
        C: Pattern<<Self::Node as Node>::Text>,
        <Self::Node as Node>::Text: AsRef<str> + From<&'static str>,
        Attr<&'static str, C>: Filter<Self::Node>,
    {
        self.attr("class", class)
    }

    /// Executes the query, and returns either the first result, or `None`
    ///
    /// Equivalent to calling `self.into_iter().next()`
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<ul><li id="one">One</li><li id="two">Two</li><li id="three">Three</li></ul>"#).unwrap();
    /// let result = soup.tag("li").first().expect("Couldn't find 'li'");
    /// assert_eq!(result.get("id"), Some(&"one".into()));
    /// ```
    fn first(self) -> Option<Self::Item>
    where
        Self: IntoIterator,
    {
        self.into_iter().next()
    }

    /// Executes the query, and returns an iterator of the results
    ///
    /// Equivalent to calling `self.into_iter()`
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<ul><li id="one">One</li><li id="two">Two</li><li id="three">Three</li></ul>"#).unwrap();
    /// let results = soup.tag("li").all().collect::<Vec<_>>();
    /// assert_eq!(results.len(), 3);
    /// assert_eq!(results[0].get("id"), Some(&"one".into()));
    /// assert_eq!(results[1].get("id"), Some(&"two".into()));
    /// assert_eq!(results[2].get("id"), Some(&"three".into()));
    /// ```
    fn all(self) -> Self::IntoIter
    where
        Self: IntoIterator,
    {
        self.into_iter()
    }
}

impl<'x, N, F> Queryable<'x> for Query<'x, N, F>
where
    N: Node,
    F: Filter<N>,
{
    type Node = N;
    type Filter = F;

    fn recursive(self) -> Query<'x, N, F> {
        Query {
            soup: self.soup,
            recursive: true,
            filter: self.filter,
        }
    }

    fn strict(self) -> Query<'x, N, F> {
        Query {
            soup: self.soup,
            recursive: false,
            filter: self.filter,
        }
    }

    fn tag<T>(self, tag: T) -> Query<'x, N, And<F, Tag<T>>>
    where
        T: Pattern<N::Text>,
        Tag<T>: Filter<N>,
    {
        Query {
            soup: self.soup,
            recursive: self.recursive,
            filter: And(self.filter, Tag { tag }),
        }
    }

    fn attr<Q, V>(self, name: Q, value: V) -> Query<'x, N, And<F, Attr<Q, V>>>
    where
        Q: Pattern<N::Text>,
        V: Pattern<N::Text>,
        Attr<Q, V>: Filter<N>,
    {
        Query {
            soup: self.soup,
            recursive: self.recursive,
            filter: And(self.filter, Attr { name, value }),
        }
    }
}

impl<'x, N> Queryable<'x> for &'x Soup<N>
where
    N: Node,
{
    type Node = N;
    type Filter = ();

    fn recursive(self) -> Query<'x, N, ()> {
        Query {
            soup: self,
            recursive: true,
            filter: (),
        }
    }

    fn strict(self) -> Query<'x, N, ()> {
        Query {
            soup: self,
            recursive: false,
            filter: (),
        }
    }

    fn tag<T>(self, tag: T) -> Query<'x, N, And<(), Tag<T>>>
    where
        T: Pattern<N::Text>,
        Tag<T>: Filter<N>,
    {
        Query {
            soup: self,
            recursive: true,
            filter: And((), Tag { tag }),
        }
    }

    fn attr<Q, V>(self, name: Q, value: V) -> Query<'x, N, And<(), Attr<Q, V>>>
    where
        Q: Pattern<N::Text>,
        V: Pattern<N::Text>,
        Attr<Q, V>: Filter<N>,
    {
        Query {
            soup: self,
            recursive: true,
            filter: And((), Attr { name, value }),
        }
    }
}

/// Item returned by a [`Query`]
#[derive(Debug, Copy, Clone)]
pub struct QueryItem<'x, N> {
    item: &'x N,
}

impl<N> QueryItem<'_, N>
where
    N: Node + Clone,
{
    /// Convert the item into one that can be queried
    #[must_use]
    pub fn query(&self) -> Soup<N> {
        Soup {
            nodes: self.item.children().to_vec(),
        }
    }
}

impl<N> std::ops::Deref for QueryItem<'_, N> {
    type Target = N;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

struct MapNodeIter<'x, N> {
    iter: Option<std::slice::Iter<'x, N>>,
    recursive: bool,
}

impl<'x, N> MapNodeIter<'x, N> {
    fn new(nodes: &'x [N], recursive: bool) -> Self {
        Self {
            iter: Some(nodes.iter()),
            recursive,
        }
    }
}

impl<'x, N> Iterator for MapNodeIter<'x, N>
where
    N: Node,
{
    type Item = NodeIter<'x, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.recursive {
            self.iter
                .as_mut()
                .and_then(|i| Some(NodeIter::tree(i.next()?)))
        } else {
            self.iter.take().map(|i| NodeIter::direct(i))
        }
    }
}

/// An [`Iterator`] over matching elements
pub struct QueryIter<'x, N: Node + 'x, F> {
    iter: std::iter::Flatten<MapNodeIter<'x, N>>,
    filter: F,
}

impl<'x, N, F> QueryIter<'x, N, F>
where
    N: Node,
{
    pub(crate) fn new(nodes: &'x [N], recursive: bool, filter: F) -> Self {
        Self {
            iter: MapNodeIter::new(nodes, recursive).flatten(),
            filter,
        }
    }
}

impl<'x, N, F> Iterator for QueryIter<'x, N, F>
where
    N: Node,
    F: Filter<N>,
{
    type Item = QueryItem<'x, N>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next()?;

            if self.filter.matches(next) {
                return Some(QueryItem { item: next });
            }
        }
    }
}

impl<'x, N, F> IntoIterator for Query<'x, N, F>
where
    N: Node,
    F: Filter<N>,
{
    type Item = QueryItem<'x, N>;
    type IntoIter = QueryIter<'x, N, F>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter::new(&self.soup.nodes, self.recursive, self.filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_query_copy() {
        let soup = Soup::html_strict("<b><a>one</a></b><a>two</a>").expect("Failed to parse HTML");

        let q = soup.strict();

        let q1 = q;
        let q2 = q;

        assert_eq!(
            q1.tag("a").first().map(|t| (*t).clone()),
            q2.tag("a").first().map(|t| (*t).clone())
        );
    }
}

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
pub struct Query<'x, N, F> {
    soup: &'x Soup<N>,
    recursive: bool,
    filter: F,
}

/// Allows you to query for sub-elements matching the given [`Filter`](`crate::filter::Filter`)
pub trait QueryExt<'x, N, F>: Sized
where
    N: Node,
    F: Filter<N>,
{
    /// Forces the query to only match direct children of the root node
    fn strict(self) -> Query<'x, N, F>;

    /// Specifies a tag for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.tag("b").first().expect("Couldn't find tag 'b'");
    /// assert_eq!(result.get("id"), Some(&"bold-tag"));
    /// ```
    fn tag<T>(self, tag: T) -> Query<'x, N, And<F, Tag<T>>>
    where
        T: Pattern<N::Text>,
        Tag<T>: Filter<N>;

    /// Specifies an attribute name/value pair for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr("id", "bold-tag").first().expect("Couldn't find tag with id 'bold-tag'");
    /// assert_eq!(result.name(), Some(&"b"));
    fn attr<Q, V>(self, name: Q, value: V) -> Query<'x, N, And<F, Attr<Q, V>>>
    where
        Q: Pattern<N::Text>,
        V: Pattern<N::Text>,
        Attr<Q, V>: Filter<N>;

    /// Searches for a tag that has an attribute with the specified name
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr_name("id").first().expect("Couldn't find element with an 'id'");
    /// assert_eq!(result.name(), Some(&"b"));
    /// ```
    fn attr_name<Q>(self, name: Q) -> Query<'x, N, And<F, Attr<Q, bool>>>
    where
        Q: Pattern<N::Text>,
        Attr<Q, bool>: Filter<N>,
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
    /// assert_eq!(result.name(), Some(&"b"));
    /// ```
    fn attr_value<V>(self, value: V) -> Query<'x, N, And<F, Attr<bool, V>>>
    where
        V: Pattern<N::Text>,
        Attr<bool, V>: Filter<N>,
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
    /// assert_eq!(result.name(), Some(&"section"));
    fn class<C>(self, class: C) -> Query<'x, N, And<F, Attr<&'static str, C>>>
    where
        C: Pattern<N::Text>,
        N::Text: AsRef<str> + From<&'static str>,
        Attr<&'static str, C>: Filter<N>,
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
    /// assert_eq!(result.get("id"), Some(&"one"));
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
    /// assert_eq!(results[0].get("id"), Some(&"one"));
    /// assert_eq!(results[1].get("id"), Some(&"two"));
    /// assert_eq!(results[2].get("id"), Some(&"three"));
    /// ```
    fn all(self) -> Self::IntoIter
    where
        Self: IntoIterator,
    {
        self.into_iter()
    }
}

impl<'x, N, F> QueryExt<'x, N, F> for Query<'x, N, F>
where
    N: Node,
    F: Filter<N>,
{
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

impl<'x, N> QueryExt<'x, N, ()> for &'x Soup<N>
where
    N: Node,
{
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

impl<'x, N> std::ops::Deref for QueryItem<'x, N> {
    type Target = N;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

/// Type that can be turned into [`Soup`] and queried
pub trait Queryable<N> {
    /// Convert the type into one that can be queried
    fn query(self) -> Soup<N>;
}

impl<'x, N> Queryable<N> for &'x QueryItem<'x, N>
where
    N: Node + Clone,
{
    fn query(self) -> Soup<N> {
        Soup {
            nodes: self.item.children().to_vec(),
        }
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

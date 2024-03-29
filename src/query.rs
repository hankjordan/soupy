use std::marker::PhantomData;

use crate::{
    filter::{
        And,
        Attr,
        Filter,
        Tag,
    },
    parser::Parser,
    Pattern,
    Soup,
};

/// A query for elements in [`Soup`](`crate::Soup`) matching the [`Filter`](`crate::filter::Filter`) `F`
pub struct Query<'a, P: Parser<'a>, F: Filter<P::Node>> {
    pub filter: F,
    pub soup: &'a Soup<'a, P>,
}

/// Allows you to query for sub-elements matching the given [`Filter`](`crate::filter::Filter`)
pub trait QueryExt<'a, P: Parser<'a>, F: Filter<P::Node>>: Sized {
    /// Specifies a tag for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::new(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.tag("b").first().expect("Couldn't find tag 'b'");
    /// assert_eq!(result.get("id"), Some("bold-tag"));
    /// ```
    fn tag<T: Pattern<P::Text>>(self, tag: T) -> Query<'a, P, And<F, Tag<T>>>
    where
        Tag<T>: Filter<P::Node>;

    /// Specifies an attribute name/value pair for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::new(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr("id", "bold-tag").first().expect("Couldn't find tag with id 'bold-tag'");
    /// assert_eq!(result.name(), Some("b"));
    fn attr<N: Pattern<P::Text>, V: Pattern<P::Text>>(self, name: N, value: V) -> Query<'a, P, And<F, Attr<N, V>>>
    where
        Attr<N, V>: Filter<P::Node>;

    /// Searches for a tag that has an attribute with the specified name
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::new(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr_name("id").first().expect("Couldn't find element with an 'id'");
    /// assert_eq!(result.name(), Some("b"));
    /// ```
    fn attr_name<N: Pattern<P::Text>>(self, name: N) -> Query<'a, P, And<F, Attr<N, bool>>>
    where
        Attr<N, bool>: Filter<P::Node>,
    {
        self.attr(name, true)
    }

    /// Search for a node with any attribute with a value that matches the specified value
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::new(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr_value("bold-tag").first().expect("Couldn't find a tag with attribute value 'bold-tag'");
    /// assert_eq!(result.name(), Some("b"));
    /// ```
    fn attr_value<V: Pattern<P::Text>>(self, value: V) -> Query<'a, P, And<F, Attr<bool, V>>>
    where
        Attr<bool, V>: Filter<P::Node>,
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
    /// let soup = Soup::new(r#"<div>Test</div><section class="content"><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.class("content").first().expect("Couldn't find tag with class 'content'");
    /// assert_eq!(result.name(), Some("section"));
    fn class<C: Pattern<P::Text>>(self, class: C) -> Query<'a, P, And<F, Attr<&'static str, C>>>
    where
        P::Text: AsRef<str> + From<&'static str>,
        Attr<&'static str, C>: Filter<P::Node>,
    {
        self.attr("class", class)
    }

    /// Executes the query, and returns either the first result, or `None`
    ///
    /// Equivalent to calling `self.into_iter().next()`
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::new(r#"<ul><li id="one">One</li><li id="two">Two</li><li id="three">Three</li></ul>"#).unwrap();
    /// let result = soup.tag("li").first().expect("Couldn't find 'li'");
    /// assert_eq!(result.get("id"), Some("one"));
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
    /// let soup = Soup::new(r#"<ul><li id="one">One</li><li id="two">Two</li><li id="three">Three</li></ul>"#).unwrap();
    /// let results = soup.tag("li").all().collect::<Vec<_>>();
    /// assert_eq!(results.len(), 3);
    /// assert_eq!(results[0].get("id"), Some("one"));
    /// assert_eq!(results[1].get("id"), Some("two"));
    /// assert_eq!(results[2].get("id"), Some("three"));
    /// ```
    fn all(self) -> Self::IntoIter
    where
        Self: IntoIterator,
    {
        self.into_iter()
    }
}

impl<'a, P: Parser<'a>, F: Filter<P::Node>> QueryExt<'a, P, F> for Query<'a, P, F> {
    fn tag<T: Pattern<P::Text>>(self, tag: T) -> Query<'a, P, And<F, Tag<T>>>
    where
        Tag<T>: Filter<P::Node>,
    {
        Query {
            filter: And(self.filter, Tag { tag }),
            soup: self.soup,
        }
    }

    fn attr<N: Pattern<P::Text>, V: Pattern<P::Text>>(self, name: N, value: V) -> Query<'a, P, And<F, Attr<N, V>>>
    where
        Attr<N, V>: Filter<P::Node>,
    {
        Query {
            filter: And(self.filter, Attr { name, value }),
            soup: self.soup,
        }
    }
}

/// An [`Iterator`] over matching elements
pub struct QueryIter<'a, I, P: Parser<'a>, F: Filter<P::Node>> {
    pub(crate) filter: F,
    pub(crate) iter: I,
    pub(crate) _marker: PhantomData<&'a P>,
}

impl<'a, I, P: Parser<'a>, F: Filter<P::Node>> Iterator for QueryIter<'a, I, P, F>
where
    I: Iterator<Item = &'a P::Node>,
    &'a P::Node: IntoIterator<Item = &'a P::Node>,
{
    type Item = &'a P::Node;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next()?;
            if self.filter.matches(next) {
                return Some(next);
            }
        }
    }
}

impl<'a, P: Parser<'a>, F: Filter<P::Node>> IntoIterator for Query<'a, P, F>
where
    &'a P::Node: IntoIterator<Item = &'a P::Node>,
{
    type Item = &'a P::Node;
    type IntoIter = QueryIter<'a, std::iter::Flatten<std::slice::Iter<'a, P::Node>>, P, F>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter {
            filter: self.filter,
            iter: self.soup.nodes.iter().flatten(),
            _marker: PhantomData,
        }
    }
}

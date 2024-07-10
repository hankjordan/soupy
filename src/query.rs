use std::marker::PhantomData;

use crate::{
    filter::{
        And,
        Attr,
        Filter,
        Tag,
    },
    parser::Parser,
    Node,
    Pattern,
    Soup,
};

/// A query for elements in [`Soup`](`crate::Soup`) matching the [`Filter`](`crate::filter::Filter`) `F`
pub struct Query<'x, 'a, P: Parser<'a>, F: Filter<P::Node>> {
    pub filter: F,
    pub soup: &'x Soup<'a, P>,
}

/// Allows you to query for sub-elements matching the given [`Filter`](`crate::filter::Filter`)
pub trait QueryExt<'x, 'a, P: Parser<'a>, F: Filter<P::Node>>: Sized {
    /// Specifies a tag for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.tag("b").first().expect("Couldn't find tag 'b'");
    /// assert_eq!(result.get("id"), Some(&"bold-tag"));
    /// ```
    fn tag<T>(self, tag: T) -> Query<'x, 'a, P, And<F, Tag<T>>>
    where
        T: Pattern<<P::Node as Node>::Text>,
        Tag<T>: Filter<P::Node>;

    /// Specifies an attribute name/value pair for which to search
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr("id", "bold-tag").first().expect("Couldn't find tag with id 'bold-tag'");
    /// assert_eq!(result.name(), Some(&"b"));
    fn attr<N, V>(self, name: N, value: V) -> Query<'x, 'a, P, And<F, Attr<N, V>>>
    where
        N: Pattern<<P::Node as Node>::Text>,
        V: Pattern<<P::Node as Node>::Text>,
        Attr<N, V>: Filter<P::Node>;

    /// Searches for a tag that has an attribute with the specified name
    ///
    /// # Example
    /// ```rust
    /// # use soupy::prelude::*;
    /// let soup = Soup::html_strict(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.attr_name("id").first().expect("Couldn't find element with an 'id'");
    /// assert_eq!(result.name(), Some(&"b"));
    /// ```
    fn attr_name<N>(self, name: N) -> Query<'x, 'a, P, And<F, Attr<N, bool>>>
    where
        N: Pattern<<P::Node as Node>::Text>,
        Attr<N, bool>: Filter<P::Node>,
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
    fn attr_value<V>(self, value: V) -> Query<'x, 'a, P, And<F, Attr<bool, V>>>
    where
        V: Pattern<<P::Node as Node>::Text>,
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
    /// let soup = Soup::html_strict(r#"<div>Test</div><section class="content"><b id="bold-tag">SOME BOLD TEXT</b></section>"#).unwrap();
    /// let result = soup.class("content").first().expect("Couldn't find tag with class 'content'");
    /// assert_eq!(result.name(), Some(&"section"));
    fn class<C>(self, class: C) -> Query<'x, 'a, P, And<F, Attr<&'static str, C>>>
    where
        C: Pattern<<P::Node as Node>::Text>,
        <P::Node as Node>::Text: AsRef<str> + From<&'static str>,
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

impl<'x, 'a, P, F> QueryExt<'x, 'a, P, F> for Query<'x, 'a, P, F>
where
    P: Parser<'a>,
    F: Filter<P::Node>,
{
    fn tag<T>(self, tag: T) -> Query<'x, 'a, P, And<F, Tag<T>>>
    where
        T: Pattern<<P::Node as Node>::Text>,
        Tag<T>: Filter<P::Node>,
    {
        Query {
            filter: And(self.filter, Tag { tag }),
            soup: self.soup,
        }
    }

    fn attr<N, V>(self, name: N, value: V) -> Query<'x, 'a, P, And<F, Attr<N, V>>>
    where
        N: Pattern<<P::Node as Node>::Text>,
        V: Pattern<<P::Node as Node>::Text>,
        Attr<N, V>: Filter<P::Node>,
    {
        Query {
            filter: And(self.filter, Attr { name, value }),
            soup: self.soup,
        }
    }
}

impl<'x, 'a, P: Parser<'a>> QueryExt<'x, 'a, P, ()> for &'x Soup<'a, P> {
    fn tag<T>(self, tag: T) -> Query<'x, 'a, P, And<(), Tag<T>>>
    where
        T: Pattern<<P::Node as Node>::Text>,
        Tag<T>: Filter<P::Node>,
    {
        Query {
            filter: And((), Tag { tag }),
            soup: self,
        }
    }

    fn attr<N, V>(self, name: N, value: V) -> Query<'x, 'a, P, And<(), Attr<N, V>>>
    where
        N: Pattern<<P::Node as Node>::Text>,
        V: Pattern<<P::Node as Node>::Text>,
        Attr<N, V>: Filter<P::Node>,
    {
        Query {
            filter: And((), Attr { name, value }),
            soup: self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryItem<'x, 'a, P: Parser<'a>> {
    item: &'x P::Node,
}

impl<'x, 'a, P> std::ops::Deref for QueryItem<'x, 'a, P>
where
    P: Parser<'a>,
{
    type Target = P::Node;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

pub trait Queryable<'a, P>
where
    P: Parser<'a>,
{
    fn query(self) -> Soup<'a, P>;
}

impl<'x, 'a, P> Queryable<'a, P> for &'x QueryItem<'x, 'a, P>
where
    P: Parser<'a>,
    P::Node: Clone,
    &'x P::Node: IntoIterator<Item = &'x P::Node>,
{
    fn query(self) -> Soup<'a, P> {
        Soup {
            nodes: self.item.into_iter().cloned().collect(),
            _marker: PhantomData,
        }
    }
}

/// An [`Iterator`] over matching elements
pub struct QueryIter<'x, 'a, I, P: Parser<'a>, F: Filter<P::Node>> {
    pub(crate) filter: F,
    pub(crate) iter: I,
    pub(crate) _marker: PhantomData<(&'x (), &'a (), P)>,
}

impl<'x, 'a, I, P, F> Iterator for QueryIter<'x, 'a, I, P, F>
where
    I: Iterator<Item = &'x P::Node>,
    P: Parser<'a>,
    P::Node: 'x,
    &'x P::Node: IntoIterator<Item = &'x P::Node>,
    F: Filter<P::Node>,
{
    type Item = QueryItem<'x, 'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next()?;
            if self.filter.matches(next) {
                return Some(QueryItem { item: next });
            }
        }
    }
}

impl<'x, 'a, P, F> IntoIterator for Query<'x, 'a, P, F>
where
    P: Parser<'a>,
    &'x P::Node: IntoIterator<Item = &'x P::Node>,
    F: Filter<P::Node>,
{
    type Item = QueryItem<'x, 'a, P>;
    type IntoIter = QueryIter<'x, 'a, std::iter::Flatten<std::slice::Iter<'x, P::Node>>, P, F>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter {
            filter: self.filter,
            iter: self.soup.nodes.iter().flatten(),
            _marker: PhantomData,
        }
    }
}

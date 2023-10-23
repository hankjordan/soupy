use std::marker::PhantomData;

use crate::{
    filter::{
        And,
        Attr,
        Filter,
        Tag,
    },
    parser::{
        Parser,
        StrictHTMLParser,
    },
    query::{
        Query,
        QueryIter,
    },
    Pattern,
    QueryExt,
};

/// Parsed nodes
#[derive(Clone, Debug)]
pub struct Soup<'a, P: Parser<'a>> {
    pub(crate) nodes: Vec<P::Node>,
    _marker: PhantomData<P>,
}

impl<'a> Soup<'a, StrictHTMLParser> {
    /// Attempts to create a new `Soup` instance from a string slice.
    ///
    /// # Errors
    /// If the text is invalid HTML.
    pub fn new(text: &'a str) -> Result<Self, <StrictHTMLParser as Parser>::Error> {
        Ok(Self {
            nodes: StrictHTMLParser::parse(text)?,
            _marker: PhantomData,
        })
    }
}

impl<'a, P: Parser<'a>> Soup<'a, P> {
    /// Attempts use the [`Parser`] to create a new `Soup` instance from a string slice.
    ///
    /// # Errors
    /// If the text has an invalid format.
    pub fn with_parser(text: &'a str) -> Result<Soup<'a, P>, P::Error> {
        Ok(Soup {
            nodes: P::parse(text)?,
            _marker: PhantomData,
        })
    }
}

impl<'a, P: Parser<'a>> QueryExt<'a, P, ()> for &'a Soup<'a, P> {
    fn tag<T: Pattern>(self, tag: T) -> Query<'a, P, And<(), Tag<T>>>
    where
        Tag<T>: Filter<P::Node>,
    {
        Query {
            filter: And((), Tag { tag }),
            soup: self,
        }
    }

    fn attr<N: Pattern, V: Pattern>(self, name: N, value: V) -> Query<'a, P, And<(), Attr<N, V>>>
    where
        Attr<N, V>: Filter<P::Node>,
    {
        Query {
            filter: And((), Attr { name, value }),
            soup: self,
        }
    }
}

impl<'a, P: Parser<'a>> IntoIterator for &'a Soup<'a, P>
where
    &'a P::Node: IntoIterator<Item = &'a P::Node>,
{
    type Item = &'a P::Node;
    type IntoIter = QueryIter<'a, std::iter::Flatten<std::slice::Iter<'a, P::Node>>, P, ()>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter {
            filter: (),
            iter: self.nodes.iter().flatten(),
            _marker: PhantomData,
        }
    }
}

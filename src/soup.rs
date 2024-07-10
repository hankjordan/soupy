use std::marker::PhantomData;

use crate::{
    parser::Parser,
    query::QueryIter,
};

/// Parsed nodes
#[derive(Clone, Debug)]
pub struct Soup<'a, P: Parser<'a>> {
    pub(crate) nodes: Vec<P::Node>,
    _marker: PhantomData<P>,
}

#[cfg(feature = "html-strict")]
impl<'a> Soup<'a, crate::parser::StrictHTMLParser> {
    /// Attempts to create a new `Soup` instance from a string slice.
    ///
    /// # Errors
    /// If the text is invalid HTML.
    pub fn html_strict(
        text: &'a str,
    ) -> Result<Self, <crate::parser::StrictHTMLParser as Parser>::Error> {
        Self::new(text)
    }
}

#[cfg(feature = "html-lenient")]
impl<'a> Soup<'a, crate::parser::LenientHTMLParser> {
    /// Creates a new `Soup` instance from a string slice.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn html(text: &'a str) -> Self {
        Self::new(text).unwrap()
    }
}

#[cfg(feature = "xml")]
impl<'a> Soup<'a, crate::parser::XMLParser> {
    /// Creates a new `Soup` instance from a string slice.
    ///
    /// # Errors
    /// If the text is invalid XML.
    #[allow(clippy::missing_panics_doc)]
    pub fn xml(text: &'a str) -> Result<Self, <crate::parser::XMLParser as Parser>::Error> {
        Self::new(text)
    }
}

impl<'a, P: Parser<'a>> Soup<'a, P> {
    /// Attempts use the [`Parser`] to create a new `Soup` instance from a string slice.
    ///
    /// # Errors
    /// If the text has an invalid format.
    pub fn new(text: &'a str) -> Result<Soup<'a, P>, P::Error> {
        Ok(Soup {
            nodes: P::parse(text)?,
            _marker: PhantomData,
        })
    }
}

impl<'a, P> Soup<'a, P>
where
    P: Parser<'a>,
    &'a P::Node: IntoIterator<Item = &'a P::Node>,
{
    /// Query the data.
    #[must_use]
    pub fn iter(&'a self) -> QueryIter<std::iter::Flatten<std::slice::Iter<P::Node>>, P, ()> {
        QueryIter {
            filter: (),
            iter: self.nodes.iter().flatten(),
            _marker: PhantomData,
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
        self.iter()
    }
}

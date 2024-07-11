use std::marker::PhantomData;

use crate::{
    parser::Parser,
    query::{
        QueryItem,
        QueryIter,
    },
};

/// Parsed nodes
#[derive(Clone, Debug)]
pub struct Soup<P: Parser> {
    pub(crate) nodes: Vec<P::Node>,
    pub(crate) _marker: PhantomData<P>,
}

#[cfg(feature = "html-strict")]
impl<'a> Soup<crate::parser::StrictHTMLParser<'a>> {
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
impl<S> Soup<crate::parser::LenientHTMLParser<S>>
where
    S: AsRef<str>,
{
    /// Creates a new `Soup` instance from a string slice.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn html(text: S) -> Self {
        Self::new(text).unwrap()
    }
}

#[cfg(feature = "xml")]
impl<R> Soup<crate::parser::XMLParser<R>>
where
    R: std::io::Read,
{
    /// Creates a new `Soup` instance from a reader.
    ///
    /// # Errors
    /// If the text is invalid XML.
    pub fn xml(reader: R) -> Result<Self, <crate::parser::XMLParser<R> as Parser>::Error> {
        Self::new(reader)
    }
}

impl<P: Parser> Soup<P> {
    /// Attempts use the [`Parser`] to create a new `Soup` instance from the input.
    ///
    /// # Errors
    /// If the text has an invalid format.
    pub fn new(input: P::Input) -> Result<Self, P::Error> {
        Ok(Soup {
            nodes: P::parse(input)?,
            _marker: PhantomData,
        })
    }
}

impl<'x, P> Soup<P>
where
    P: Parser,
    P::Node: 'x,
{
    /// Query the data.
    #[must_use]
    pub fn iter(&'x self) -> QueryIter<'x, std::slice::Iter<'x, P::Node>, P, ()> {
        QueryIter::new((), self.nodes.iter())
    }
}

impl<'x, P> IntoIterator for &'x Soup<P>
where
    P: Parser,
    P::Node: 'x,
{
    type Item = QueryItem<'x, P>;
    type IntoIter = QueryIter<'x, std::slice::Iter<'x, P::Node>, P, ()>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

use crate::{
    parser::Parser,
    query::{
        QueryItem,
        QueryIter,
    },
    Node,
};

/// Parsed nodes
#[derive(Clone, Debug)]
pub struct Soup<N = ()> {
    pub(crate) nodes: Vec<N>,
}

#[cfg(feature = "html-strict")]
impl Soup {
    /// Attempts to create a new `Soup` instance from a string slice.
    ///
    /// # Errors
    /// If the text is invalid HTML.
    pub fn html_strict(
        text: &str,
    ) -> Result<
        Soup<<crate::parser::StrictHTMLParser as Parser>::Node>,
        <crate::parser::StrictHTMLParser as Parser>::Error,
    > {
        Soup::new::<crate::parser::StrictHTMLParser>(text)
    }
}

#[cfg(feature = "html-lenient")]
impl Soup {
    /// Creates a new `Soup` instance from a string slice.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn html<S>(text: S) -> Soup<<crate::parser::LenientHTMLParser<S> as Parser>::Node>
    where
        S: AsRef<str>,
    {
        Soup::new::<crate::parser::LenientHTMLParser<S>>(text).unwrap()
    }
}

#[cfg(feature = "xml")]
impl Soup {
    /// Creates a new `Soup` instance from a reader.
    ///
    /// # Errors
    /// If the text is invalid XML.
    pub fn xml<R: std::io::Read>(
        reader: R,
    ) -> Result<
        Soup<<crate::parser::XMLParser<R> as Parser>::Node>,
        <crate::parser::XMLParser<R> as Parser>::Error,
    > {
        Soup::new::<crate::parser::XMLParser<R>>(reader)
    }
}

impl Soup {
    /// Attempts use the [`Parser`] to create a new `Soup` instance from the input.
    ///
    /// # Errors
    /// If the text has an invalid format.
    pub fn new<P: Parser>(input: P::Input) -> Result<Soup<P::Node>, P::Error> {
        Ok(Soup {
            nodes: P::parse(input)?,
        })
    }
}

impl<N> Soup<N>
where
    N: Node,
{
    /// Query the data.
    #[must_use]
    pub fn iter(&self) -> QueryIter<N, ()> {
        QueryIter::new(&self.nodes, true, ())
    }
}

impl<'x, N> IntoIterator for &'x Soup<N>
where
    N: Node,
{
    type Item = QueryItem<'x, N>;
    type IntoIter = QueryIter<'x, N, ()>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

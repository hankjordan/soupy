use std::collections::BTreeMap;

pub trait NodeIterable {}

impl<T> NodeIterable for T where for<'a> &'a T: IntoIterator<Item = &'a T> {}

pub trait Node: NodeIterable {
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
}

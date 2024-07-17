/// A trait used to indicate a type which can be used to match a value
///
/// Any type that implements this trait can be passed to the various
/// [`QueryExt`](crate::QueryExt) methods in order to match an element
///
/// # Example
/// ```rust
/// use soupy::prelude::*;
///
/// struct MyType(String);
///
/// impl<'a, S> Pattern<S> for MyType
/// where
///     S: AsRef<str> + From<&'a str>,
/// {
///     fn matches(&self, haystack: &S) -> bool {
///         haystack.as_ref() == self.0
///     }
/// }
///
/// let soup = Soup::html_strict(r#"<div id="foo"></div>"#).unwrap();
/// let result = soup.tag(MyType("div".to_string())).first().expect("Couldn't find div with id foo");
/// assert_eq!(result.get("id"), Some(&"foo"));
/// ```
pub trait Pattern<S> {
    /// Matches the `Pattern` with the value `haystack`
    fn matches(&self, haystack: &S) -> bool;

    /// Convert the pattern into the haystack's type
    ///
    /// Used for optimization.
    fn value(&self) -> Option<S> {
        None
    }
}

impl<S> Pattern<S> for bool {
    fn matches(&self, _haystack: &S) -> bool {
        *self
    }
}

impl<'a, S> Pattern<S> for &'a str
where
    S: AsRef<str> + From<&'a str>,
{
    fn matches(&self, haystack: &S) -> bool {
        &haystack.as_ref() == self
    }

    fn value(&self) -> Option<S> {
        Some((*self).into())
    }
}

impl<S> Pattern<S> for String
where
    S: AsRef<str> + for<'a> From<&'a str>,
{
    fn matches(&self, haystack: &S) -> bool {
        *self == haystack.as_ref()
    }

    fn value(&self) -> Option<S> {
        Some(self.as_str().into())
    }
}

#[cfg(feature = "regex")]
impl<S> Pattern<S> for regex::Regex
where
    S: AsRef<str>,
{
    fn matches(&self, haystack: &S) -> bool {
        self.is_match(haystack.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    const HELLO: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<root>
    <simple>Here's some text</simple>
    <complex id="hello">
        <nested>Nested text!</nested>
        <example>More text</example>

        <tree depth="1">
            <tree depth="2">
                <tree depth="3">Tree text</tree>
            </tree>
        </tree>
    </complex>

    <b>
        <a>Inner text</a>
    </b>

    <a>Outer text</a>
</root>"#;

    #[test]
    fn test_regex() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let regex = regex::Regex::new("e$").expect("Failed to compile regex");

        let mut tags = soup.tag(regex).all();

        assert_eq!(
            tags.next().and_then(|t| t.name().cloned()),
            Some("simple".into())
        );

        assert_eq!(
            tags.next().and_then(|t| t.name().cloned()),
            Some("example".into())
        );

        assert_eq!(
            tags.next().and_then(|t| t.name().cloned()),
            Some("tree".into())
        );
    }

    #[test]
    fn test_bool() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let mut tags = soup.tag(true).all();

        assert_eq!(
            tags.next().and_then(|t| t.name().cloned()),
            Some("root".into())
        );
        assert_eq!(
            tags.next().and_then(|t| t.name().cloned()),
            Some("simple".into())
        );
        assert_eq!(
            tags.next().and_then(|t| t.name().cloned()),
            Some("complex".into())
        );

        let mut depth = soup.attr("depth", true).all();

        assert_eq!(
            depth.next().and_then(|t| t.name().cloned()),
            Some("tree".into())
        );
        assert_eq!(
            depth.next().and_then(|t| t.name().cloned()),
            Some("tree".into())
        );
        assert_eq!(
            depth.next().and_then(|t| t.name().cloned()),
            Some("tree".into())
        );
        assert_eq!(depth.next().and_then(|t| t.name().cloned()), None);
    }

    #[test]
    fn test_string() {
        let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

        let mut tags = soup.tag("simple".to_string()).all();

        assert_eq!(
            tags.next().map(|t| t.all_text()),
            Some("Here's some text".into())
        );
        assert_eq!(tags.next().map(|t| t.all_text()), None);
    }
}

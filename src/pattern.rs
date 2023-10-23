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
/// impl Pattern for MyType {
///     fn matches(&self, haystack: &str) -> bool {
///         self.0.matches(haystack)
///     }
/// }
///
/// let soup = Soup::new(r#"<div id="foo"></div>"#).unwrap();
/// let result = soup.tag(MyType("div".to_string())).first().expect("Couldn't find div with id foo");
/// assert_eq!(result.get("id"), Some("foo"));
/// ```
pub trait Pattern {
    /// Matches the `Pattern` with the value `haystack`
    fn matches(&self, _haystack: &str) -> bool;

    #[doc(hidden)]
    fn as_bool(&self) -> Option<bool> {
        None
    }

    #[doc(hidden)]
    fn as_str(&self) -> Option<&str> {
        None
    }
}

impl Pattern for bool {
    fn matches(&self, _haystack: &str) -> bool {
        *self
    }

    fn as_bool(&self) -> Option<bool> {
        Some(*self)
    }
}

impl<'a> Pattern for &'a str {
    fn matches(&self, haystack: &str) -> bool {
        *self == haystack
    }

    fn as_str(&self) -> Option<&str> {
        Some(self)
    }
}

impl Pattern for String {
    fn matches(&self, haystack: &str) -> bool {
        *self == haystack
    }

    fn as_str(&self) -> Option<&str> {
        Some(self)
    }
}

#[cfg(feature = "regex")]
impl Pattern for regex::Regex {
    fn matches(&self, haystack: &str) -> bool {
        self.is_match(haystack)
    }
}

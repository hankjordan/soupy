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

    /// If `Some`, skip the match and return the value
    fn bypass(&self) -> Option<bool> {
        None
    }

    /// Convert the pattern into the haystack's type
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

// impl<S> Pattern<S> for String
// where
//     S: AsRef<str> + From<String>,
// {
//     fn matches(&self, haystack: &S) -> bool {
//         *self == haystack
//     }

//     fn value(&self) -> Option<S> {
//         Some(self)
//     }
// }

// #[cfg(feature = "regex")]
// impl Pattern for regex::Regex {
//     fn matches(&self, haystack: &str) -> bool {
//         self.is_match(haystack)
//     }
// }

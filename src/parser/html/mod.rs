#[cfg(feature = "html-lenient")]
mod lenient;
#[cfg(any(feature = "html-lenient", feature = "html-strict"))]
mod node;
#[cfg(feature = "html-strict")]
mod strict;

#[cfg(feature = "html-lenient")]
pub use lenient::LenientHTMLParser;
#[cfg(any(feature = "html-lenient", feature = "html-strict"))]
pub use node::HTMLNode;
#[cfg(feature = "html-strict")]
pub use strict::StrictHTMLParser;

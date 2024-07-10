#[cfg(feature = "html-lenient")]
mod lenient;
#[cfg(feature = "html")]
mod node;
#[cfg(feature = "html-strict")]
mod strict;

#[cfg(feature = "html-lenient")]
pub use lenient::LenientHTMLParser;
#[cfg(feature = "html")]
pub use node::HTMLNode;
#[cfg(feature = "html-strict")]
pub use strict::StrictHTMLParser;

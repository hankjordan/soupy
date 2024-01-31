#[cfg(feature = "lenient")]
mod lenient;
mod strict;

#[cfg(feature = "lenient")]
pub use lenient::LenientHTMLParser;
pub use strict::StrictHTMLParser;

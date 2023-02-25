mod error;
mod location;
#[cfg(feature = "benchmarking")]
pub mod reader;
#[cfg(not(feature = "benchmarking"))]
mod reader;
mod span;

pub use error::*;
pub use location::Location;
pub use reader::parse;
pub use span::Span;

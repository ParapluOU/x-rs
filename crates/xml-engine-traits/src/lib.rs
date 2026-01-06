//! Core trait abstractions for XML processing engines.
//!
//! This crate defines the fundamental traits that XML processing engines
//! must implement to be used in the unified test harness.

pub mod error;
pub mod tree;
pub mod xpath;
pub mod xquery;
pub mod xslt;

pub use error::Error;
pub use tree::{NodeType, XmlTree};
pub use xpath::XPathEngine;
pub use xquery::XQueryEngine;
pub use xslt::XsltEngine;

/// Trait for engines that support multiple XML processing capabilities
pub trait UnifiedEngine:
    XPathEngine<Tree = Self::TreeType>
    + XsltEngine<Tree = Self::TreeType>
    + XQueryEngine<Tree = Self::TreeType>
{
    type TreeType: XmlTree;

    fn engine_name(&self) -> &'static str;
    fn engine_version(&self) -> &'static str;
}

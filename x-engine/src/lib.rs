//! x-engine: Unified XML/XPath/XSLT/XQuery engine wrapper
//!
//! This library provides common traits for XML processing engines,
//! enabling unified conformance testing against W3C test suites.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use x_engine::{XEngine, Backend};
//!
//! let mut engine = XEngine::with_backend(Backend::Xee);
//! let doc = engine.parse("<root><item>Hello</item></root>")?;
//! let result = engine.xpath(&doc, "//item/text()")?;
//! println!("{}", result.to_string());
//! ```

pub mod error;
pub mod result;
pub mod traits;

pub mod engine_xee;
pub mod engine_xrust;
pub mod engine_xust;

pub mod unified;
pub mod testdriver;
pub mod reporter;

// Re-export core types
pub use error::Error;
pub use result::{NodeType, ResultItem, ValidationResult};
pub use traits::{QueryResult, XmlDocument, XmlParser, XPathEngine, XQueryEngine, XsdValidator, XsltEngine};

// Re-export unified API
pub use unified::{Backend, XDocument, XEngine, XQueryResult};

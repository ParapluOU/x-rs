//! xee engine adapter for the unified XML engine abstraction
//!
//! This adapter wraps the xee XPath 3.1 engine to implement the
//! xml-engine-traits interfaces.

pub mod tree;
pub mod xpath;

// Re-export main types
pub use tree::XotTreeWrapper;
pub use xpath::XeeEngine;

// Re-export key types for convenience
pub use xee_interpreter::xml::DocumentHandle;
pub use xee_xpath::{Documents, Queries};
pub use xot::Node;

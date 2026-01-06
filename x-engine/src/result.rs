//! Result types for x-engine

use serde::{Deserialize, Serialize};

/// XML node type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Document,
    Element,
    Attribute,
    Text,
    Comment,
    ProcessingInstruction,
    Namespace,
}

/// Information about an XML node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_type: NodeType,
    pub name: Option<String>,
    pub value: Option<String>,
}

/// A single item in a query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultItem {
    Node(NodeInfo),
    String(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
    Date(String),
    DateTime(String),
    Duration(String),
    QName(String),
    Empty,
}

impl ResultItem {
    pub fn as_string(&self) -> String {
        match self {
            ResultItem::Node(info) => info.value.clone().unwrap_or_default(),
            ResultItem::String(s) => s.clone(),
            ResultItem::Integer(i) => i.to_string(),
            ResultItem::Double(d) => d.to_string(),
            ResultItem::Boolean(b) => b.to_string(),
            ResultItem::Date(s) => s.clone(),
            ResultItem::DateTime(s) => s.clone(),
            ResultItem::Duration(s) => s.clone(),
            ResultItem::QName(s) => s.clone(),
            ResultItem::Empty => String::new(),
        }
    }
}

/// XSD validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

/// Result of XSD validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }
}

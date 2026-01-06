//! XML tree abstraction trait

use crate::error::Result;
use std::fmt::Debug;

/// Type of XML node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Document node
    Document,
    /// Element node
    Element,
    /// Attribute node
    Attribute,
    /// Text node
    Text,
    /// Comment node
    Comment,
    /// Processing instruction node
    ProcessingInstruction,
    /// Namespace node
    Namespace,
}

/// Trait for XML tree implementations.
///
/// This trait abstracts over different XML tree representations,
/// allowing engines with different internal tree structures to
/// be used interchangeably.
pub trait XmlTree: Send + Sync {
    /// Type representing a node handle in this tree
    type Node: Clone + Send + Sync + Debug;

    /// Type representing a document handle in this tree
    type Document: Clone + Send + Sync + Debug;

    /// Parse XML from a string and return a document handle
    fn parse_xml(&mut self, xml: &str) -> Result<Self::Document>;

    /// Parse XML from a string with a base URI
    fn parse_xml_with_uri(&mut self, uri: &str, xml: &str) -> Result<Self::Document>;

    /// Get the document element (root element) of a document
    fn document_element(&self, doc: &Self::Document) -> Result<Self::Node>;

    /// Get the parent of a node, if it has one
    fn parent(&self, node: &Self::Node) -> Option<Self::Node>;

    /// Get all children of a node
    fn children(&self, node: &Self::Node) -> Vec<Self::Node>;

    /// Get all attributes of an element node as (name, value) pairs
    fn attributes(&self, node: &Self::Node) -> Vec<(String, String)>;

    /// Get the qualified name of a node (if applicable)
    fn node_name(&self, node: &Self::Node) -> Option<String>;

    /// Get the local name of a node (without namespace prefix)
    fn node_local_name(&self, node: &Self::Node) -> Option<String>;

    /// Get the namespace URI of a node
    fn node_namespace_uri(&self, node: &Self::Node) -> Option<String>;

    /// Get the text content/value of a node
    fn node_value(&self, node: &Self::Node) -> Option<String>;

    /// Get the type of a node
    fn node_type(&self, node: &Self::Node) -> NodeType;

    /// Serialize a node to an XML string
    fn serialize(&self, node: &Self::Node) -> Result<String>;

    /// Serialize a document to an XML string
    fn serialize_document(&self, doc: &Self::Document) -> Result<String>;
}

/// Trait for XML trees that support XPath data model operations
pub trait XPathDataModel: XmlTree {
    /// Get the string value of a node (as defined by XPath)
    fn string_value(&self, node: &Self::Node) -> Result<String>;

    /// Check if two nodes are the same node
    fn is_same_node(&self, a: &Self::Node, b: &Self::Node) -> bool;

    /// Get the base URI of a node
    fn base_uri(&self, node: &Self::Node) -> Option<String>;

    /// Get the document URI of a document
    fn document_uri(&self, doc: &Self::Document) -> Option<String>;
}

/// Helper trait for trees that need mutable access
pub trait MutableXmlTree: XmlTree {
    /// Create a new element node
    fn create_element(&mut self, name: &str, namespace: Option<&str>) -> Result<Self::Node>;

    /// Create a new text node
    fn create_text(&mut self, text: &str) -> Result<Self::Node>;

    /// Append a child to a node
    fn append_child(&mut self, parent: &Self::Node, child: &Self::Node) -> Result<()>;

    /// Set an attribute on an element
    fn set_attribute(
        &mut self,
        element: &Self::Node,
        name: &str,
        value: &str,
        namespace: Option<&str>,
    ) -> Result<()>;
}

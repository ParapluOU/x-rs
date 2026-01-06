//! XmlTree implementation for xot

use xml_engine_traits::{
    error::{Error, Result},
    tree::{NodeType, XmlTree},
};
use xot::{Node, Xot};

/// Wrapper around Xot that implements XmlTree trait
#[derive(Debug)]
pub struct XotTreeWrapper {
    pub(crate) xot: Xot,
}

impl XotTreeWrapper {
    /// Create a new XotTreeWrapper
    pub fn new() -> Self {
        Self { xot: Xot::new() }
    }

    /// Get a reference to the underlying Xot
    pub fn xot(&self) -> &Xot {
        &self.xot
    }

    /// Get a mutable reference to the underlying Xot
    pub fn xot_mut(&mut self) -> &mut Xot {
        &mut self.xot
    }
}

impl Default for XotTreeWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlTree for XotTreeWrapper {
    type Node = Node;
    type Document = Node; // In xot, document is also a Node

    fn parse_xml(&mut self, xml: &str) -> Result<Self::Document> {
        self.xot
            .parse(xml)
            .map_err(|e| Error::XmlParse(e.to_string()))
    }

    fn parse_xml_with_uri(&mut self, _uri: &str, xml: &str) -> Result<Self::Document> {
        // xot doesn't have built-in URI support, just parse normally
        // URI handling is done at the Documents level in xee
        self.parse_xml(xml)
    }

    fn document_element(&self, doc: &Self::Document) -> Result<Self::Node> {
        self.xot
            .document_element(*doc)
            .ok_or_else(|| Error::NodeAccess("Document has no root element".to_string()))
    }

    fn parent(&self, node: &Self::Node) -> Option<Self::Node> {
        self.xot.parent(*node)
    }

    fn children(&self, node: &Self::Node) -> Vec<Self::Node> {
        self.xot.children(*node).collect()
    }

    fn attributes(&self, node: &Self::Node) -> Vec<(String, String)> {
        self.xot
            .attributes(*node)
            .map(|(name, value)| {
                let name_str = self.xot.name_string(name);
                let value_str = value.get().to_string();
                (name_str, value_str)
            })
            .collect()
    }

    fn node_name(&self, node: &Self::Node) -> Option<String> {
        match self.xot.value(*node) {
            xot::Value::Element(element) => Some(self.xot.name_string(element.name())),
            xot::Value::Attribute(attr) => Some(self.xot.name_string(attr.name())),
            xot::Value::Namespace(ns) => ns.prefix().map(|p| self.xot.prefix_string(p)),
            xot::Value::ProcessingInstruction(pi) => Some(pi.target().to_string()),
            _ => None,
        }
    }

    fn node_local_name(&self, node: &Self::Node) -> Option<String> {
        match self.xot.value(*node) {
            xot::Value::Element(element) => {
                Some(self.xot.name(*element.name()).local_name().to_string())
            }
            xot::Value::Attribute(attr) => {
                Some(self.xot.name(*attr.name()).local_name().to_string())
            }
            _ => None,
        }
    }

    fn node_namespace_uri(&self, node: &Self::Node) -> Option<String> {
        match self.xot.value(*node) {
            xot::Value::Element(element) => self
                .xot
                .namespace_for_name(*element.name())
                .map(|ns_id| self.xot.namespace_str(ns_id).to_string()),
            xot::Value::Attribute(attr) => self
                .xot
                .namespace_for_name(*attr.name())
                .map(|ns_id| self.xot.namespace_str(ns_id).to_string()),
            xot::Value::Namespace(ns) => Some(ns.uri().to_string()),
            _ => None,
        }
    }

    fn node_value(&self, node: &Self::Node) -> Option<String> {
        match self.xot.value(*node) {
            xot::Value::Text(text) => Some(text.get().to_string()),
            xot::Value::Comment(comment) => Some(comment.get().to_string()),
            xot::Value::ProcessingInstruction(pi) => Some(pi.data().to_string()),
            xot::Value::Attribute(attr) => Some(attr.value().get().to_string()),
            xot::Value::Element(_) => {
                // For elements, return the text content
                Some(self.xot.text_content(*node))
            }
            _ => None,
        }
    }

    fn node_type(&self, node: &Self::Node) -> NodeType {
        match self.xot.value(*node) {
            xot::Value::Document => NodeType::Document,
            xot::Value::Element(_) => NodeType::Element,
            xot::Value::Text(_) => NodeType::Text,
            xot::Value::Comment(_) => NodeType::Comment,
            xot::Value::ProcessingInstruction(_) => NodeType::ProcessingInstruction,
            xot::Value::Attribute(_) => NodeType::Attribute,
            xot::Value::Namespace(_) => NodeType::Namespace,
        }
    }

    fn serialize(&self, node: &Self::Node) -> Result<String> {
        self.xot
            .to_string(*node)
            .map_err(|e| Error::Other(format!("Serialization error: {}", e)))
    }

    fn serialize_document(&self, doc: &Self::Document) -> Result<String> {
        self.serialize(doc)
    }
}

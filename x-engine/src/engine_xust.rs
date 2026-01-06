//! xust engine wrapper
//!
//! Supports:
//! - XML parsing
//! - XPath 3.1 evaluation (via XQuery)
//! - XQuery 3.1 evaluation
//! - XSD 1.1 validation
//!
//! Does NOT support:
//! - XSLT

use std::path::Path;
use std::rc::Rc;

use xust_eval::eval::context::{default_tree_context_init, Context, GlobalContext};
use xust_eval::eval::eval_xquery;
use xust_eval::r#fn::function_definitions;
use xust_eval::xdm::{Item, Sequence};
use xust_grammar::{parse as parse_xquery, ParseInit};
use xust_tree::node::{Node, NodeKind};
use xust_tree::tree::Tree;
use xust_xml::read::parse_xml_from_bytes;
use xust_xsd::atomic::Atomic;
use xust_xsd::load_validator;
use xust_xsd::xsd_validator::XsdValidator as XustXsdValidator;

use crate::error::{Error, Result};
use crate::result::{NodeInfo, NodeType, ResultItem, ValidationError, ValidationResult};
use crate::traits::{
    QueryResult, XPathEngine, XPathVersion, XQueryEngine, XQueryVersion, XmlDocument, XmlParser,
    XsdValidator, XsdVersion, XsltEngine, XsltVersion,
};

type XustTree = Tree<Atomic>;

/// xust engine wrapper
pub struct XustEngine {
    validator: Option<XustXsdValidator>,
}

impl Default for XustEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl XustEngine {
    pub fn new() -> Self {
        Self { validator: None }
    }
}

/// Document handle for xust (wraps Tree<Atomic>)
pub struct XustDocument {
    tree: Rc<XustTree>,
}

impl XmlDocument for XustDocument {
    fn to_string(&self) -> Result<String> {
        use xust_xml::write::{tree_to_string, XmlOutputParameters};
        tree_to_string(&*self.tree, XmlOutputParameters::default())
            .map_err(|e| Error::EngineError(e.to_string()))
    }
}

/// Query result for xust
pub struct XustQueryResult {
    items: Vec<ResultItem>,
    string_repr: String,
}

impl QueryResult for XustQueryResult {
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn count(&self) -> usize {
        self.items.len()
    }

    fn to_string(&self) -> String {
        self.string_repr.clone()
    }

    fn to_xml(&self) -> Result<String> {
        Ok(self.string_repr.clone())
    }

    fn items(&self) -> Vec<ResultItem> {
        self.items.clone()
    }
}

impl XmlParser for XustEngine {
    type Document = XustDocument;

    fn parse(&mut self, xml: &str) -> Result<Self::Document> {
        let bytes = xml.as_bytes().to_vec();
        let tree: XustTree = parse_xml_from_bytes(bytes, None, None)
            .map_err(|e| Error::ParseError(e.to_string()))?;
        Ok(XustDocument {
            tree: Rc::new(tree),
        })
    }
}

impl XPathEngine for XustEngine {
    type QueryResult = XustQueryResult;

    fn evaluate_xpath(&mut self, doc: &Self::Document, xpath: &str) -> Result<Self::QueryResult> {
        // XPath in xust is evaluated through XQuery
        self.execute_xquery(doc, xpath)
    }

    fn xpath_version(&self) -> XPathVersion {
        XPathVersion::V3_1
    }
}

impl XQueryEngine for XustEngine {
    type QueryResult = XustQueryResult;

    fn execute_xquery(&mut self, doc: &Self::Document, xquery: &str) -> Result<Self::QueryResult> {
        use std::collections::HashMap;

        // Create function definitions first - these contain count(), etc.
        let fd = function_definitions();
        let empty_namespaces: HashMap<String, String> = HashMap::new();

        // Create parse init with function definitions
        let parse_init = ParseInit {
            fd: &fd,
            namespaces: &empty_namespaces,
            ..ParseInit::default()
        };

        // Parse the query
        let parsed_query = parse_xquery(xquery, parse_init)
            .map_err(|e| Error::XQueryError(format!("{:?}", e)))?;

        // Create context initialization using qnames from parsed query
        let context_init = default_tree_context_init(parsed_query.qnames().clone(), fd);

        // Create global context
        let global_context = GlobalContext::new(&context_init, parsed_query);

        // Create evaluation context
        let mut context =
            Context::new(global_context).map_err(|e| Error::XQueryError(format!("{:?}", e)))?;

        // Set context item to the document root
        let root = Node::root(doc.tree.clone());
        let context_item = Item::Node(root);
        context.set_only_item(&context_item);

        // Evaluate
        let sequence: Sequence<Rc<XustTree>> =
            eval_xquery(&mut context).map_err(|e| Error::XQueryError(format!("{:?}", e)))?;

        // Convert to our result types
        let mut items = Vec::new();
        let mut string_parts = Vec::new();

        for item in &sequence {
            match item {
                Item::Atomic(atomic) => {
                    // Use Display implementation
                    let s = atomic.to_string();
                    string_parts.push(s.clone());
                    items.push(ResultItem::String(s));
                }
                Item::Node(node) => {
                    let node_type = match node.node_kind() {
                        NodeKind::Document => NodeType::Document,
                        NodeKind::Element => NodeType::Element,
                        NodeKind::Text => NodeType::Text,
                        NodeKind::Comment => NodeType::Comment,
                        NodeKind::ProcessingInstruction => NodeType::ProcessingInstruction,
                        NodeKind::Attribute => NodeType::Attribute,
                        NodeKind::Namespace => NodeType::Namespace,
                    };
                    let name = node.node_name().map(|qn| format!("{}", qn));
                    // Use Debug for node value since Display isn't implemented
                    let value = Some(format!("{:?}", node));
                    string_parts.push(value.clone().unwrap_or_default());
                    items.push(ResultItem::Node(NodeInfo {
                        node_type,
                        name,
                        value,
                    }));
                }
                Item::Array(_) => {
                    items.push(ResultItem::String("<array>".to_string()));
                    string_parts.push("<array>".to_string());
                }
                Item::Map(_) => {
                    items.push(ResultItem::String("<map>".to_string()));
                    string_parts.push("<map>".to_string());
                }
                Item::Function(_) => {
                    items.push(ResultItem::String("<function>".to_string()));
                    string_parts.push("<function>".to_string());
                }
            }
        }

        Ok(XustQueryResult {
            items,
            string_repr: string_parts.join("\n"),
        })
    }

    fn xquery_version(&self) -> XQueryVersion {
        XQueryVersion::V3_1
    }
}

impl XsltEngine for XustEngine {
    fn transform(&mut self, _doc: &Self::Document, _stylesheet: &str) -> Result<Self::Document> {
        Err(Error::Unsupported)
    }

    fn xslt_version(&self) -> XsltVersion {
        XsltVersion::V3_0
    }
}

impl XsdValidator for XustEngine {
    fn load_schema(&mut self, xsd: &str) -> Result<()> {
        // Write XSD to a temp file and load it
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| Error::XsdError(format!("Failed to create temp file: {}", e)))?;
        temp_file
            .write_all(xsd.as_bytes())
            .map_err(|e| Error::XsdError(format!("Failed to write to temp file: {}", e)))?;

        let path = temp_file.path().to_path_buf();
        let validator = load_validator(&[path], None)
            .map_err(|e| Error::XsdError(format!("Failed to load schema: {}", e)))?;

        self.validator = Some(validator);
        Ok(())
    }

    fn load_schema_file(&mut self, path: &Path) -> Result<()> {
        let validator = load_validator(&[path.to_path_buf()], None)
            .map_err(|e| Error::XsdError(format!("Failed to load schema: {}", e)))?;

        self.validator = Some(validator);
        Ok(())
    }

    fn validate(&self, doc: &Self::Document) -> Result<ValidationResult> {
        let validator = self
            .validator
            .as_ref()
            .ok_or_else(|| Error::XsdError("No schema loaded".to_string()))?;

        // Re-serialize document for validation
        let xml_str = doc.to_string()?;
        let bytes = xml_str.as_bytes().to_vec();
        let (_, normalized_xml) = xust_xml::read::decode_bytes(bytes)
            .map_err(|e| Error::XsdError(format!("Failed to decode XML: {}", e)))?;

        match validator.validate_to_tree(&normalized_xml, None) {
            Ok(_) => Ok(ValidationResult::valid()),
            Err(e) => {
                let errors = vec![ValidationError {
                    message: e.to_string(),
                    line: None,
                    column: None,
                }];
                Ok(ValidationResult::invalid(errors))
            }
        }
    }

    fn xsd_version(&self) -> XsdVersion {
        XsdVersion::V1_1
    }
}

//! xee engine wrapper
//!
//! Supports:
//! - XML parsing via xot
//! - XPath 3.1 evaluation
//! - XSLT 3.0 (partial)
//!
//! Does NOT support:
//! - XQuery
//! - XSD validation

use std::path::Path;

use crate::error::{Error, Result};
use crate::result::{NodeInfo, NodeType, ResultItem, ValidationResult};
use crate::traits::{
    QueryResult, XPathEngine, XPathVersion, XQueryEngine, XQueryVersion, XmlDocument, XmlParser,
    XsdValidator, XsdVersion, XsltEngine, XsltVersion,
};

/// xee engine wrapper
pub struct XeeEngine {
    xot: xot::Xot,
}

impl Default for XeeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl XeeEngine {
    pub fn new() -> Self {
        Self { xot: xot::Xot::new() }
    }
}

/// Document handle for xee (wraps xot::Node)
pub struct XeeDocument {
    root: xot::Node,
}

impl XmlDocument for XeeDocument {
    fn to_string(&self) -> Result<String> {
        // We need access to Xot to serialize, but we only have the node
        // This is a limitation - we'd need to store xot reference
        // For now, return an error indicating this limitation
        Err(Error::EngineError(
            "XeeDocument::to_string requires Xot context".to_string(),
        ))
    }
}

/// Query result for xee
pub struct XeeQueryResult {
    items: Vec<ResultItem>,
    string_repr: String,
}

impl QueryResult for XeeQueryResult {
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
        // Return string representation for non-node results
        Ok(self.string_repr.clone())
    }

    fn items(&self) -> Vec<ResultItem> {
        self.items.clone()
    }
}

impl XmlParser for XeeEngine {
    type Document = XeeDocument;

    fn parse(&mut self, xml: &str) -> Result<Self::Document> {
        let root = self
            .xot
            .parse(xml)
            .map_err(|e| Error::ParseError(e.to_string()))?;
        Ok(XeeDocument { root })
    }
}

impl XPathEngine for XeeEngine {
    type QueryResult = XeeQueryResult;

    fn evaluate_xpath(
        &mut self,
        doc: &Self::Document,
        xpath: &str,
    ) -> Result<Self::QueryResult> {
        use xee_xpath::{Documents, Queries, Query};

        let mut documents = Documents::new();

        // Add the document from our Xot instance
        // We need to serialize and re-parse because Documents manages its own Xot
        let xml_str = self
            .xot
            .to_string(doc.root)
            .map_err(|e| Error::EngineError(e.to_string()))?;

        let doc_handle = documents
            .add_string_without_uri(&xml_str)
            .map_err(|e| Error::XPathError(format!("{:?}", e)))?;

        let queries = Queries::default();
        let query = queries
            .sequence(xpath)
            .map_err(|e| Error::XPathError(format!("{:?}", e)))?;

        let sequence = query
            .execute(&mut documents, doc_handle)
            .map_err(|e| Error::XPathError(format!("{:?}", e)))?;

        // Convert sequence to our result types
        let mut items = Vec::new();
        let mut string_parts = Vec::new();

        for item in sequence.iter() {
            match item {
                xee_xpath::Item::Atomic(atomic) => {
                    let result_item = convert_atomic_to_result_item(&atomic);
                    string_parts.push(result_item.as_string());
                    items.push(result_item);
                }
                xee_xpath::Item::Node(node) => {
                    let xot = documents.xot();
                    let node_type = match xot.value_type(node) {
                        xot::ValueType::Document => NodeType::Document,
                        xot::ValueType::Element => NodeType::Element,
                        xot::ValueType::Text => NodeType::Text,
                        xot::ValueType::Comment => NodeType::Comment,
                        xot::ValueType::ProcessingInstruction => {
                            NodeType::ProcessingInstruction
                        }
                        xot::ValueType::Attribute => NodeType::Attribute,
                        xot::ValueType::Namespace => NodeType::Namespace,
                    };
                    let name = xot.node_name(node).map(|n| xot.name_ns_str(n).1.to_string());
                    let value = xot.to_string(node).ok();
                    string_parts.push(value.clone().unwrap_or_default());
                    items.push(ResultItem::Node(NodeInfo {
                        node_type,
                        name,
                        value,
                    }));
                }
                xee_xpath::Item::Function(_) => {
                    items.push(ResultItem::String("<function>".to_string()));
                    string_parts.push("<function>".to_string());
                }
            }
        }

        Ok(XeeQueryResult {
            items,
            string_repr: string_parts.join("\n"),
        })
    }

    fn xpath_version(&self) -> XPathVersion {
        XPathVersion::V3_1
    }
}

fn convert_atomic_to_result_item(atomic: &xee_xpath::Atomic) -> ResultItem {
    use xee_xpath::Atomic;
    match atomic {
        Atomic::String(_, s) => ResultItem::String(s.to_string()),
        Atomic::Untyped(s) => ResultItem::String(s.to_string()),
        Atomic::Boolean(b) => ResultItem::Boolean(*b),
        Atomic::Integer(_, i) => {
            // Convert IBig to i64 if possible
            if let Ok(val) = i64::try_from(i.as_ref()) {
                ResultItem::Integer(val)
            } else {
                ResultItem::String(i.to_string())
            }
        }
        Atomic::Decimal(d) => ResultItem::Double(d.to_string().parse().unwrap_or(0.0)),
        Atomic::Float(f) => ResultItem::Double(f.into_inner() as f64),
        Atomic::Double(d) => ResultItem::Double(d.into_inner()),
        Atomic::Date(d) => ResultItem::Date(format!("{:?}", d)),
        Atomic::DateTime(dt) => ResultItem::DateTime(format!("{:?}", dt)),
        Atomic::Time(t) => ResultItem::DateTime(format!("{:?}", t)),
        Atomic::Duration(d) => ResultItem::Duration(format!("{:?}", d)),
        Atomic::YearMonthDuration(d) => ResultItem::Duration(format!("{:?}", d)),
        Atomic::DayTimeDuration(d) => ResultItem::Duration(format!("{:?}", d)),
        Atomic::QName(q) => ResultItem::QName(format!("{:?}", q)),
        _ => ResultItem::String(format!("{:?}", atomic)),
    }
}

impl XsltEngine for XeeEngine {
    fn transform(
        &mut self,
        doc: &Self::Document,
        stylesheet: &str,
    ) -> Result<Self::Document> {
        // Serialize the input document
        let xml_str = self
            .xot
            .to_string(doc.root)
            .map_err(|e| Error::EngineError(e.to_string()))?;

        // Use xee_xslt_compiler::evaluate
        let sequence = xee_xslt_compiler::evaluate(&mut self.xot, &xml_str, stylesheet)
            .map_err(|e| Error::XsltError(format!("{:?}", e)))?;

        // Get the first node from the result
        if let Some(item) = sequence.iter().next() {
            if let Ok(node) = item.to_node() {
                return Ok(XeeDocument { root: node });
            }
        }

        Err(Error::XsltError(
            "XSLT transformation did not produce a node".to_string(),
        ))
    }

    fn transform_to_string(
        &mut self,
        doc: &Self::Document,
        stylesheet: &str,
    ) -> Result<String> {
        let xml_str = self
            .xot
            .to_string(doc.root)
            .map_err(|e| Error::EngineError(e.to_string()))?;

        let sequence = xee_xslt_compiler::evaluate(&mut self.xot, &xml_str, stylesheet)
            .map_err(|e| Error::XsltError(format!("{:?}", e)))?;

        // Serialize all nodes in the result
        let mut result = String::new();
        for item in sequence.iter() {
            if let Ok(node) = item.to_node() {
                if let Ok(s) = self.xot.to_string(node) {
                    result.push_str(&s);
                }
            }
        }

        Ok(result)
    }

    fn xslt_version(&self) -> XsltVersion {
        XsltVersion::V3_0
    }
}

impl XQueryEngine for XeeEngine {
    type QueryResult = XeeQueryResult;

    fn execute_xquery(
        &mut self,
        _doc: &Self::Document,
        _xquery: &str,
    ) -> Result<Self::QueryResult> {
        Err(Error::Unsupported)
    }

    fn xquery_version(&self) -> XQueryVersion {
        XQueryVersion::V3_1 // Would be supported version if implemented
    }
}

impl XsdValidator for XeeEngine {
    fn load_schema(&mut self, _xsd: &str) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn load_schema_file(&mut self, _path: &Path) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn validate(&self, _doc: &Self::Document) -> Result<ValidationResult> {
        Err(Error::Unsupported)
    }

    fn xsd_version(&self) -> XsdVersion {
        XsdVersion::V1_1 // Would be supported version if implemented
    }
}

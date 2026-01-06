//! xrust engine wrapper
//!
//! Supports:
//! - XML parsing
//! - XPath ~1.0 evaluation
//! - XSLT ~1.0 transformation
//!
//! Does NOT support:
//! - XQuery
//! - XSD validation

use std::path::Path;

use xrust::item::{Item as XrustItem, Node, NodeType as XrustNodeType, SequenceTrait};
use xrust::parser::xml::parse as parse_xml;
use xrust::parser::xpath::parse as parse_xpath;
use xrust::transform::context::{ContextBuilder, StaticContextBuilder};
use xrust::trees::smite::RNode;
use xrust::xdmerror::{Error as XrustError, ErrorKind};
use xrust::xslt::from_document;

use crate::error::{Error, Result};
use crate::result::{NodeInfo, NodeType, ResultItem, ValidationResult};
use crate::traits::{
    QueryResult, XPathEngine, XPathVersion, XQueryEngine, XQueryVersion, XmlDocument, XmlParser,
    XsdValidator, XsdVersion, XsltEngine, XsltVersion,
};

/// xrust engine wrapper
pub struct XrustEngine;

impl Default for XrustEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl XrustEngine {
    pub fn new() -> Self {
        Self
    }
}

/// Document handle for xrust (wraps RNode)
pub struct XrustDocument {
    root: RNode,
}

impl XmlDocument for XrustDocument {
    fn to_string(&self) -> Result<String> {
        Ok(self.root.to_xml())
    }
}

/// Query result for xrust
pub struct XrustQueryResult {
    items: Vec<ResultItem>,
    string_repr: String,
}

impl QueryResult for XrustQueryResult {
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

impl XmlParser for XrustEngine {
    type Document = XrustDocument;

    fn parse(&mut self, xml: &str) -> Result<Self::Document> {
        let doc = RNode::new_document();
        parse_xml(doc.clone(), xml, None).map_err(|e| Error::ParseError(e.to_string()))?;
        Ok(XrustDocument { root: doc })
    }
}

impl XPathEngine for XrustEngine {
    type QueryResult = XrustQueryResult;

    fn evaluate_xpath(&mut self, doc: &Self::Document, xpath: &str) -> Result<Self::QueryResult> {
        // Parse the XPath expression
        let xpath_transform =
            parse_xpath::<RNode>(xpath, None).map_err(|e| Error::XPathError(e.to_string()))?;

        // Create context with the document as context item
        let context = ContextBuilder::new()
            .context(vec![XrustItem::Node(doc.root.clone())])
            .build();

        // Create static context with minimal implementations
        let mut static_context = StaticContextBuilder::new()
            .message(|_| Ok(()))
            .fetcher(|_| Err(XrustError::new(ErrorKind::NotImplemented, "not implemented")))
            .parser(|_| Err(XrustError::new(ErrorKind::NotImplemented, "not implemented")))
            .build();

        // Evaluate
        let sequence = context
            .dispatch(&mut static_context, &xpath_transform)
            .map_err(|e| Error::XPathError(e.to_string()))?;

        // Convert to our result types
        let mut items = Vec::new();
        let string_repr = sequence.to_string();

        for item in &sequence {
            match item {
                XrustItem::Node(n) => {
                    let node_type = match n.node_type() {
                        XrustNodeType::Document => NodeType::Document,
                        XrustNodeType::Element => NodeType::Element,
                        XrustNodeType::Text => NodeType::Text,
                        XrustNodeType::Attribute => NodeType::Attribute,
                        XrustNodeType::Comment => NodeType::Comment,
                        XrustNodeType::ProcessingInstruction => {
                            NodeType::ProcessingInstruction
                        }
                        XrustNodeType::Namespace => NodeType::Namespace,
                        _ => NodeType::Element, // Unknown/Reference
                    };
                    let name = {
                        let qn = n.name();
                        let local = qn.localname_to_string();
                        if local.is_empty() {
                            None
                        } else {
                            Some(local)
                        }
                    };
                    items.push(ResultItem::Node(NodeInfo {
                        node_type,
                        name,
                        value: Some(n.to_string()),
                    }));
                }
                XrustItem::Value(v) => {
                    use xrust::value::Value;
                    let result_item = match v.as_ref() {
                        Value::String(s) => ResultItem::String(s.clone()),
                        Value::Integer(i) => ResultItem::Integer(*i),
                        Value::Double(d) => ResultItem::Double(*d),
                        Value::Decimal(d) => ResultItem::Double(d.to_string().parse().unwrap_or(0.0)),
                        Value::Boolean(b) => ResultItem::Boolean(*b),
                        _ => ResultItem::String(format!("{:?}", v)),
                    };
                    items.push(result_item);
                }
                XrustItem::Function => {
                    items.push(ResultItem::String("<function>".to_string()));
                }
            }
        }

        Ok(XrustQueryResult { items, string_repr })
    }

    fn xpath_version(&self) -> XPathVersion {
        XPathVersion::V1_0
    }
}

impl XsltEngine for XrustEngine {
    fn transform(&mut self, doc: &Self::Document, stylesheet: &str) -> Result<Self::Document> {
        // Parse the stylesheet
        let style = RNode::new_document();
        parse_xml(style.clone(), stylesheet, None)
            .map_err(|e| Error::XsltError(format!("Failed to parse stylesheet: {}", e)))?;

        // Compile stylesheet
        let mut context = from_document(
            style,
            None,
            |s: &str| {
                let doc = RNode::new_document();
                parse_xml(doc.clone(), s, None)?;
                Ok(doc)
            },
            |_| Ok(String::new()),
        )
        .map_err(|e| Error::XsltError(e.to_string()))?;

        // Set source document as context
        context.context(vec![XrustItem::Node(doc.root.clone())], 0);

        // Create result document
        let result_doc = RNode::new_document();
        context.result_document(result_doc.clone());

        // Create static context and evaluate
        let mut static_context = StaticContextBuilder::new()
            .message(|_| Ok(()))
            .fetcher(|_| Err(XrustError::new(ErrorKind::NotImplemented, "not implemented")))
            .parser(|_| Err(XrustError::new(ErrorKind::NotImplemented, "not implemented")))
            .build();

        context
            .evaluate(&mut static_context)
            .map_err(|e| Error::XsltError(e.to_string()))?;

        Ok(XrustDocument { root: result_doc })
    }

    fn transform_to_string(&mut self, doc: &Self::Document, stylesheet: &str) -> Result<String> {
        let result = self.transform(doc, stylesheet)?;
        result.to_string()
    }

    fn xslt_version(&self) -> XsltVersion {
        XsltVersion::V1_0
    }
}

impl XQueryEngine for XrustEngine {
    type QueryResult = XrustQueryResult;

    fn execute_xquery(
        &mut self,
        _doc: &Self::Document,
        _xquery: &str,
    ) -> Result<Self::QueryResult> {
        Err(Error::Unsupported)
    }

    fn xquery_version(&self) -> XQueryVersion {
        XQueryVersion::V1_0
    }
}

impl XsdValidator for XrustEngine {
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
        XsdVersion::V1_0
    }
}

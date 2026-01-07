//! Unified engine wrapper with runtime backend selection
//!
//! Provides a single `XEngine` type that can use any backend (xee, xrust, xust)
//! with the same API, selectable at runtime.

use crate::engine_xee::{XeeDocument, XeeEngine, XeeQueryResult};
use crate::engine_xrust::{XrustDocument, XrustEngine, XrustQueryResult};
use crate::engine_xust::{XustDocument, XustEngine, XustQueryResult};
use crate::error::{Error, Result};
use crate::result::{ResultItem, ValidationResult};
use crate::traits::{
    QueryResult, XPathEngine, XPathVersion, XQueryEngine, XQueryVersion, XmlParser, XsdValidator,
    XsdVersion, XsltEngine, XsltVersion,
};
use std::path::Path;

/// Backend engine selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// xee - XPath 3.1, XSLT 3.0 (partial)
    Xee,
    /// xrust - XPath ~1.0, XSLT ~1.0
    Xrust,
    /// xust - XPath 3.1, XQuery 3.1, XSD 1.1
    Xust,
}

/// Unified XML engine with runtime backend selection
pub enum XEngine {
    Xee(XeeEngine),
    Xrust(XrustEngine),
    Xust(XustEngine),
}

/// Unified document handle
pub enum XDocument {
    Xee(XeeDocument),
    Xrust(XrustDocument),
    Xust(XustDocument),
}

/// Unified query result
pub enum XQueryResult {
    Xee(XeeQueryResult),
    Xrust(XrustQueryResult),
    Xust(XustQueryResult),
}

impl XEngine {
    /// Create a new engine with the xee backend (XPath 3.1, XSLT 3.0)
    pub fn xee() -> Self {
        Self::Xee(XeeEngine::new())
    }

    /// Create a new engine with the xrust backend (XPath ~1.0, XSLT ~1.0)
    pub fn xrust() -> Self {
        Self::Xrust(XrustEngine::new())
    }

    /// Create a new engine with the xust backend (XQuery 3.1, XSD 1.1)
    pub fn xust() -> Self {
        Self::Xust(XustEngine::new())
    }

    /// Create a new engine with the specified backend
    pub fn with_backend(backend: Backend) -> Self {
        match backend {
            Backend::Xee => Self::xee(),
            Backend::Xrust => Self::xrust(),
            Backend::Xust => Self::xust(),
        }
    }

    /// Get the current backend
    pub fn backend(&self) -> Backend {
        match self {
            Self::Xee(_) => Backend::Xee,
            Self::Xrust(_) => Backend::Xrust,
            Self::Xust(_) => Backend::Xust,
        }
    }

    // ==================== XML Parsing ====================

    /// Parse XML from a string
    pub fn parse(&mut self, xml: &str) -> Result<XDocument> {
        match self {
            Self::Xee(e) => e.parse(xml).map(XDocument::Xee),
            Self::Xrust(e) => e.parse(xml).map(XDocument::Xrust),
            Self::Xust(e) => e.parse(xml).map(XDocument::Xust),
        }
    }

    /// Parse XML from a file
    pub fn parse_file(&mut self, path: &Path) -> Result<XDocument> {
        match self {
            Self::Xee(e) => e.parse_file(path).map(XDocument::Xee),
            Self::Xrust(e) => e.parse_file(path).map(XDocument::Xrust),
            Self::Xust(e) => e.parse_file(path).map(XDocument::Xust),
        }
    }

    // ==================== XPath ====================

    /// Evaluate an XPath expression
    pub fn xpath(&mut self, doc: &XDocument, xpath: &str) -> Result<XQueryResult> {
        match (self, doc) {
            (Self::Xee(e), XDocument::Xee(d)) => e.evaluate_xpath(d, xpath).map(XQueryResult::Xee),
            (Self::Xrust(e), XDocument::Xrust(d)) => {
                e.evaluate_xpath(d, xpath).map(XQueryResult::Xrust)
            }
            (Self::Xust(e), XDocument::Xust(d)) => {
                e.evaluate_xpath(d, xpath).map(XQueryResult::Xust)
            }
            _ => Err(Error::EngineError(
                "Document was created with a different engine".to_string(),
            )),
        }
    }

    /// Get the XPath version supported by this engine
    pub fn xpath_version(&self) -> XPathVersion {
        match self {
            Self::Xee(e) => e.xpath_version(),
            Self::Xrust(e) => e.xpath_version(),
            Self::Xust(e) => e.xpath_version(),
        }
    }

    // ==================== XQuery ====================

    /// Execute an XQuery expression
    pub fn xquery(&mut self, doc: &XDocument, query: &str) -> Result<XQueryResult> {
        match (self, doc) {
            (Self::Xee(e), XDocument::Xee(d)) => e.execute_xquery(d, query).map(XQueryResult::Xee),
            (Self::Xrust(e), XDocument::Xrust(d)) => {
                e.execute_xquery(d, query).map(XQueryResult::Xrust)
            }
            (Self::Xust(e), XDocument::Xust(d)) => {
                e.execute_xquery(d, query).map(XQueryResult::Xust)
            }
            _ => Err(Error::EngineError(
                "Document was created with a different engine".to_string(),
            )),
        }
    }

    /// Get the XQuery version supported by this engine
    pub fn xquery_version(&self) -> XQueryVersion {
        match self {
            Self::Xee(e) => e.xquery_version(),
            Self::Xrust(e) => e.xquery_version(),
            Self::Xust(e) => e.xquery_version(),
        }
    }

    // ==================== XSLT ====================

    /// Transform a document using an XSLT stylesheet
    pub fn xslt(&mut self, doc: &XDocument, stylesheet: &str) -> Result<XDocument> {
        match (self, doc) {
            (Self::Xee(e), XDocument::Xee(d)) => e.transform(d, stylesheet).map(XDocument::Xee),
            (Self::Xrust(e), XDocument::Xrust(d)) => {
                e.transform(d, stylesheet).map(XDocument::Xrust)
            }
            (Self::Xust(e), XDocument::Xust(d)) => e.transform(d, stylesheet).map(XDocument::Xust),
            _ => Err(Error::EngineError(
                "Document was created with a different engine".to_string(),
            )),
        }
    }

    /// Transform a document to string using an XSLT stylesheet
    pub fn xslt_to_string(&mut self, doc: &XDocument, stylesheet: &str) -> Result<String> {
        match (self, doc) {
            (Self::Xee(e), XDocument::Xee(d)) => e.transform_to_string(d, stylesheet),
            (Self::Xrust(e), XDocument::Xrust(d)) => e.transform_to_string(d, stylesheet),
            (Self::Xust(e), XDocument::Xust(d)) => e.transform_to_string(d, stylesheet),
            _ => Err(Error::EngineError(
                "Document was created with a different engine".to_string(),
            )),
        }
    }

    /// Get the XSLT version supported by this engine
    pub fn xslt_version(&self) -> XsltVersion {
        match self {
            Self::Xee(e) => e.xslt_version(),
            Self::Xrust(e) => e.xslt_version(),
            Self::Xust(e) => e.xslt_version(),
        }
    }

    // ==================== XSD Validation ====================

    /// Load an XSD schema from a string
    pub fn load_schema(&mut self, xsd: &str) -> Result<()> {
        match self {
            Self::Xee(e) => e.load_schema(xsd),
            Self::Xrust(e) => e.load_schema(xsd),
            Self::Xust(e) => e.load_schema(xsd),
        }
    }

    /// Load an XSD schema from a file
    pub fn load_schema_file(&mut self, path: &Path) -> Result<()> {
        match self {
            Self::Xee(e) => e.load_schema_file(path),
            Self::Xrust(e) => e.load_schema_file(path),
            Self::Xust(e) => e.load_schema_file(path),
        }
    }

    /// Validate a document against the loaded schema
    pub fn validate(&self, doc: &XDocument) -> Result<ValidationResult> {
        match (self, doc) {
            (Self::Xee(e), XDocument::Xee(d)) => e.validate(d),
            (Self::Xrust(e), XDocument::Xrust(d)) => e.validate(d),
            (Self::Xust(e), XDocument::Xust(d)) => e.validate(d),
            _ => Err(Error::EngineError(
                "Document was created with a different engine".to_string(),
            )),
        }
    }

    /// Get the XSD version supported by this engine
    pub fn xsd_version(&self) -> XsdVersion {
        match self {
            Self::Xee(e) => e.xsd_version(),
            Self::Xrust(e) => e.xsd_version(),
            Self::Xust(e) => e.xsd_version(),
        }
    }

    // ==================== Convenience Methods ====================

    /// Transform a document using an XSLT stylesheet (alias for xslt_to_string)
    pub fn transform(&mut self, doc: &XDocument, stylesheet: &str) -> Result<String> {
        self.xslt_to_string(doc, stylesheet)
    }

    /// Validate a schema file (checks if the schema itself is valid)
    pub fn validate_schema(&mut self, schema_path: &Path) -> Result<bool> {
        match self.load_schema_file(schema_path) {
            Ok(()) => Ok(true),
            Err(Error::Unsupported) => Err(Error::Unsupported),
            Err(_) => Ok(false), // Schema is invalid
        }
    }

    /// Validate an instance document against a schema file
    pub fn validate_instance(&mut self, instance_path: &Path, schema_path: &Path) -> Result<bool> {
        // Load schema
        self.load_schema_file(schema_path)?;

        // Parse instance
        let instance_content = std::fs::read_to_string(instance_path)
            .map_err(|e| Error::EngineError(format!("Failed to read instance: {}", e)))?;
        let doc = self.parse(&instance_content)?;

        // Validate
        let result = self.validate(&doc)?;
        Ok(result.valid)
    }
}

impl XDocument {
    /// Serialize the document to a string
    pub fn to_string(&self) -> Result<String> {
        use crate::traits::XmlDocument;
        match self {
            Self::Xee(d) => d.to_string(),
            Self::Xrust(d) => d.to_string(),
            Self::Xust(d) => d.to_string(),
        }
    }
}

impl XQueryResult {
    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Xee(r) => r.is_empty(),
            Self::Xrust(r) => r.is_empty(),
            Self::Xust(r) => r.is_empty(),
        }
    }

    /// Get the number of items in the result
    pub fn count(&self) -> usize {
        match self {
            Self::Xee(r) => r.count(),
            Self::Xrust(r) => r.count(),
            Self::Xust(r) => r.count(),
        }
    }

    /// Convert the result to a string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::Xee(r) => r.to_string(),
            Self::Xrust(r) => r.to_string(),
            Self::Xust(r) => r.to_string(),
        }
    }

    /// Convert the result to XML (if applicable)
    pub fn to_xml(&self) -> Result<String> {
        match self {
            Self::Xee(r) => r.to_xml(),
            Self::Xrust(r) => r.to_xml(),
            Self::Xust(r) => r.to_xml(),
        }
    }

    /// Get all items in the result
    pub fn items(&self) -> Vec<ResultItem> {
        match self {
            Self::Xee(r) => r.items(),
            Self::Xrust(r) => r.items(),
            Self::Xust(r) => r.items(),
        }
    }
}

impl Default for XEngine {
    /// Default to xee backend
    fn default() -> Self {
        Self::xee()
    }
}

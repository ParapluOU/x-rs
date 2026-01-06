//! Core traits for XML engine abstraction

use std::path::Path;

use crate::error::Result;
use crate::result::{ResultItem, ValidationResult};

/// Version information for XPath
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XPathVersion {
    V1_0,
    V2_0,
    V3_0,
    V3_1,
}

/// Version information for XQuery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XQueryVersion {
    V1_0,
    V3_0,
    V3_1,
}

/// Version information for XSLT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XsltVersion {
    V1_0,
    V2_0,
    V3_0,
}

/// Version information for XSD
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XsdVersion {
    V1_0,
    V1_1,
}

/// XML document handle - opaque reference to parsed XML
pub trait XmlDocument {
    /// Serialize the document to a string
    fn to_string(&self) -> Result<String>;
}

/// XML parsing capability
pub trait XmlParser {
    /// The document type returned by this parser
    type Document: XmlDocument;

    /// Parse XML from a string
    fn parse(&mut self, xml: &str) -> Result<Self::Document>;

    /// Parse XML from a file
    fn parse_file(&mut self, path: &Path) -> Result<Self::Document> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content)
    }
}

/// Query/evaluation result
pub trait QueryResult {
    /// Check if the result is empty
    fn is_empty(&self) -> bool;

    /// Get the number of items in the result
    fn count(&self) -> usize;

    /// Convert the result to a string representation
    fn to_string(&self) -> String;

    /// Convert the result to XML (if applicable)
    fn to_xml(&self) -> Result<String>;

    /// Get all items in the result
    fn items(&self) -> Vec<ResultItem>;
}

/// XPath evaluation capability
pub trait XPathEngine: XmlParser {
    /// The result type returned by XPath evaluation
    type QueryResult: QueryResult;

    /// Evaluate an XPath expression against a document
    fn evaluate_xpath(
        &mut self,
        doc: &Self::Document,
        xpath: &str,
    ) -> Result<Self::QueryResult>;

    /// Get the XPath version supported by this engine
    fn xpath_version(&self) -> XPathVersion;
}

/// XQuery evaluation capability
pub trait XQueryEngine: XmlParser {
    /// The result type returned by XQuery evaluation
    type QueryResult: QueryResult;

    /// Execute an XQuery against a document
    fn execute_xquery(
        &mut self,
        doc: &Self::Document,
        xquery: &str,
    ) -> Result<Self::QueryResult>;

    /// Get the XQuery version supported by this engine
    fn xquery_version(&self) -> XQueryVersion;
}

/// XSLT transformation capability
pub trait XsltEngine: XmlParser {
    /// Transform a document using an XSLT stylesheet (as string)
    fn transform(
        &mut self,
        doc: &Self::Document,
        stylesheet: &str,
    ) -> Result<Self::Document>;

    /// Transform a document and return as string
    fn transform_to_string(
        &mut self,
        doc: &Self::Document,
        stylesheet: &str,
    ) -> Result<String> {
        let result = self.transform(doc, stylesheet)?;
        result.to_string()
    }

    /// Get the XSLT version supported by this engine
    fn xslt_version(&self) -> XsltVersion;
}

/// XSD validation capability
pub trait XsdValidator: XmlParser {
    /// Load a schema from a string
    fn load_schema(&mut self, xsd: &str) -> Result<()>;

    /// Load a schema from a file
    fn load_schema_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.load_schema(&content)
    }

    /// Validate a document against the loaded schema
    fn validate(&self, doc: &Self::Document) -> Result<ValidationResult>;

    /// Get the XSD version supported by this engine
    fn xsd_version(&self) -> XsdVersion;
}

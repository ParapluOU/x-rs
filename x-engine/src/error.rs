//! Error types for x-engine

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Operation not supported by this engine")]
    Unsupported,

    #[error("XML parsing error: {0}")]
    ParseError(String),

    #[error("XPath evaluation error: {0}")]
    XPathError(String),

    #[error("XQuery evaluation error: {0}")]
    XQueryError(String),

    #[error("XSLT transformation error: {0}")]
    XsltError(String),

    #[error("XSD validation error: {0}")]
    XsdError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Engine error: {0}")]
    EngineError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

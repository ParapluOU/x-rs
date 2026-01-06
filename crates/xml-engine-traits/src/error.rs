//! Error types for XML engine operations


/// Result type for XML engine operations
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for all XML engine operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// XML parsing failed
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// XPath compilation failed
    #[error("XPath compilation error: {0}")]
    XPathCompile(String),

    /// XPath evaluation failed
    #[error("XPath evaluation error: {0}")]
    XPathEval(String),

    /// XSLT compilation failed
    #[error("XSLT compilation error: {0}")]
    XsltCompile(String),

    /// XSLT transformation failed
    #[error("XSLT transformation error: {0}")]
    XsltTransform(String),

    /// XQuery compilation failed
    #[error("XQuery compilation error: {0}")]
    XQueryCompile(String),

    /// XQuery evaluation failed
    #[error("XQuery evaluation error: {0}")]
    XQueryEval(String),

    /// Requested engine is not supported
    #[error("Engine not supported: {0}")]
    EngineNotSupported(String),

    /// Requested feature is not supported by this engine
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    /// Test catalog parsing or access error
    #[error("Test catalog error: {0}")]
    TestCatalog(String),

    /// Type conversion error
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// Node access error
    #[error("Node access error: {0}")]
    NodeAccess(String),

    /// Context error
    #[error("Context error: {0}")]
    Context(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a new XPath compilation error
    pub fn xpath_compile<S: Into<String>>(msg: S) -> Self {
        Error::XPathCompile(msg.into())
    }

    /// Create a new XPath evaluation error
    pub fn xpath_eval<S: Into<String>>(msg: S) -> Self {
        Error::XPathEval(msg.into())
    }

    /// Create a new XSLT compilation error
    pub fn xslt_compile<S: Into<String>>(msg: S) -> Self {
        Error::XsltCompile(msg.into())
    }

    /// Create a new XSLT transformation error
    pub fn xslt_transform<S: Into<String>>(msg: S) -> Self {
        Error::XsltTransform(msg.into())
    }

    /// Create a new XQuery compilation error
    pub fn xquery_compile<S: Into<String>>(msg: S) -> Self {
        Error::XQueryCompile(msg.into())
    }

    /// Create a new XQuery evaluation error
    pub fn xquery_eval<S: Into<String>>(msg: S) -> Self {
        Error::XQueryEval(msg.into())
    }
}

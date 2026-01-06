//! XSLT engine abstraction trait

use crate::error::Result;
use crate::tree::XmlTree;
use std::collections::HashMap;

/// Trait for XSLT transformation engines.
///
/// This trait abstracts over different XSLT implementation strategies,
/// allowing different engines to be used interchangeably.
pub trait XsltEngine: Send + Sync {
    /// The XML tree implementation this engine works with
    type Tree: XmlTree;

    /// Type representing a compiled stylesheet
    type Stylesheet: Send + Sync;

    /// Type representing transformation context
    type Context: Clone + Send + Sync;

    /// Type representing transformation parameters
    type Parameters: Clone + Send + Sync + Default;

    /// Get access to the underlying tree
    fn tree(&mut self) -> &mut Self::Tree;

    /// Compile an XSLT stylesheet from a document
    fn compile_xslt(
        &mut self,
        xslt_doc: &<Self::Tree as XmlTree>::Document,
    ) -> Result<Self::Stylesheet>;

    /// Compile an XSLT stylesheet from a string
    fn compile_xslt_string(&mut self, xslt: &str) -> Result<Self::Stylesheet> {
        let doc = self.tree().parse_xml(xslt)?;
        self.compile_xslt(&doc)
    }

    /// Transform a source document using a compiled stylesheet
    fn transform(
        &mut self,
        stylesheet: &Self::Stylesheet,
        source: &<Self::Tree as XmlTree>::Document,
        params: &Self::Parameters,
    ) -> Result<<Self::Tree as XmlTree>::Document>;

    /// Create a new parameter set
    fn create_parameters(&self) -> Self::Parameters {
        Self::Parameters::default()
    }

    /// Add a parameter to the parameter set
    fn add_parameter(
        &mut self,
        params: &mut Self::Parameters,
        name: &str,
        value: &str,
    ) -> Result<()>;

    /// Get the XSLT version supported by this engine
    fn xslt_version(&self) -> &'static str;

    /// Get the list of feature strings supported by this engine
    fn supported_features(&self) -> Vec<String>;

    /// Check if a specific feature is supported
    fn supports_feature(&self, feature: &str) -> bool {
        self.supported_features()
            .iter()
            .any(|f| f.eq_ignore_ascii_case(feature))
    }
}

/// Extended XSLT engine capabilities
pub trait ExtendedXsltEngine: XsltEngine {
    /// Transform with a map of parameters
    fn transform_with_params(
        &mut self,
        stylesheet: &Self::Stylesheet,
        source: &<Self::Tree as XmlTree>::Document,
        params: HashMap<String, String>,
    ) -> Result<<Self::Tree as XmlTree>::Document> {
        let mut param_set = self.create_parameters();
        for (name, value) in params {
            self.add_parameter(&mut param_set, &name, &value)?;
        }
        self.transform(stylesheet, source, &param_set)
    }

    /// Quick transformation from strings
    fn transform_string(
        &mut self,
        xslt: &str,
        source_xml: &str,
    ) -> Result<String> {
        let stylesheet = self.compile_xslt_string(xslt)?;
        let source = self.tree().parse_xml(source_xml)?;
        let params = self.create_parameters();
        let result = self.transform(&stylesheet, &source, &params)?;
        self.tree().serialize_document(&result)
    }
}

// Blanket implementation for all XSLT engines
impl<T: XsltEngine> ExtendedXsltEngine for T {}

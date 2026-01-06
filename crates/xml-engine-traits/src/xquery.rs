//! XQuery engine abstraction trait

use crate::error::Result;
use crate::tree::XmlTree;

/// Trait for XQuery engines.
///
/// This trait abstracts over different XQuery implementation strategies,
/// allowing different engines to be used interchangeably.
pub trait XQueryEngine: Send + Sync {
    /// The XML tree implementation this engine works with
    type Tree: XmlTree;

    /// Type representing a compiled query
    type Query: Send + Sync;

    /// Type representing an execution context
    type Context: Clone + Send + Sync;

    /// Type representing a query result sequence
    type Sequence: Clone + Send + Sync;

    /// Get access to the underlying tree
    fn tree(&mut self) -> &mut Self::Tree;

    /// Compile an XQuery expression into a query
    fn compile_xquery(&self, xquery: &str) -> Result<Self::Query>;

    /// Evaluate a compiled query
    fn evaluate(
        &mut self,
        query: &Self::Query,
        context: &Self::Context,
    ) -> Result<Self::Sequence>;

    /// Create a new execution context
    fn create_context(&self) -> Self::Context;

    /// Add a document to the context
    fn add_document(
        &mut self,
        ctx: &mut Self::Context,
        uri: &str,
        doc: &<Self::Tree as XmlTree>::Document,
    ) -> Result<()>;

    /// Add a variable binding to the context
    fn add_variable(
        &mut self,
        ctx: &mut Self::Context,
        name: &str,
        value: &str,
    ) -> Result<()>;

    /// Add a namespace binding to the context
    fn add_namespace(
        &mut self,
        ctx: &mut Self::Context,
        prefix: &str,
        uri: &str,
    ) -> Result<()>;

    /// Convert a sequence to a string
    fn sequence_to_string(&self, seq: &Self::Sequence) -> Result<String>;

    /// Convert a sequence to a boolean
    fn sequence_to_boolean(&self, seq: &Self::Sequence) -> Result<bool>;

    /// Convert a sequence to a number
    fn sequence_to_number(&self, seq: &Self::Sequence) -> Result<f64>;

    /// Get the count of items in a sequence
    fn sequence_count(&self, seq: &Self::Sequence) -> usize;

    /// Check if a sequence is empty
    fn sequence_is_empty(&self, seq: &Self::Sequence) -> bool {
        self.sequence_count(seq) == 0
    }

    /// Get the XQuery version supported by this engine
    fn xquery_version(&self) -> &'static str;

    /// Get the list of feature strings supported by this engine
    fn supported_features(&self) -> Vec<String>;

    /// Check if a specific feature is supported
    fn supports_feature(&self, feature: &str) -> bool {
        self.supported_features()
            .iter()
            .any(|f| f.eq_ignore_ascii_case(feature))
    }
}

/// Extended XQuery engine capabilities
pub trait ExtendedXQueryEngine: XQueryEngine {
    /// Evaluate an XQuery expression directly (compile + evaluate in one call)
    fn evaluate_xquery(
        &mut self,
        xquery: &str,
        context: &Self::Context,
    ) -> Result<Self::Sequence> {
        let query = self.compile_xquery(xquery)?;
        self.evaluate(&query, context)
    }

    /// Quick evaluation returning a string
    fn eval_string(&mut self, xquery: &str) -> Result<String> {
        let context = self.create_context();
        let seq = self.evaluate_xquery(xquery, &context)?;
        self.sequence_to_string(&seq)
    }

    /// Quick evaluation returning a boolean
    fn eval_boolean(&mut self, xquery: &str) -> Result<bool> {
        let context = self.create_context();
        let seq = self.evaluate_xquery(xquery, &context)?;
        self.sequence_to_boolean(&seq)
    }

    /// Quick evaluation returning a number
    fn eval_number(&mut self, xquery: &str) -> Result<f64> {
        let context = self.create_context();
        let seq = self.evaluate_xquery(xquery, &context)?;
        self.sequence_to_number(&seq)
    }
}

// Blanket implementation for all XQuery engines
impl<T: XQueryEngine> ExtendedXQueryEngine for T {}

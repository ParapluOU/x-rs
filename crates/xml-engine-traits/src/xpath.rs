//! XPath engine abstraction trait

use crate::error::Result;
use crate::tree::XmlTree;

/// Trait for XPath query engines.
///
/// This trait abstracts over different XPath implementation strategies,
/// allowing different engines to be used interchangeably.
///
/// Note: This trait does not require Send + Sync as most XML
/// libraries use Rc<T> for internal references. Users needing
/// thread-safety should wrap the engine in Arc<Mutex<T>>.
pub trait XPathEngine {
    /// The XML tree implementation this engine works with
    type Tree: XmlTree;

    /// Type representing an execution context
    type Context: Clone;

    /// Type representing a compiled query
    type Query: Clone;

    /// Type representing a query result sequence
    type Sequence: Clone;

    /// Get access to the underlying tree
    fn tree(&mut self) -> &mut Self::Tree;

    /// Compile an XPath expression into a query
    fn compile_xpath(&self, xpath: &str) -> Result<Self::Query>;

    /// Evaluate a compiled query against a context node
    fn evaluate(
        &mut self,
        query: &Self::Query,
        context_node: &<Self::Tree as XmlTree>::Node,
        context: &Self::Context,
    ) -> Result<Self::Sequence>;

    /// Create a new execution context for this tree
    fn create_context(&self, tree: &Self::Tree) -> Self::Context;

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

    /// Get the XPath version supported by this engine
    fn xpath_version(&self) -> &'static str;

    /// Get the list of feature strings supported by this engine
    fn supported_features(&self) -> Vec<String>;

    /// Check if a specific feature is supported
    fn supports_feature(&self, feature: &str) -> bool {
        self.supported_features()
            .iter()
            .any(|f| f.eq_ignore_ascii_case(feature))
    }
}

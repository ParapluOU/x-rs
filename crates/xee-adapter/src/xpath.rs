//! XPathEngine implementation for xee

use std::rc::Rc;
use xml_engine_traits::{
    error::{Error, Result},
    tree::XmlTree,
    xpath::XPathEngine,
};
use xee_interpreter::{
    atomic::Atomic,
    context::{DynamicContext, StaticContextBuilder},
    interpreter::Program,
    sequence::{Item, Sequence},
    xml::DocumentHandle,
};
use xee_xpath::Documents;
use xee_xpath_compiler::parse;
use xot::Node;

use crate::tree::XotTreeWrapper;

/// xee XPath engine adapter
pub struct XeeEngine {
    documents: Documents,
    queries: Queries<'static>,
}

impl XeeEngine {
    /// Create a new xee engine
    pub fn new() -> Self {
        Self {
            documents: Documents::new(),
            queries: Queries::default(),
        }
    }

    /// Get a reference to the documents
    pub fn documents(&self) -> &Documents {
        &self.documents
    }

    /// Get a mutable reference to the documents
    pub fn documents_mut(&mut self) -> &mut Documents {
        &self.documents
    }
}

impl Default for XeeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiled XPath query for xee
#[derive(Clone)]
pub struct XeeQuery {
    program: Rc<Program>,
}

/// Execution context for xee
#[derive(Clone)]
pub struct XeeContext {
    static_context_builder: StaticContextBuilder<'static>,
}

impl Default for XeeContext {
    fn default() -> Self {
        Self {
            static_context_builder: StaticContextBuilder::default(),
        }
    }
}

impl XPathEngine for XeeEngine {
    type Tree = XotTreeWrapper;
    type Context = XeeContext;
    type Query = XeeQuery;
    type Sequence = Sequence;

    fn tree(&mut self) -> &mut Self::Tree {
        // This is a bit of a hack - we need to provide a wrapper around the xot
        // instance in documents. For now, we'll create a temporary wrapper.
        // In practice, users should access documents directly for tree operations.
        panic!("Direct tree access not supported for xee adapter. Use documents() instead.");
    }

    fn compile_xpath(&self, xpath: &str) -> Result<Self::Query> {
        let static_context = StaticContextBuilder::default().build();
        let program = parse(static_context, xpath)
            .map_err(|e| Error::XPathCompile(format!("{:?}", e)))?;

        Ok(XeeQuery {
            program: Rc::new(program),
        })
    }

    fn evaluate(
        &mut self,
        query: &Self::Query,
        context_node: &Node,
        _context: &Self::Context,
    ) -> Result<Self::Sequence> {
        // Create a dynamic context for evaluation
        let mut dynamic_context = DynamicContext::new(
            self.documents.documents(),
            &mut self.documents.xot,
        );

        // Set the context item to the provided node
        let context_item = Item::Node(*context_node);
        dynamic_context.set_context_item(Some(context_item));

        // Execute the program
        query
            .program
            .execute(&mut dynamic_context)
            .map_err(|e| Error::XPathEval(format!("{:?}", e)))
    }

    fn create_context(&self, _tree: &Self::Tree) -> Self::Context {
        XeeContext::default()
    }

    fn add_variable(
        &mut self,
        ctx: &mut Self::Context,
        name: &str,
        value: &str,
    ) -> Result<()> {
        // Add variable to static context builder
        // This is simplified - in reality we'd need to properly convert the value
        ctx.static_context_builder
            .add_variable(name.to_string(), Atomic::String(value.to_string()));
        Ok(())
    }

    fn add_namespace(
        &mut self,
        ctx: &mut Self::Context,
        prefix: &str,
        uri: &str,
    ) -> Result<()> {
        // Add namespace to static context builder
        ctx.static_context_builder
            .add_namespace(prefix.to_string(), uri.to_string());
        Ok(())
    }

    fn sequence_to_string(&self, seq: &Self::Sequence) -> Result<String> {
        if seq.is_empty() {
            return Ok(String::new());
        }

        // Get the first item and convert to string
        let item = &seq[0];
        match item {
            Item::Atomic(atomic) => Ok(atomic.to_string()),
            Item::Node(node) => {
                // Get text content of node
                Ok(self.documents.xot().text_content(*node))
            }
            Item::Function(_) => Err(Error::TypeConversion(
                "Cannot convert function to string".to_string(),
            )),
            Item::Map(_) => Err(Error::TypeConversion(
                "Cannot convert map to string".to_string(),
            )),
            Item::Array(_) => Err(Error::TypeConversion(
                "Cannot convert array to string".to_string(),
            )),
        }
    }

    fn sequence_to_boolean(&self, seq: &Self::Sequence) -> Result<bool> {
        // XPath effective boolean value rules
        if seq.is_empty() {
            return Ok(false);
        }

        let item = &seq[0];
        match item {
            Item::Atomic(Atomic::Boolean(b)) => Ok(*b),
            Item::Atomic(Atomic::String(_, s)) => Ok(!s.is_empty()),
            Item::Atomic(Atomic::Integer(_, i)) => Ok(**i != 0.into()),
            Item::Atomic(Atomic::Decimal(d)) => Ok(!d.is_zero()),
            Item::Atomic(Atomic::Double(d)) => Ok(d.0 != 0.0 && !d.is_nan()),
            Item::Node(_) => Ok(true),
            _ => Ok(true),
        }
    }

    fn sequence_to_number(&self, seq: &Self::Sequence) -> Result<f64> {
        if seq.is_empty() {
            return Ok(f64::NAN);
        }

        let item = &seq[0];
        match item {
            Item::Atomic(Atomic::Integer(_, i)) => {
                Ok(i.to_string().parse().unwrap_or(f64::NAN))
            }
            Item::Atomic(Atomic::Decimal(d)) => {
                Ok(d.to_string().parse().unwrap_or(f64::NAN))
            }
            Item::Atomic(Atomic::Double(d)) => Ok(d.0),
            Item::Atomic(Atomic::String(_, s)) => {
                Ok(s.trim().parse().unwrap_or(f64::NAN))
            }
            _ => Ok(f64::NAN),
        }
    }

    fn sequence_count(&self, seq: &Self::Sequence) -> usize {
        seq.len()
    }

    fn xpath_version(&self) -> &'static str {
        "3.1"
    }

    fn supported_features(&self) -> Vec<String> {
        vec![
            "xpath-3.1".to_string(),
            "higher-order-functions".to_string(),
            "maps-and-arrays".to_string(),
        ]
    }
}

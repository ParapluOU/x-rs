# XML Engine Abstraction Layer - Architecture Plan

## Executive Summary

This document outlines the architecture for a unified abstraction layer that enables swappable XML processing engines (xee, xrust, xust) for running W3C conformance tests and generating a compliance matrix across all Rust XML crates.

## Goals

1. **Engine Abstraction**: Create a trait-based abstraction layer that allows any XML engine to be plugged in
2. **Test Harness**: Build a unified test harness that can run W3C test suites (QT3, XSLT, XSD) against any engine
3. **Compliance Matrix**: Generate comprehensive compliance reports comparing all engines across all test suites
4. **Minimal Engine Modification**: Design the abstraction to require minimal changes to existing engines

## Current State Analysis

### Library Comparison

| Feature | xee | xrust | xust | xot |
|---------|-----|-------|------|-----|
| **Primary Use** | XPath 3.1, XSLT 3.0 | XPath 1.0, XSLT 1.0 | XQuery 3.1, XSD 1.1 | XML tree manipulation |
| **Tree Model** | Uses xot | smite (custom) | Custom tree | Native tree impl |
| **Test Infrastructure** | xee-testrunner | None visible | xust_tests | N/A |
| **XPath Support** | 3.1 (91% QT3) | 1.0 equivalent | 3.1 | None |
| **XSLT Support** | 3.0 partial | 1.0 equivalent | None | None |
| **XQuery Support** | None | None | 3.1 | None |
| **XSD Support** | Basic types | Basic types | 1.1 full | None |

### Key Architectural Insights

#### xee Architecture
- Uses xot for XML tree representation
- Bytecode interpreter for XPath/XSLT execution
- Well-developed test runner (`xee-testrunner`) for QT3 and XSLT tests
- API: `Documents` and `Queries` for query execution
- Compilation: XPath → AST → IR → Bytecode → Interpreter

#### xrust Architecture
- Uses smite tree (mutable, fully navigable)
- Separation of parsing and evaluation
- Transform-based execution model
- API: `Context` and `Transform` for execution
- Compilation: XPath/XSLT → Transform → Evaluation

#### xust Architecture
- Custom tree implementation with XSD type system integration
- Grammar-based parsing (xust_grammar)
- Evaluation engine (xust_eval)
- Test infrastructure: qt3tests, xml_test_suite, xsd_test_suite
- API: `parse()` → `eval_xquery()` with Context

#### xot Architecture
- Pure XML tree library (no XPath/XSLT)
- Node-based API with Xot struct
- Used by xee as underlying tree implementation
- Full tree manipulation capabilities

## Proposed Architecture

### Layer 1: Core Abstractions

#### 1.1 XML Tree Abstraction

```rust
// Core trait for XML tree representation
pub trait XmlTree: Send + Sync {
    type Node: Clone + Send + Sync;
    type Document: Clone + Send + Sync;

    // Tree construction
    fn parse_xml(&mut self, xml: &str) -> Result<Self::Document, Error>;
    fn parse_xml_with_uri(&mut self, uri: &str, xml: &str) -> Result<Self::Document, Error>;

    // Tree navigation
    fn document_element(&self, doc: &Self::Document) -> Result<Self::Node, Error>;
    fn parent(&self, node: &Self::Node) -> Option<Self::Node>;
    fn children(&self, node: &Self::Node) -> Vec<Self::Node>;
    fn attributes(&self, node: &Self::Node) -> Vec<(String, String)>;

    // Node properties
    fn node_name(&self, node: &Self::Node) -> Option<String>;
    fn node_value(&self, node: &Self::Node) -> Option<String>;
    fn node_type(&self, node: &Self::Node) -> NodeType;

    // Serialization
    fn serialize(&self, node: &Self::Node) -> Result<String, Error>;
}

pub enum NodeType {
    Document,
    Element,
    Attribute,
    Text,
    Comment,
    ProcessingInstruction,
    Namespace,
}
```

#### 1.2 XPath Engine Abstraction

```rust
// Trait for XPath query execution
pub trait XPathEngine: Send + Sync {
    type Tree: XmlTree;
    type Context;
    type Query;
    type Sequence;

    // Query compilation
    fn compile_xpath(&self, xpath: &str) -> Result<Self::Query, Error>;

    // Query execution
    fn evaluate(
        &mut self,
        query: &Self::Query,
        context_node: &<Self::Tree as XmlTree>::Node,
        context: &Self::Context,
    ) -> Result<Self::Sequence, Error>;

    // Context management
    fn create_context(&self, tree: &Self::Tree) -> Self::Context;
    fn add_variable(&mut self, ctx: &mut Self::Context, name: &str, value: &str) -> Result<(), Error>;
    fn add_namespace(&mut self, ctx: &mut Self::Context, prefix: &str, uri: &str) -> Result<(), Error>;

    // Result extraction
    fn sequence_to_string(&self, seq: &Self::Sequence) -> Result<String, Error>;
    fn sequence_to_boolean(&self, seq: &Self::Sequence) -> Result<bool, Error>;
    fn sequence_to_number(&self, seq: &Self::Sequence) -> Result<f64, Error>;
    fn sequence_count(&self, seq: &Self::Sequence) -> usize;

    // Version info
    fn xpath_version(&self) -> &'static str;
    fn supported_features(&self) -> Vec<String>;
}
```

#### 1.3 XSLT Engine Abstraction

```rust
// Trait for XSLT transformation execution
pub trait XsltEngine: Send + Sync {
    type Tree: XmlTree;
    type Stylesheet;
    type Context;
    type Parameters;

    // Stylesheet compilation
    fn compile_xslt(&mut self, xslt_doc: &<Self::Tree as XmlTree>::Document)
        -> Result<Self::Stylesheet, Error>;

    // Transformation execution
    fn transform(
        &mut self,
        stylesheet: &Self::Stylesheet,
        source: &<Self::Tree as XmlTree>::Document,
        params: &Self::Parameters,
    ) -> Result<<Self::Tree as XmlTree>::Document, Error>;

    // Parameter management
    fn create_parameters(&self) -> Self::Parameters;
    fn add_parameter(&mut self, params: &mut Self::Parameters, name: &str, value: &str)
        -> Result<(), Error>;

    // Version info
    fn xslt_version(&self) -> &'static str;
    fn supported_features(&self) -> Vec<String>;
}
```

#### 1.4 XQuery Engine Abstraction

```rust
// Trait for XQuery execution
pub trait XQueryEngine: Send + Sync {
    type Tree: XmlTree;
    type Query;
    type Context;
    type Sequence;

    // Query compilation
    fn compile_xquery(&self, xquery: &str) -> Result<Self::Query, Error>;

    // Query execution
    fn evaluate(
        &mut self,
        query: &Self::Query,
        context: &Self::Context,
    ) -> Result<Self::Sequence, Error>;

    // Context management
    fn create_context(&self) -> Self::Context;
    fn add_document(&mut self, ctx: &mut Self::Context, uri: &str, doc: &<Self::Tree as XmlTree>::Document)
        -> Result<(), Error>;
    fn add_variable(&mut self, ctx: &mut Self::Context, name: &str, value: &str)
        -> Result<(), Error>;

    // Result extraction (same as XPath)
    fn sequence_to_string(&self, seq: &Self::Sequence) -> Result<String, Error>;
    fn sequence_to_boolean(&self, seq: &Self::Sequence) -> Result<bool, Error>;
    fn sequence_to_number(&self, seq: &Self::Sequence) -> Result<f64, Error>;

    // Version info
    fn xquery_version(&self) -> &'static str;
    fn supported_features(&self) -> Vec<String>;
}
```

### Layer 2: Engine Implementations

#### 2.1 xee Engine Adapter

```rust
pub struct XeeEngine {
    documents: Documents,
    queries: Queries,
}

impl XPathEngine for XeeEngine {
    type Tree = XotTreeWrapper;
    type Context = XeeContext;
    type Query = Query</* appropriate type */>;
    type Sequence = Sequence;

    // Implementation maps to xee-xpath API
}

impl XsltEngine for XeeEngine {
    // Uses xee-xslt-compiler
}
```

#### 2.2 xrust Engine Adapter

```rust
pub struct XrustEngine {
    // Internal state
}

impl XPathEngine for XrustEngine {
    type Tree = SmiteTreeWrapper;
    type Context = Context;
    type Query = Transform;
    type Sequence = Sequence;

    // Implementation maps to xrust API
}

impl XsltEngine for XrustEngine {
    // Uses xrust::xslt
}
```

#### 2.3 xust Engine Adapter

```rust
pub struct XustEngine {
    context_init: ContextInit,
}

impl XPathEngine for XustEngine {
    // XPath support via XQuery
}

impl XQueryEngine for XustEngine {
    type Tree = XustTreeWrapper;
    type Query = ParsedQuery;
    type Context = GlobalContext;
    type Sequence = Vec<Item>;

    // Implementation maps to xust_eval API
}
```

### Layer 3: Test Infrastructure

#### 3.1 Test Catalog Parser

```rust
pub struct TestCatalog {
    pub test_sets: Vec<TestSet>,
}

pub struct TestSet {
    pub name: String,
    pub description: String,
    pub test_cases: Vec<TestCase>,
    pub dependencies: Vec<Dependency>,
}

pub struct TestCase {
    pub name: String,
    pub description: String,
    pub test: TestDefinition,
    pub assertions: Vec<Assertion>,
    pub dependencies: Vec<Dependency>,
}

pub enum TestDefinition {
    XPath { query: String, context: Option<String> },
    XQuery { query: String, modules: Vec<String> },
    Xslt { stylesheet: String, source: String },
}

pub enum Assertion {
    Equal(String),
    Error(String),
    True,
    False,
    Count(usize),
    Type(String),
    Serialization(String),
}

// Unified catalog parser supporting QT3, XSLT, XSD formats
pub fn parse_qt3_catalog(path: &Path) -> Result<TestCatalog, Error>;
pub fn parse_xslt_catalog(path: &Path) -> Result<TestCatalog, Error>;
pub fn parse_xsd_catalog(path: &Path) -> Result<TestCatalog, Error>;
```

#### 3.2 Test Runner

```rust
pub struct TestRunner<E>
where
    E: XPathEngine + XsltEngine + XQueryEngine,
{
    engine: E,
    catalog: TestCatalog,
    results: Vec<TestResult>,
}

pub struct TestResult {
    pub test_name: String,
    pub test_set: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub error_message: Option<String>,
}

pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

impl<E> TestRunner<E>
where
    E: XPathEngine + XsltEngine + XQueryEngine,
{
    pub fn new(engine: E, catalog: TestCatalog) -> Self;
    pub fn run_all(&mut self) -> Vec<TestResult>;
    pub fn run_test_set(&mut self, name: &str) -> Vec<TestResult>;
    pub fn run_test(&mut self, name: &str) -> TestResult;
}
```

#### 3.3 Compliance Matrix Generator

```rust
pub struct ComplianceMatrix {
    pub engines: Vec<EngineResults>,
    pub test_suites: Vec<String>,
}

pub struct EngineResults {
    pub engine_name: String,
    pub version: String,
    pub suite_results: HashMap<String, SuiteResults>,
}

pub struct SuiteResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub pass_rate: f64,
}

impl ComplianceMatrix {
    pub fn new() -> Self;
    pub fn add_engine_results(&mut self, engine: &str, results: Vec<TestResult>);
    pub fn generate_markdown(&self) -> String;
    pub fn generate_html(&self) -> String;
    pub fn generate_json(&self) -> String;
    pub fn generate_comparison_table(&self) -> String;
}
```

### Layer 4: CLI Application

```rust
// Main CLI structure
pub struct XmlTestCli {
    // Configuration
}

pub enum Engine {
    Xee,
    Xrust,
    Xust,
    All,
}

pub enum TestSuite {
    Qt3,
    Xslt,
    Xsd,
    All,
}

pub struct CliArgs {
    pub engine: Engine,
    pub test_suite: TestSuite,
    pub test_filter: Option<String>,
    pub output_format: OutputFormat,
    pub output_path: Option<PathBuf>,
    pub verbose: bool,
}

pub enum OutputFormat {
    Markdown,
    Html,
    Json,
    Junit,
}

// CLI commands
pub fn run_tests(args: &CliArgs) -> Result<(), Error>;
pub fn generate_matrix(args: &CliArgs) -> Result<(), Error>;
pub fn compare_engines(args: &CliArgs) -> Result<(), Error>;
```

## Implementation Plan

### Phase 1: Foundation (Weeks 1-2)
1. Create workspace structure for new crates
2. Define core traits in `xml-engine-traits` crate
3. Implement error types and common utilities
4. Set up testing infrastructure

### Phase 2: Engine Adapters (Weeks 3-5)
1. Implement xee adapter
   - XPath engine trait
   - XSLT engine trait (basic)
   - Tree wrapper for xot
2. Implement xrust adapter
   - XPath engine trait
   - XSLT engine trait
   - Tree wrapper for smite
3. Implement xust adapter
   - XQuery engine trait
   - XPath via XQuery
   - Tree wrapper

### Phase 3: Test Infrastructure (Weeks 6-8)
1. Implement QT3 catalog parser
   - Reuse/adapt xee-testrunner catalog code
   - Reuse/adapt xust catalog parsing
2. Implement XSLT catalog parser
3. Implement XSD catalog parser
4. Build unified test runner
5. Implement result collection

### Phase 4: Compliance Matrix (Weeks 9-10)
1. Implement matrix generator
2. Add output formatters (Markdown, HTML, JSON)
3. Create comparison views
4. Generate visualizations

### Phase 5: CLI & Documentation (Weeks 11-12)
1. Build CLI application
2. Write comprehensive documentation
3. Add examples
4. Create tutorial/guide

## Workspace Structure

```
x-rs/
├── Cargo.toml                    # Workspace root
├── README.md                     # Main README with compliance matrix
├── ARCHITECTURE_PLAN.md          # This document
│
├── crates/
│   ├── xml-engine-traits/        # Core trait definitions
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tree.rs
│   │   │   ├── xpath.rs
│   │   │   ├── xslt.rs
│   │   │   ├── xquery.rs
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── xee-adapter/              # xee engine adapter
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── xpath.rs
│   │   │   ├── xslt.rs
│   │   │   └── tree.rs
│   │   └── Cargo.toml
│   │
│   ├── xrust-adapter/            # xrust engine adapter
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── xpath.rs
│   │   │   ├── xslt.rs
│   │   │   └── tree.rs
│   │   └── Cargo.toml
│   │
│   ├── xust-adapter/             # xust engine adapter
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── xquery.rs
│   │   │   ├── xpath.rs
│   │   │   └── tree.rs
│   │   └── Cargo.toml
│   │
│   ├── xml-test-harness/         # Test infrastructure
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── catalog/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── qt3.rs
│   │   │   │   ├── xslt.rs
│   │   │   │   └── xsd.rs
│   │   │   ├── runner.rs
│   │   │   ├── assertion.rs
│   │   │   └── result.rs
│   │   └── Cargo.toml
│   │
│   ├── xml-compliance-matrix/    # Compliance matrix generator
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── matrix.rs
│   │   │   ├── output/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── markdown.rs
│   │   │   │   ├── html.rs
│   │   │   │   └── json.rs
│   │   │   └── compare.rs
│   │   └── Cargo.toml
│   │
│   └── xml-test-cli/             # CLI application
│       ├── src/
│       │   ├── main.rs
│       │   ├── cli.rs
│       │   └── commands/
│       │       ├── mod.rs
│       │       ├── run.rs
│       │       ├── matrix.rs
│       │       └── compare.rs
│       └── Cargo.toml
│
├── xee/                          # xee submodule (unchanged)
├── xrust/                        # xrust submodule (unchanged)
├── xust/                         # xust submodule (unchanged)
├── xot/                          # xot submodule (unchanged)
│
└── tests/                        # W3C test suites (unchanged)
    ├── qt3tests/
    ├── xslt30-test/
    └── xsdtests/
```

## Usage Examples

### Running Tests for Single Engine

```bash
# Run all XPath tests on xee
xml-test-cli run --engine xee --suite qt3 --output matrix.md

# Run specific test set on xrust
xml-test-cli run --engine xrust --suite qt3 --filter "fn-string" --verbose

# Run XSLT tests on all engines
xml-test-cli run --engine all --suite xslt --output compliance.json
```

### Generating Compliance Matrix

```bash
# Generate full compliance matrix
xml-test-cli matrix --output README.md --format markdown

# Generate HTML report
xml-test-cli matrix --output report.html --format html

# Compare two engines
xml-test-cli compare --engines xee,xrust --suite qt3
```

### Programmatic Usage

```rust
use xml_engine_traits::{XPathEngine, XmlTree};
use xee_adapter::XeeEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine
    let mut engine = XeeEngine::new();

    // Parse XML
    let mut tree = engine.tree();
    let doc = tree.parse_xml("<root><item>foo</item></root>")?;

    // Compile XPath
    let query = engine.compile_xpath("//item/text()")?;

    // Execute query
    let context = engine.create_context(&tree);
    let root = tree.document_element(&doc)?;
    let result = engine.evaluate(&query, &root, &context)?;

    // Get result
    let text = engine.sequence_to_string(&result)?;
    println!("Result: {}", text);

    Ok(())
}
```

## Testing Strategy

### Unit Tests
- Test each trait implementation independently
- Mock engines for test harness testing
- Validate catalog parsing with known test cases

### Integration Tests
- Run subset of W3C tests through each engine
- Verify adapter correctness
- Test error handling

### Compliance Tests
- Full W3C test suite runs
- Performance benchmarks
- Memory usage profiling

## Performance Considerations

1. **Lazy Evaluation**: Parse test catalogs lazily to reduce startup time
2. **Parallel Execution**: Run independent tests in parallel
3. **Caching**: Cache parsed stylesheets and queries
4. **Streaming Results**: Generate reports incrementally for large test suites

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("XPath compilation error: {0}")]
    XPathCompile(String),

    #[error("XPath evaluation error: {0}")]
    XPathEval(String),

    #[error("XSLT compilation error: {0}")]
    XsltCompile(String),

    #[error("XSLT transformation error: {0}")]
    XsltTransform(String),

    #[error("XQuery compilation error: {0}")]
    XQueryCompile(String),

    #[error("XQuery evaluation error: {0}")]
    XQueryEval(String),

    #[error("Engine not supported: {0}")]
    EngineNotSupported(String),

    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    #[error("Test catalog error: {0}")]
    TestCatalog(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Future Enhancements

1. **Web Interface**: Build web UI for browsing compliance matrix
2. **Continuous Testing**: CI/CD integration for automatic compliance tracking
3. **More Engines**: Add adapters for other Rust XML libraries (roxmltree, quick-xml, etc.)
4. **Performance Comparison**: Add timing metrics to compliance matrix
5. **Test Case Debugger**: Interactive tool to debug failing tests
6. **Differential Testing**: Automatically identify where engines diverge

## Success Metrics

1. **Coverage**: All three engines (xee, xrust, xust) successfully adapted
2. **Tests**: All W3C test suites runnable through unified harness
3. **Matrix**: Automated generation of comprehensive compliance matrix
4. **Documentation**: Complete API documentation and usage guide
5. **Performance**: Test suite runs complete in reasonable time (<1 hour for full suite)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Engine APIs too different | High | Keep trait abstractions flexible, allow engine-specific extensions |
| Test catalog complexity | Medium | Reuse existing parsers from xee/xust where possible |
| Performance overhead | Medium | Profile and optimize hot paths, consider caching strategies |
| Maintenance burden | Medium | Clear separation of concerns, comprehensive tests |
| Breaking changes in engines | High | Pin to specific versions, document required engine versions |

## Conclusion

This architecture provides a clean, extensible foundation for comparing XML processing engines in Rust. By defining clear trait boundaries and building reusable infrastructure, we enable:

1. Fair comparison across different implementation approaches
2. Easy addition of new engines in the future
3. Comprehensive compliance tracking
4. Valuable insights for the Rust XML ecosystem

The implementation is ambitious but achievable, with clear milestones and deliverables at each phase.

# Implementation Plan - XML Engine Abstraction

## Overview

This document provides a step-by-step implementation plan for building the XML engine abstraction layer that enables swappable engines (xee, xrust, xust) for W3C conformance testing.

## Status: Foundation Complete ✓

The core trait abstractions have been implemented in `crates/xml-engine-traits/`.

## Next Steps

### Phase 1: Complete Stub Implementations (Immediate)

#### 1.1 Create xee-adapter stub
```bash
mkdir -p crates/xee-adapter/src
```

Create:
- `Cargo.toml` with dependencies on `xml-engine-traits`, `xee-xpath`, `xot`
- `src/lib.rs` with XeeEngine struct
- `src/tree.rs` with XotTreeWrapper implementing XmlTree
- `src/xpath.rs` with XPathEngine implementation
- `src/xslt.rs` with XsltEngine stub (returns FeatureNotSupported)

#### 1.2 Create xrust-adapter stub
```bash
mkdir -p crates/xrust-adapter/src
```

Create:
- `Cargo.toml` with dependencies on `xml-engine-traits`, `xrust`
- `src/lib.rs` with XrustEngine struct
- `src/tree.rs` with SmiteTreeWrapper implementing XmlTree
- `src/xpath.rs` with XPathEngine implementation
- `src/xslt.rs` with XsltEngine implementation

#### 1.3 Create xust-adapter stub
```bash
mkdir -p crates/xust-adapter/src
```

Create:
- `Cargo.toml` with dependencies on `xml-engine-traits`, `xust_*` crates
- `src/lib.rs` with XustEngine struct
- `src/tree.rs` with XustTreeWrapper implementing XmlTree
- `src/xquery.rs` with XQueryEngine implementation
- `src/xpath.rs` with XPath via XQuery wrapper

### Phase 2: Test Infrastructure (Week 1-2)

#### 2.1 Create xml-test-harness
```bash
mkdir -p crates/xml-test-harness/src/{catalog,runner}
```

Files to create:
- `Cargo.toml` - dependencies on `xml-engine-traits`, XML parsing
- `src/lib.rs` - public API exports
- `src/catalog/mod.rs` - catalog abstraction
- `src/catalog/qt3.rs` - QT3 catalog parser (adapt from xee-testrunner)
- `src/catalog/xslt.rs` - XSLT catalog parser
- `src/catalog/xsd.rs` - XSD catalog parser (adapt from xust)
- `src/runner.rs` - test execution engine
- `src/assertion.rs` - assertion checking logic
- `src/result.rs` - test result types
- `src/environment.rs` - test environment setup

Key types:
```rust
pub struct TestCatalog { /* ... */ }
pub struct TestRunner<E: XPathEngine> { /* ... */ }
pub enum TestResult { Passed, Failed, Skipped, Error }
```

#### 2.2 Basic Test Execution
Implement:
- Catalog file parsing (XML parsing)
- Test case extraction
- Simple assertion checking (string equality, error matching)
- Result collection

### Phase 3: Engine Adapter Implementation (Week 3-4)

#### 3.1 xee-adapter Complete Implementation

Priority order:
1. XmlTree trait for xot wrapper (should be straightforward)
2. XPathEngine trait - map to `xee-xpath` API:
   - `Documents::new()` → tree
   - `Queries::default()` → query compilation
   - `Query::execute()` → evaluation
3. Test with simple XPath queries
4. Handle error mapping from xee errors to trait errors

#### 3.2 xrust-adapter Complete Implementation

Priority order:
1. XmlTree trait for smite wrapper
2. XPathEngine trait - map to xrust API:
   - `parser::xml()` → parsing
   - `xpath::parse()` → query compilation
   - `Context::new()` → context creation
   - `transform::Transform::apply()` → evaluation
3. XsltEngine trait - map to xrust XSLT:
   - `xslt::from_document()` → stylesheet compilation
   - Apply transformation
4. Test with XPath 1.0 queries and basic XSLT

#### 3.3 xust-adapter Complete Implementation

Priority order:
1. XmlTree trait for xust tree wrapper
2. XQueryEngine trait - map to xust API:
   - `xust_xml::parse()` → parsing
   - `xust_grammar::parse()` → query compilation
   - `xust_eval::eval_xquery()` → evaluation
3. XPathEngine via XQuery (XPath is subset of XQuery)
4. Test with XPath 3.1 and XQuery 3.1

### Phase 4: Compliance Matrix Generator (Week 5)

#### 4.1 Create xml-compliance-matrix
```bash
mkdir -p crates/xml-compliance-matrix/src/output
```

Files to create:
- `Cargo.toml` - dependencies for serialization (serde, etc.)
- `src/lib.rs` - public API
- `src/matrix.rs` - ComplianceMatrix type and aggregation logic
- `src/output/mod.rs` - output format traits
- `src/output/markdown.rs` - Markdown table generator
- `src/output/html.rs` - HTML report generator
- `src/output/json.rs` - JSON export
- `src/compare.rs` - engine comparison utilities

Key functionality:
```rust
pub struct ComplianceMatrix { /* ... */ }

impl ComplianceMatrix {
    pub fn add_results(&mut self, engine: &str, results: Vec<TestResult>);
    pub fn generate_markdown(&self) -> String;
    pub fn generate_html(&self) -> String;
    pub fn generate_json(&self) -> String;
}
```

#### 4.2 Report Generation
- Aggregate test results by engine and test suite
- Calculate pass rates
- Generate comparison tables
- Create feature support matrix

### Phase 5: CLI Application (Week 6)

#### 5.1 Create xml-test-cli
```bash
mkdir -p crates/xml-test-cli/src/commands
```

Files to create:
- `Cargo.toml` - dependencies on all other crates, clap
- `src/main.rs` - entry point
- `src/cli.rs` - argument parsing with clap
- `src/commands/mod.rs` - command dispatch
- `src/commands/run.rs` - run tests command
- `src/commands/matrix.rs` - generate matrix command
- `src/commands/compare.rs` - compare engines command

CLI structure:
```bash
xml-test-cli run --engine <xee|xrust|xust|all> --suite <qt3|xslt|xsd|all>
xml-test-cli matrix --output <file> --format <md|html|json>
xml-test-cli compare --engines <engine1,engine2> --suite <suite>
```

### Phase 6: Documentation and Examples (Week 7)

#### 6.1 Documentation
- API documentation for all public types
- Architecture overview
- Implementation guide for new engines
- User guide for CLI

#### 6.2 Examples
Create examples/:
- `simple_xpath.rs` - Basic XPath evaluation
- `simple_xslt.rs` - Basic XSLT transformation
- `custom_engine.rs` - Implementing custom engine adapter
- `batch_testing.rs` - Running test suites programmatically

## Testing Strategy

### Unit Tests
- Each trait implementation has unit tests
- Mock engines for test harness testing
- Catalog parser tests with sample data

### Integration Tests
Create `tests/` directory with:
- `integration_xee.rs` - Test xee adapter with real queries
- `integration_xrust.rs` - Test xrust adapter
- `integration_xust.rs` - Test xust adapter
- `test_harness.rs` - Test running W3C tests

### End-to-End Tests
- Run actual W3C test subsets through each engine
- Verify compliance matrix generation
- Test CLI commands

## Development Workflow

### Iteration Cycle
1. Implement feature in trait definitions (if needed)
2. Implement in one adapter (xee recommended as most complete)
3. Write tests for that adapter
4. Implement in other adapters
5. Update test harness if needed
6. Document

### Testing Each Phase
```bash
# Check compilation
cargo check --all

# Run tests
cargo test --all

# Try with specific engine
cargo run --bin xml-test-cli -- run --engine xee --suite qt3 --filter "fn-string"

# Generate matrix
cargo run --bin xml-test-cli -- matrix --output compliance.md
```

## Success Criteria

### Phase 1-2 Complete When:
- [ ] All three adapter crates compile
- [ ] Basic test harness can parse QT3 catalog
- [ ] Can run at least one test through each engine

### Phase 3 Complete When:
- [ ] xee adapter passes 100+ QT3 tests
- [ ] xrust adapter passes 50+ QT3 tests
- [ ] xust adapter passes 100+ QT3 tests

### Phase 4 Complete When:
- [ ] Compliance matrix generates for all engines
- [ ] Markdown output matches expected format
- [ ] HTML output is readable and styled

### Phase 5 Complete When:
- [ ] CLI can run all W3C test suites
- [ ] CLI can generate matrices in all formats
- [ ] CLI help text is comprehensive

### Phase 6 Complete When:
- [ ] All public APIs documented
- [ ] README has usage examples
- [ ] At least 3 working examples provided

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Engine APIs too different | Keep trait abstractions flexible, use adapter pattern |
| xrust lacks active maintenance | Document thoroughly, consider forking if needed |
| Test catalog parsing complexity | Reuse existing parsers from xee/xust |
| Performance with large test suites | Add parallelization, caching, filtering |

## Timeline

Assuming ~20 hours/week of development:

- **Week 1**: Phase 1-2 (Foundation + Test Infrastructure)
- **Week 2-3**: Phase 3.1 (xee adapter complete)
- **Week 3-4**: Phase 3.2-3.3 (xrust and xust adapters)
- **Week 5**: Phase 4 (Compliance matrix generator)
- **Week 6**: Phase 5 (CLI application)
- **Week 7**: Phase 6 (Documentation and polish)

Total: ~7 weeks to production-ready system

## Quick Start for Contributors

```bash
# Clone repo
git clone <repo-url>
cd x-rs

# Initialize submodules
git submodule update --init --recursive

# Build everything
cargo build --all

# Run tests
cargo test --all

# Try running a test
cargo run --bin xml-test-cli -- run --engine xee --suite qt3 --filter "fn-string-1"

# Generate compliance matrix
cargo run --bin xml-test-cli -- matrix --output README.md
```

## Current Status Summary

✅ **Completed:**
- Architecture design
- Core trait definitions (xml-engine-traits)
- Error types
- Workspace structure

🚧 **In Progress:**
- Adapter implementations (stubs created)

📋 **Planned:**
- Test harness
- Compliance matrix generator
- CLI application
- Documentation

## Next Immediate Actions

1. Create stub implementations for all three adapters
2. Verify they compile
3. Implement xee-adapter XmlTree trait
4. Write first integration test
5. Iterate on remaining features

## Questions to Resolve

1. **Performance**: Should we optimize for speed or completeness first?
   - **Decision**: Completeness first, optimize later

2. **Error handling**: How detailed should error messages be?
   - **Decision**: Preserve as much detail as possible from underlying engines

3. **Feature detection**: How to handle engine-specific features?
   - **Decision**: Use feature strings, engines return supported features list

4. **Test filtering**: What filtering capabilities needed?
   - **Decision**: By name, by feature, by test set, by pass/fail status

5. **Parallel execution**: Run tests in parallel?
   - **Decision**: Yes, with configurable parallelism level (future enhancement)

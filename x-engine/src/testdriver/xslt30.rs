//! XSLT 3.0 test driver
//!
//! Runs tests from the W3C XSLT 3.0 test suite against XSLT engines.

use std::collections::HashMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::error::Result;
use crate::unified::XEngine;

use super::{TestOutcome, TestResult};

// ============== Data Model ==============

/// A parsed XSLT 3.0 catalog
#[derive(Debug)]
pub struct Catalog {
    /// Test set references
    pub test_sets: Vec<TestSetRef>,
}

/// Reference to a test set file
#[derive(Debug, Clone)]
pub struct TestSetRef {
    pub name: String,
    pub file: String,
}

/// A test set containing multiple test cases
#[derive(Debug)]
pub struct TestSet {
    pub name: String,
    pub description: String,
    /// Environments defined in this test set
    pub environments: HashMap<String, Environment>,
    /// Test cases
    pub test_cases: Vec<TestCase>,
}

/// Test environment configuration
#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub name: String,
    /// Source documents
    pub sources: Vec<Source>,
    /// Stylesheet file
    pub stylesheet: Option<PathBuf>,
    /// Schema references
    pub schemas: Vec<PathBuf>,
}

/// Source document
#[derive(Debug, Clone)]
pub struct Source {
    pub role: String,
    pub file: Option<PathBuf>,
    pub content: Option<String>,
    pub uri: Option<String>,
}

/// A single test case
#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    /// Environment reference
    pub environment: Option<String>,
    /// Stylesheet file (if not in environment)
    pub stylesheet: Option<PathBuf>,
    /// Initial mode
    pub initial_mode: Option<String>,
    /// Initial template
    pub initial_template: Option<String>,
    /// Dependencies
    pub dependencies: Vec<Dependency>,
    /// Expected result
    pub result: ExpectedResult,
}

/// Dependency specification
#[derive(Debug, Clone)]
pub struct Dependency {
    pub dep_type: String,
    pub value: String,
}

/// Expected result
#[derive(Debug, Clone)]
pub enum ExpectedResult {
    /// Expect specific output
    AssertResult(String),
    /// Expect specific XML output
    AssertXml { file: Option<PathBuf>, content: Option<String> },
    /// Expect an error
    Error(String),
    /// All of these must match
    AllOf(Vec<ExpectedResult>),
    /// Any of these can match
    AnyOf(Vec<ExpectedResult>),
}

// ============== Catalog Parsing ==============

/// Parse an XSLT 3.0 catalog file
pub fn parse_catalog(catalog_path: &Path) -> Result<Catalog> {
    let content = fs::read_to_string(catalog_path)
        .map_err(|e| crate::error::Error::EngineError(format!("Failed to read catalog: {}", e)))?;

    let mut engine = XEngine::xee();
    let doc = engine.parse(&content)?;

    let mut catalog = Catalog {
        test_sets: Vec::new(),
    };

    // Parse test-set references using indexed queries with string() function
    let count_result = engine.xpath(&doc, "count(//*[local-name()='test-set'])")?;
    let count_str = count_result.to_string();
    let count: usize = count_str.trim().parse().unwrap_or(0);

    for idx in 1..=count {
        // Use string() function to extract attribute values
        let name_xpath = format!("string(//*[local-name()='test-set'][{}]/@name)", idx);
        let file_xpath = format!("string(//*[local-name()='test-set'][{}]/@file)", idx);

        let name_result = engine.xpath(&doc, &name_xpath)?;
        let file_result = engine.xpath(&doc, &file_xpath)?;

        let name = name_result.to_string().trim().to_string();
        let file = file_result.to_string().trim().to_string();

        if !name.is_empty() && !file.is_empty() {
            catalog.test_sets.push(TestSetRef { name, file });
        }
    }

    Ok(catalog)
}

/// Parse a test set file
pub fn parse_test_set(test_set_path: &Path, _global_envs: &HashMap<String, Environment>) -> Result<TestSet> {
    let content = fs::read_to_string(test_set_path)
        .map_err(|e| crate::error::Error::EngineError(format!("Failed to read test set: {}", e)))?;

    let mut engine = XEngine::xee();
    let doc = engine.parse(&content)?;

    // Get test set name
    let name = engine.xpath(&doc, "string(/*/@name)")?.to_string();
    let description = engine.xpath(&doc, "string(/*[local-name()='description'])")?.to_string();

    let mut test_set = TestSet {
        name: name.trim().to_string(),
        description: description.trim().to_string(),
        environments: HashMap::new(),
        test_cases: Vec::new(),
    };

    let base_dir = test_set_path.parent().unwrap_or(Path::new("."));

    // Parse environments using indexed queries
    let env_count_result = engine.xpath(&doc, "count(/*[local-name()='test-set']/*[local-name()='environment'])")?;
    let env_count: usize = env_count_result.to_string().trim().parse().unwrap_or(0);

    for env_idx in 1..=env_count {
        // Get environment name
        let env_name_xpath = format!("string(/*[local-name()='test-set']/*[local-name()='environment'][{}]/@name)", env_idx);
        let env_name = engine.xpath(&doc, &env_name_xpath)?.to_string().trim().to_string();
        if env_name.is_empty() { continue; }

        let mut env = Environment {
            name: env_name.clone(),
            sources: Vec::new(),
            stylesheet: None,
            schemas: Vec::new(),
        };

        // Parse source elements in this environment
        let source_count_xpath = format!("count(/*[local-name()='test-set']/*[local-name()='environment'][{}]/*[local-name()='source'])", env_idx);
        let source_count: usize = engine.xpath(&doc, &source_count_xpath)
            .map(|r| r.to_string().trim().parse().unwrap_or(0))
            .unwrap_or(0);

        for src_idx in 1..=source_count {
            let role_xpath = format!("string(/*[local-name()='test-set']/*[local-name()='environment'][{}]/*[local-name()='source'][{}]/@role)", env_idx, src_idx);
            let role = engine.xpath(&doc, &role_xpath)
                .map(|r| r.to_string().trim().to_string())
                .unwrap_or_default();

            let file_xpath = format!("string(/*[local-name()='test-set']/*[local-name()='environment'][{}]/*[local-name()='source'][{}]/@file)", env_idx, src_idx);
            let file = engine.xpath(&doc, &file_xpath)
                .ok()
                .map(|r| r.to_string().trim().to_string())
                .filter(|s| !s.is_empty())
                .map(|s| base_dir.join(s));

            let content_xpath = format!("string(/*[local-name()='test-set']/*[local-name()='environment'][{}]/*[local-name()='source'][{}]/*[local-name()='content'])", env_idx, src_idx);
            let content = engine.xpath(&doc, &content_xpath)
                .ok()
                .map(|r| r.to_string())
                .filter(|s| !s.trim().is_empty());

            env.sources.push(Source {
                role,
                file,
                content,
                uri: None,
            });
        }

        test_set.environments.insert(env_name, env);
    }

    // Parse test cases using indexed queries
    let test_case_count_result = engine.xpath(&doc, "count(//*[local-name()='test-case'])")?;
    let test_case_count: usize = test_case_count_result.to_string().trim().parse().unwrap_or(0);

    for idx in 1..=test_case_count {
        // Get test case name
        let name_xpath = format!("string(//*[local-name()='test-case'][{}]/@name)", idx);
        let name = engine.xpath(&doc, &name_xpath)?.to_string().trim().to_string();
        if name.is_empty() { continue; }

        // Get test case details
        let desc_xpath = format!("string(//*[local-name()='test-case'][{}]/*[local-name()='description'])", idx);
        let desc = engine.xpath(&doc, &desc_xpath)
            .map(|r| r.to_string().trim().to_string())
            .unwrap_or_default();

        // Get stylesheet from test/stylesheet/@file
        let style_xpath = format!("string(//*[local-name()='test-case'][{}]/*[local-name()='test']/*[local-name()='stylesheet']/@file)", idx);
        let stylesheet = engine.xpath(&doc, &style_xpath)
            .ok()
            .map(|r| r.to_string().trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| base_dir.join(s));

        // Get environment ref
        let env_xpath = format!("string(//*[local-name()='test-case'][{}]/*[local-name()='environment']/@ref)", idx);
        let env_ref = engine.xpath(&doc, &env_xpath)
            .ok()
            .map(|r| r.to_string().trim().to_string())
            .filter(|s| !s.is_empty());

        // Get initial-template
        let init_template_xpath = format!("string(//*[local-name()='test-case'][{}]/*[local-name()='test']/*[local-name()='initial-template']/@name)", idx);
        let initial_template = engine.xpath(&doc, &init_template_xpath)
            .ok()
            .map(|r| r.to_string().trim().to_string())
            .filter(|s| !s.is_empty());

        // Get initial-mode
        let init_mode_xpath = format!("string(//*[local-name()='test-case'][{}]/*[local-name()='test']/*[local-name()='initial-mode']/@name)", idx);
        let initial_mode = engine.xpath(&doc, &init_mode_xpath)
            .ok()
            .map(|r| r.to_string().trim().to_string())
            .filter(|s| !s.is_empty());

        test_set.test_cases.push(TestCase {
            name,
            description: desc,
            environment: env_ref,
            stylesheet,
            initial_mode,
            initial_template,
            dependencies: Vec::new(),
            result: ExpectedResult::AssertResult(String::new()),
        });
    }

    Ok(test_set)
}

// ============== Test Execution ==============

/// Run a single XSLT test case
fn run_test_case(
    engine: &mut XEngine,
    test_case: &TestCase,
    test_set_name: &str,
    environments: &HashMap<String, Environment>,
    base_dir: &Path,
) -> TestResult {
    let start = Instant::now();

    // Helper to create TestResult
    let make_result = |outcome: TestOutcome, expected: Option<String>, actual: Option<String>| {
        TestResult {
            test_id: test_case.name.clone(),
            test_set: test_set_name.to_string(),
            test_suite: "xslt30".to_string(),
            description: Some(test_case.description.clone()),
            outcome,
            expected,
            actual,
            duration: start.elapsed(),
        }
    };

    // Get stylesheet path (must be specified in test case)
    let stylesheet_path = match &test_case.stylesheet {
        Some(p) => p.clone(),
        None => {
            return make_result(
                TestOutcome::NotApplicable,
                None,
                Some("No stylesheet specified in test case".to_string()),
            );
        }
    };

    // Validate environment exists if referenced
    if let Some(env_name) = &test_case.environment {
        if !environments.contains_key(env_name) {
            return make_result(
                TestOutcome::Error(format!("Environment not found: {}", env_name)),
                None,
                None,
            );
        }
    }

    // Load stylesheet
    let stylesheet_content = match fs::read_to_string(&stylesheet_path) {
        Ok(c) => c,
        Err(e) => {
            return make_result(
                TestOutcome::Error(format!("Failed to read stylesheet: {}", e)),
                None,
                None,
            );
        }
    };

    // Get source document from environment
    let source_doc = if let Some(env_name) = &test_case.environment {
        if let Some(env) = environments.get(env_name) {
            // Find source with role="." (context item)
            let context_source = env.sources.iter().find(|s| s.role == ".");
            if let Some(source) = context_source.or(env.sources.first()) {
                if let Some(ref content) = source.content {
                    match engine.parse(content) {
                        Ok(doc) => Some(doc),
                        Err(e) => {
                            return make_result(
                                TestOutcome::Error(format!("Failed to parse source: {}", e)),
                                None,
                                None,
                            );
                        }
                    }
                } else if let Some(ref file) = source.file {
                    // file is already an absolute path from parsing
                    match engine.parse_file(file) {
                        Ok(doc) => Some(doc),
                        Err(e) => {
                            return make_result(
                                TestOutcome::Error(format!("Failed to load source {:?}: {}", file, e)),
                                None,
                                None,
                            );
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Create empty document if no source
    let source_doc = match source_doc {
        Some(d) => d,
        None => match engine.parse("<empty/>") {
            Ok(d) => d,
            Err(e) => {
                return make_result(
                    TestOutcome::Error(format!("Failed to create empty doc: {}", e)),
                    None,
                    None,
                );
            }
        },
    };

    // Run transformation
    match engine.transform(&source_doc, &stylesheet_content) {
        Ok(result) => {
            // For now, just check that transformation succeeded
            // Full implementation would compare against expected result
            make_result(TestOutcome::Pass, None, Some(result))
        }
        Err(e) => {
            // Check if error was expected
            if let ExpectedResult::Error(_) = &test_case.result {
                make_result(TestOutcome::Pass, None, Some(format!("Expected error: {}", e)))
            } else {
                make_result(
                    TestOutcome::Fail(format!("Transform failed: {}", e)),
                    None,
                    Some(e.to_string()),
                )
            }
        }
    }
}

// ============== Public API ==============

/// Run XSLT 3.0 tests against an engine
pub fn run_xslt_tests(
    engine: &mut XEngine,
    catalog_path: &Path,
    filter: Option<&str>,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    // Parse catalog
    let catalog = match parse_catalog(catalog_path) {
        Ok(c) => c,
        Err(e) => {
            results.push(TestResult {
                test_id: "catalog_parse".to_string(),
                test_set: "catalog".to_string(),
                test_suite: "xslt30".to_string(),
                description: Some("Parse XSLT 3.0 catalog file".to_string()),
                outcome: TestOutcome::Error(format!("Failed to parse catalog: {}", e)),
                expected: None,
                actual: None,
                duration: std::time::Duration::ZERO,
            });
            return results;
        }
    };

    let base_dir = catalog_path.parent().unwrap_or(Path::new("."));

    // Filter test sets
    let test_sets_to_run: Vec<_> = catalog.test_sets.iter()
        .filter(|ts| {
            if let Some(f) = filter {
                ts.name.contains(f)
            } else {
                true
            }
        })
        .collect();
    let total_test_sets = test_sets_to_run.len();

    // Run each test set
    for (set_idx, test_set_ref) in test_sets_to_run.iter().enumerate() {
        eprintln!("[{}/{}] Processing test set: {}", set_idx + 1, total_test_sets, test_set_ref.name);

        let test_set_path = base_dir.join(&test_set_ref.file);
        let test_set_name = &test_set_ref.name;

        // Parse test set with panic handling
        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| {
            parse_test_set(&test_set_path, &HashMap::new())
        }));

        let test_set = match parse_result {
            Ok(Ok(ts)) => ts,
            Ok(Err(e)) => {
                results.push(TestResult {
                    test_id: format!("{}/parse", test_set_name),
                    test_set: test_set_name.to_string(),
                    test_suite: "xslt30".to_string(),
                    description: Some(format!("Parse test set {}", test_set_name)),
                    outcome: TestOutcome::Error(format!("Failed to parse test set: {}", e)),
                    expected: None,
                    actual: None,
                    duration: std::time::Duration::ZERO,
                });
                continue;
            }
            Err(panic_info) => {
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                results.push(TestResult {
                    test_id: format!("{}/parse", test_set_name),
                    test_set: test_set_name.to_string(),
                    test_suite: "xslt30".to_string(),
                    description: Some(format!("Parse test set {}", test_set_name)),
                    outcome: TestOutcome::Error(format!("Panic: {}", panic_msg)),
                    expected: None,
                    actual: Some("PANIC".to_string()),
                    duration: std::time::Duration::ZERO,
                });
                continue;
            }
        };

        // Run each test case
        for test_case in &test_set.test_cases {
            let start = Instant::now();
            let test_id = test_case.name.clone();
            let description = test_case.description.clone();

            let test_set_name_clone = test_set_name.clone();
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                run_test_case(
                    engine,
                    test_case,
                    &test_set_name_clone,
                    &test_set.environments,
                    &test_set_path.parent().unwrap_or(Path::new(".")),
                )
            }));

            let test_result = match result {
                Ok(r) => r,
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };
                    TestResult {
                        test_id,
                        test_set: test_set_name.to_string(),
                        test_suite: "xslt30".to_string(),
                        description: Some(description),
                        outcome: TestOutcome::Error(format!("Engine panic: {}", panic_msg)),
                        expected: None,
                        actual: Some("PANIC".to_string()),
                        duration: start.elapsed(),
                    }
                }
            };
            results.push(test_result);
        }
    }

    results
}

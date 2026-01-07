//! XSD (XML Schema) test driver
//!
//! Runs tests from the W3C XSD test suite against schema validators.

use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::error::Result;
use crate::unified::XEngine;

use super::{TestOutcome, TestResult};

// ============== Data Model ==============

/// A parsed XSD test suite
#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    /// Test set references
    pub test_set_refs: Vec<TestSetRef>,
}

/// Reference to a test set file
#[derive(Debug, Clone)]
pub struct TestSetRef {
    pub href: String,
}

/// A test set containing test groups
#[derive(Debug)]
pub struct TestSet {
    pub name: String,
    pub contributor: String,
    /// Test groups
    pub test_groups: Vec<TestGroup>,
}

/// A group of related tests
#[derive(Debug)]
pub struct TestGroup {
    pub name: String,
    pub title: String,
    pub description: String,
    /// Schema test (validates the schema itself)
    pub schema_test: Option<SchemaTest>,
    /// Instance tests (validates XML against schema)
    pub instance_tests: Vec<InstanceTest>,
}

/// Test that validates a schema document
#[derive(Debug)]
pub struct SchemaTest {
    pub name: String,
    pub schema_document: PathBuf,
    pub expected_validity: Validity,
}

/// Test that validates an instance document against a schema
#[derive(Debug)]
pub struct InstanceTest {
    pub name: String,
    pub instance_document: PathBuf,
    pub expected_validity: Validity,
}

/// Expected validity
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Validity {
    Valid,
    Invalid,
    Indeterminate,
}

impl Validity {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "valid" => Validity::Valid,
            "invalid" => Validity::Invalid,
            _ => Validity::Indeterminate,
        }
    }
}

// ============== Parsing ==============

/// Parse the XSD test suite catalog
pub fn parse_suite(suite_path: &Path) -> Result<TestSuite> {
    let content = fs::read_to_string(suite_path)
        .map_err(|e| crate::error::Error::EngineError(format!("Failed to read suite: {}", e)))?;

    let mut engine = XEngine::xee();
    let doc = engine.parse(&content)?;

    // Get suite name
    let name = engine.xpath(&doc, "string(/*/@name)")?.to_string();

    let mut suite = TestSuite {
        name: name.trim().to_string(),
        test_set_refs: Vec::new(),
    };

    // Parse testSetRef elements using indexed queries
    let count_result = engine.xpath(&doc, "count(//*[local-name()='testSetRef'])")?;
    let count: usize = count_result.to_string().trim().parse().unwrap_or(0);

    for idx in 1..=count {
        let href_xpath = format!("string(//*[local-name()='testSetRef'][{}]/@*[local-name()='href'])", idx);
        let href = engine.xpath(&doc, &href_xpath)?.to_string().trim().to_string();
        if !href.is_empty() {
            suite.test_set_refs.push(TestSetRef { href });
        }
    }

    Ok(suite)
}

/// Parse a test set file
pub fn parse_test_set(test_set_path: &Path) -> Result<TestSet> {
    let content = fs::read_to_string(test_set_path)
        .map_err(|e| crate::error::Error::EngineError(format!("Failed to read test set: {}", e)))?;

    let mut engine = XEngine::xee();
    let doc = engine.parse(&content)?;

    let base_dir = test_set_path.parent().unwrap_or(Path::new("."));

    // Get test set attributes
    let name = engine.xpath(&doc, "string(/*/@name)")?.to_string();
    let contributor = engine.xpath(&doc, "string(/*/@contributor)")?.to_string();

    let mut test_set = TestSet {
        name: name.trim().to_string(),
        contributor: contributor.trim().to_string(),
        test_groups: Vec::new(),
    };

    // Parse test groups using indexed queries
    let group_count_result = engine.xpath(&doc, "count(//*[local-name()='testGroup'])")?;
    let group_count: usize = group_count_result.to_string().trim().parse().unwrap_or(0);

    for group_idx in 1..=group_count {
        // Get group name
        let name_xpath = format!("string(//*[local-name()='testGroup'][{}]/@name)", group_idx);
        let group_name = engine.xpath(&doc, &name_xpath)?.to_string().trim().to_string();
        if group_name.is_empty() { continue; }

        // Get group details
        let title_xpath = format!(
            "string(//*[local-name()='testGroup'][{}]//*[local-name()='Title'])",
            group_idx
        );
        let title = engine.xpath(&doc, &title_xpath)
            .map(|r| r.to_string().trim().to_string())
            .unwrap_or_default();

        let desc_xpath = format!(
            "string(//*[local-name()='testGroup'][{}]//*[local-name()='Description'])",
            group_idx
        );
        let description = engine.xpath(&doc, &desc_xpath)
            .map(|r| r.to_string().trim().to_string())
            .unwrap_or_default();

        let mut test_group = TestGroup {
            name: group_name.clone(),
            title: title.trim().to_string(),
            description: description.trim().to_string(),
            schema_test: None,
            instance_tests: Vec::new(),
        };

        // Parse schema test using indexed query
        let schema_name_xpath = format!(
            "string(//*[local-name()='testGroup'][{}]//*[local-name()='schemaTest']/@name)",
            group_idx
        );
        let schema_name = engine.xpath(&doc, &schema_name_xpath)
            .map(|r| r.to_string().trim().to_string())
            .unwrap_or_default();

        if !schema_name.is_empty() {
            let schema_doc_xpath = format!(
                "string(//*[local-name()='testGroup'][{}]//*[local-name()='schemaTest']//*[local-name()='schemaDocument']/@*[local-name()='href'])",
                group_idx
            );
            let schema_doc = engine.xpath(&doc, &schema_doc_xpath)
                .map(|r| r.to_string().trim().to_string())
                .unwrap_or_default();

            let validity_xpath = format!(
                "string(//*[local-name()='testGroup'][{}]//*[local-name()='schemaTest']//*[local-name()='expected']/@validity)",
                group_idx
            );
            let validity_str = engine.xpath(&doc, &validity_xpath)
                .map(|r| r.to_string().trim().to_string())
                .unwrap_or_default();

            if !schema_doc.is_empty() {
                test_group.schema_test = Some(SchemaTest {
                    name: schema_name,
                    schema_document: base_dir.join(&schema_doc),
                    expected_validity: Validity::from_str(&validity_str),
                });
            }
        }

        // Parse instance tests using indexed queries
        let instance_count_xpath = format!(
            "count(//*[local-name()='testGroup'][{}]//*[local-name()='instanceTest'])",
            group_idx
        );
        let instance_count: usize = engine.xpath(&doc, &instance_count_xpath)
            .map(|r| r.to_string().trim().parse().unwrap_or(0))
            .unwrap_or(0);

        for instance_idx in 1..=instance_count {
            let instance_name_xpath = format!(
                "string(//*[local-name()='testGroup'][{}]//*[local-name()='instanceTest'][{}]/@name)",
                group_idx, instance_idx
            );
            let instance_name = engine.xpath(&doc, &instance_name_xpath)
                .map(|r| r.to_string().trim().to_string())
                .unwrap_or_default();
            if instance_name.is_empty() { continue; }

            let instance_doc_xpath = format!(
                "string(//*[local-name()='testGroup'][{}]//*[local-name()='instanceTest'][{}]//*[local-name()='instanceDocument']/@*[local-name()='href'])",
                group_idx, instance_idx
            );
            let instance_doc = engine.xpath(&doc, &instance_doc_xpath)
                .map(|r| r.to_string().trim().to_string())
                .unwrap_or_default();

            let validity_xpath = format!(
                "string(//*[local-name()='testGroup'][{}]//*[local-name()='instanceTest'][{}]//*[local-name()='expected']/@validity)",
                group_idx, instance_idx
            );
            let validity_str = engine.xpath(&doc, &validity_xpath)
                .map(|r| r.to_string().trim().to_string())
                .unwrap_or_default();

            if !instance_doc.is_empty() {
                test_group.instance_tests.push(InstanceTest {
                    name: instance_name,
                    instance_document: base_dir.join(&instance_doc),
                    expected_validity: Validity::from_str(&validity_str),
                });
            }
        }

        test_set.test_groups.push(test_group);
    }

    Ok(test_set)
}

// ============== Test Execution ==============

/// Run a schema validation test
fn run_schema_test(
    engine: &mut XEngine,
    test: &SchemaTest,
    test_set_name: &str,
    group_name: &str,
) -> TestResult {
    let start = Instant::now();
    let test_id = format!("{}/{}", group_name, test.name);

    // Try to validate the schema
    match engine.validate_schema(&test.schema_document) {
        Ok(valid) => {
            let actual_validity = if valid { Validity::Valid } else { Validity::Invalid };
            let outcome = if actual_validity == test.expected_validity {
                TestOutcome::Pass
            } else {
                TestOutcome::Fail(format!(
                    "Expected {:?}, got {:?}",
                    test.expected_validity, actual_validity
                ))
            };

            TestResult {
                test_id,
                test_set: test_set_name.to_string(),
                test_suite: "xsd".to_string(),
                description: Some(format!("Schema validation: {}", test.name)),
                outcome,
                expected: Some(format!("{:?}", test.expected_validity)),
                actual: Some(format!("{:?}", actual_validity)),
                duration: start.elapsed(),
            }
        }
        Err(e) => {
            // Error during validation - check if invalid was expected
            let outcome = if test.expected_validity == Validity::Invalid {
                TestOutcome::Pass
            } else {
                TestOutcome::Fail(format!("Schema validation error: {}", e))
            };

            TestResult {
                test_id,
                test_set: test_set_name.to_string(),
                test_suite: "xsd".to_string(),
                description: Some(format!("Schema validation: {}", test.name)),
                outcome,
                expected: Some(format!("{:?}", test.expected_validity)),
                actual: Some(format!("Error: {}", e)),
                duration: start.elapsed(),
            }
        }
    }
}

/// Run an instance validation test
fn run_instance_test(
    engine: &mut XEngine,
    test: &InstanceTest,
    schema_path: Option<&Path>,
    test_set_name: &str,
    group_name: &str,
) -> TestResult {
    let start = Instant::now();
    let test_id = format!("{}/{}", group_name, test.name);

    // If no schema, mark as not applicable
    let schema_path = match schema_path {
        Some(p) => p,
        None => {
            return TestResult {
                test_id,
                test_set: test_set_name.to_string(),
                test_suite: "xsd".to_string(),
                description: Some(format!("Instance validation: {}", test.name)),
                outcome: TestOutcome::NotApplicable,
                expected: None,
                actual: Some("No schema for validation".to_string()),
                duration: start.elapsed(),
            };
        }
    };

    // Validate instance against schema
    match engine.validate_instance(&test.instance_document, schema_path) {
        Ok(valid) => {
            let actual_validity = if valid { Validity::Valid } else { Validity::Invalid };
            let outcome = if actual_validity == test.expected_validity {
                TestOutcome::Pass
            } else {
                TestOutcome::Fail(format!(
                    "Expected {:?}, got {:?}",
                    test.expected_validity, actual_validity
                ))
            };

            TestResult {
                test_id,
                test_set: test_set_name.to_string(),
                test_suite: "xsd".to_string(),
                description: Some(format!("Instance validation: {}", test.name)),
                outcome,
                expected: Some(format!("{:?}", test.expected_validity)),
                actual: Some(format!("{:?}", actual_validity)),
                duration: start.elapsed(),
            }
        }
        Err(e) => {
            let outcome = if test.expected_validity == Validity::Invalid {
                TestOutcome::Pass
            } else {
                TestOutcome::Fail(format!("Validation error: {}", e))
            };

            TestResult {
                test_id,
                test_set: test_set_name.to_string(),
                test_suite: "xsd".to_string(),
                description: Some(format!("Instance validation: {}", test.name)),
                outcome,
                expected: Some(format!("{:?}", test.expected_validity)),
                actual: Some(format!("Error: {}", e)),
                duration: start.elapsed(),
            }
        }
    }
}

// ============== Public API ==============

/// Run XSD tests against an engine
pub fn run_xsd_tests(
    engine: &mut XEngine,
    suite_path: &Path,
    filter: Option<&str>,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    // Parse suite
    let suite = match parse_suite(suite_path) {
        Ok(s) => s,
        Err(e) => {
            results.push(TestResult {
                test_id: "suite_parse".to_string(),
                test_set: "suite".to_string(),
                test_suite: "xsd".to_string(),
                description: Some("Parse XSD test suite".to_string()),
                outcome: TestOutcome::Error(format!("Failed to parse suite: {}", e)),
                expected: None,
                actual: None,
                duration: std::time::Duration::ZERO,
            });
            return results;
        }
    };

    let base_dir = suite_path.parent().unwrap_or(Path::new("."));

    // Filter test sets
    let test_sets_to_run: Vec<_> = suite.test_set_refs.iter()
        .filter(|ts| {
            if let Some(f) = filter {
                ts.href.contains(f)
            } else {
                true
            }
        })
        .collect();
    let total_test_sets = test_sets_to_run.len();

    // Run each test set
    for (set_idx, test_set_ref) in test_sets_to_run.iter().enumerate() {
        let test_set_path = base_dir.join(&test_set_ref.href);
        let test_set_name = test_set_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        eprintln!("[{}/{}] Processing test set: {}", set_idx + 1, total_test_sets, test_set_name);

        // Parse test set with panic handling
        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| {
            parse_test_set(&test_set_path)
        }));

        let test_set = match parse_result {
            Ok(Ok(ts)) => ts,
            Ok(Err(e)) => {
                results.push(TestResult {
                    test_id: format!("{}/parse", test_set_name),
                    test_set: test_set_name.clone(),
                    test_suite: "xsd".to_string(),
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
                    test_set: test_set_name.clone(),
                    test_suite: "xsd".to_string(),
                    description: Some(format!("Parse test set {}", test_set_name)),
                    outcome: TestOutcome::Error(format!("Panic: {}", panic_msg)),
                    expected: None,
                    actual: Some("PANIC".to_string()),
                    duration: std::time::Duration::ZERO,
                });
                continue;
            }
        };

        // Run each test group
        for group in &test_set.test_groups {
            // Run schema test if present
            if let Some(schema_test) = &group.schema_test {
                let start = Instant::now();
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    run_schema_test(engine, schema_test, &test_set_name, &group.name)
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
                            test_id: format!("{}/{}", group.name, schema_test.name),
                            test_set: test_set_name.clone(),
                            test_suite: "xsd".to_string(),
                            description: Some(format!("Schema test: {}", schema_test.name)),
                            outcome: TestOutcome::Error(format!("Panic: {}", panic_msg)),
                            expected: None,
                            actual: Some("PANIC".to_string()),
                            duration: start.elapsed(),
                        }
                    }
                };
                results.push(test_result);
            }

            // Run instance tests
            let schema_path = group.schema_test.as_ref().map(|st| st.schema_document.as_path());

            for instance_test in &group.instance_tests {
                let start = Instant::now();
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    run_instance_test(engine, instance_test, schema_path, &test_set_name, &group.name)
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
                            test_id: format!("{}/{}", group.name, instance_test.name),
                            test_set: test_set_name.clone(),
                            test_suite: "xsd".to_string(),
                            description: Some(format!("Instance test: {}", instance_test.name)),
                            outcome: TestOutcome::Error(format!("Panic: {}", panic_msg)),
                            expected: None,
                            actual: Some("PANIC".to_string()),
                            duration: start.elapsed(),
                        }
                    }
                };
                results.push(test_result);
            }
        }
    }

    results
}

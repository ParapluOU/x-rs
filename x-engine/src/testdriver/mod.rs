//! Test drivers for W3C conformance testing
//!
//! This module provides test runners for:
//! - QT3 tests (XPath/XQuery)
//! - XSLT 3.0 tests
//! - XSD tests

pub mod qt3;
pub mod xslt30;
pub mod xsd;

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Result of running a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test identifier (unique within test set)
    pub test_id: String,
    /// Test set name (e.g., "fn-abs", "as", "suntest")
    pub test_set: String,
    /// Test suite name (e.g., "qt3", "xslt30", "xsd")
    pub test_suite: String,
    /// Human-readable description of the test
    pub description: Option<String>,
    /// Outcome of the test
    pub outcome: TestOutcome,
    /// Expected result (if applicable)
    pub expected: Option<String>,
    /// Actual result (if applicable)
    pub actual: Option<String>,
    /// Duration of test execution
    pub duration: Duration,
}

impl TestResult {
    /// Create a new test result with all metadata
    pub fn new(
        test_id: impl Into<String>,
        test_set: impl Into<String>,
        test_suite: impl Into<String>,
        description: Option<String>,
        outcome: TestOutcome,
        duration: Duration,
    ) -> Self {
        Self {
            test_id: test_id.into(),
            test_set: test_set.into(),
            test_suite: test_suite.into(),
            description,
            outcome,
            expected: None,
            actual: None,
            duration,
        }
    }

    /// Set expected/actual values
    pub fn with_values(mut self, expected: Option<String>, actual: Option<String>) -> Self {
        self.expected = expected;
        self.actual = actual;
        self
    }
}

/// Outcome of a test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOutcome {
    /// Test passed
    Pass,
    /// Test failed with reason
    Fail(String),
    /// Test errored with reason
    Error(String),
    /// Test not applicable (engine doesn't support this feature)
    NotApplicable,
    /// Test was skipped
    Skipped,
}

impl TestOutcome {
    pub fn is_pass(&self) -> bool {
        matches!(self, TestOutcome::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, TestOutcome::Fail(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, TestOutcome::Error(_))
    }

    /// Get a short string representation for CSV/table output
    pub fn as_str(&self) -> &str {
        match self {
            TestOutcome::Pass => "pass",
            TestOutcome::Fail(_) => "fail",
            TestOutcome::Error(_) => "error",
            TestOutcome::NotApplicable => "n/a",
            TestOutcome::Skipped => "skipped",
        }
    }

    /// Get the message if any
    pub fn message(&self) -> Option<&str> {
        match self {
            TestOutcome::Fail(msg) | TestOutcome::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

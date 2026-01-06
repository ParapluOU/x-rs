//! Test drivers for W3C conformance testing
//!
//! This module provides test runners for:
//! - QT3 tests (XPath/XQuery)
//! - XSLT 3.0 tests
//! - XSD tests

pub mod qt3;

use std::time::Duration;

/// Result of running a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test identifier
    pub test_id: String,
    /// Outcome of the test
    pub outcome: TestOutcome,
    /// Expected result (if applicable)
    pub expected: Option<String>,
    /// Actual result (if applicable)
    pub actual: Option<String>,
    /// Duration of test execution
    pub duration: Duration,
}

/// Outcome of a test
#[derive(Debug, Clone)]
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
}

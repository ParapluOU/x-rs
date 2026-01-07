//! Compliance report generation
//!
//! Generates reports showing how each engine performs against W3C specs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::testdriver::{TestOutcome, TestResult};

/// Summary of compliance test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub not_applicable: usize,
    pub skipped: usize,
    pub pass_rate: f64,
}

impl ComplianceSummary {
    pub fn from_results(results: &[TestResult]) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.outcome.is_pass()).count();
        let failed = results.iter().filter(|r| r.outcome.is_fail()).count();
        let errors = results.iter().filter(|r| r.outcome.is_error()).count();
        let not_applicable = results
            .iter()
            .filter(|r| matches!(r.outcome, TestOutcome::NotApplicable))
            .count();
        let skipped = results
            .iter()
            .filter(|r| matches!(r.outcome, TestOutcome::Skipped))
            .count();

        let applicable = total - not_applicable - skipped;
        let pass_rate = if applicable > 0 {
            (passed as f64 / applicable as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            passed,
            failed,
            errors,
            not_applicable,
            skipped,
            pass_rate,
        }
    }
}

/// A compliance report for a single engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub engine: String,
    pub timestamp: DateTime<Utc>,
    pub suite: String,
    pub summary: ComplianceSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<DetailedTestResult>,
}

/// Detailed test result for serialization (includes all metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedTestResult {
    pub test_id: String,
    pub test_set: String,
    pub test_suite: String,
    pub description: Option<String>,
    pub outcome: String,
    pub message: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub duration_ms: u64,
}

impl From<&TestResult> for DetailedTestResult {
    fn from(r: &TestResult) -> Self {
        let (outcome, message) = match &r.outcome {
            TestOutcome::Pass => ("pass".to_string(), None),
            TestOutcome::Fail(msg) => ("fail".to_string(), Some(msg.clone())),
            TestOutcome::Error(msg) => ("error".to_string(), Some(msg.clone())),
            TestOutcome::NotApplicable => ("n/a".to_string(), None),
            TestOutcome::Skipped => ("skipped".to_string(), None),
        };

        Self {
            test_id: r.test_id.clone(),
            test_set: r.test_set.clone(),
            test_suite: r.test_suite.clone(),
            description: r.description.clone(),
            outcome,
            message,
            expected: r.expected.clone(),
            actual: r.actual.clone(),
            duration_ms: r.duration.as_millis() as u64,
        }
    }
}

/// Simplified test result for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultSummary {
    pub test_id: String,
    pub outcome: String,
    pub message: Option<String>,
    pub duration_ms: u64,
}

impl From<&TestResult> for TestResultSummary {
    fn from(r: &TestResult) -> Self {
        let (outcome, message) = match &r.outcome {
            TestOutcome::Pass => ("pass".to_string(), None),
            TestOutcome::Fail(msg) => ("fail".to_string(), Some(msg.clone())),
            TestOutcome::Error(msg) => ("error".to_string(), Some(msg.clone())),
            TestOutcome::NotApplicable => ("not_applicable".to_string(), None),
            TestOutcome::Skipped => ("skipped".to_string(), None),
        };

        Self {
            test_id: r.test_id.clone(),
            outcome,
            message,
            duration_ms: r.duration.as_millis() as u64,
        }
    }
}

impl ComplianceReport {
    /// Create a new compliance report
    pub fn new(engine: &str, suite: &str, results: Vec<TestResult>) -> Self {
        let summary = ComplianceSummary::from_results(&results);
        let detailed_results = results.iter().map(DetailedTestResult::from).collect();

        Self {
            engine: engine.to_string(),
            timestamp: Utc::now(),
            suite: suite.to_string(),
            summary,
            results: detailed_results,
        }
    }

    /// Generate a markdown report
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {} Compliance Report\n\n", self.engine));
        md.push_str(&format!("**Suite:** {}\n", self.suite));
        md.push_str(&format!("**Date:** {}\n\n", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));

        md.push_str("## Summary\n\n");
        md.push_str("| Metric | Count |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Total | {} |\n", self.summary.total));
        md.push_str(&format!("| Passed | {} |\n", self.summary.passed));
        md.push_str(&format!("| Failed | {} |\n", self.summary.failed));
        md.push_str(&format!("| Errors | {} |\n", self.summary.errors));
        md.push_str(&format!("| Not Applicable | {} |\n", self.summary.not_applicable));
        md.push_str(&format!("| Skipped | {} |\n", self.summary.skipped));
        md.push_str(&format!("| **Pass Rate** | **{:.2}%** |\n\n", self.summary.pass_rate));

        if !self.results.is_empty() {
            md.push_str("## Failed Tests\n\n");
            let failed: Vec<_> = self.results.iter().filter(|r| r.outcome == "fail" || r.outcome == "error").collect();

            if failed.is_empty() {
                md.push_str("No failed tests!\n\n");
            } else {
                md.push_str("| Test Set | Test ID | Outcome | Message |\n");
                md.push_str("|----------|---------|---------|--------|\n");
                for r in failed.iter().take(100) {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        r.test_set,
                        r.test_id,
                        r.outcome,
                        r.message.as_deref().unwrap_or("-").chars().take(50).collect::<String>()
                    ));
                }
                if failed.len() > 100 {
                    md.push_str(&format!("\n... and {} more failed tests\n", failed.len() - 100));
                }
            }
        }

        md
    }

    /// Generate a JSON report
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate a CSV report with all test results
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("test_suite,test_set,test_id,description,outcome,message,duration_ms\n");

        // Data rows
        for r in &self.results {
            let desc = r.description.as_deref().unwrap_or("").replace('"', "\"\"");
            let msg = r.message.as_deref().unwrap_or("").replace('"', "\"\"");
            csv.push_str(&format!(
                "{},{},{},\"{}\",{},\"{}\",{}\n",
                r.test_suite,
                r.test_set,
                r.test_id,
                desc,
                r.outcome,
                msg,
                r.duration_ms
            ));
        }

        csv
    }
}

/// Comparison report across multiple engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub timestamp: DateTime<Utc>,
    pub suite: String,
    pub engines: Vec<EngineSummary>,
}

/// Summary for a single engine in a comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSummary {
    pub name: String,
    pub passed: usize,
    pub total: usize,
    pub pass_rate: f64,
}

/// Compare reports from multiple engines
pub fn compare_reports(reports: &[ComplianceReport]) -> ComparisonReport {
    let suite = reports.first().map(|r| r.suite.clone()).unwrap_or_default();

    let engines = reports
        .iter()
        .map(|r| EngineSummary {
            name: r.engine.clone(),
            passed: r.summary.passed,
            total: r.summary.total,
            pass_rate: r.summary.pass_rate,
        })
        .collect();

    ComparisonReport {
        timestamp: Utc::now(),
        suite,
        engines,
    }
}

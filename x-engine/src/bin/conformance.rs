//! Conformance testing CLI tool
//!
//! Run W3C conformance tests against xee, xrust, and xust engines.

use std::path::PathBuf;
use std::process;

use x_engine::reporter::ComplianceReport;
use x_engine::testdriver::qt3::run_xpath_tests;
use x_engine::{Backend, XEngine};

fn print_usage() {
    eprintln!("x-engine conformance testing tool");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  conformance run --engine <ENGINE> --suite <SUITE> [--filter <PATTERN>]");
    eprintln!("  conformance report --engine <ENGINE> --suite <SUITE> --output <FORMAT>");
    eprintln!();
    eprintln!("Engines: xee, xrust, xust");
    eprintln!("Suites: qt3");
    eprintln!("Output formats: json, markdown");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  conformance run --engine xee --suite qt3");
    eprintln!("  conformance run --engine xee --suite qt3 --filter fn-abs");
    eprintln!("  conformance report --engine xee --suite qt3 --output markdown");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "run" => run_tests(&args[2..]),
        "report" => run_report(&args[2..]),
        "--help" | "-h" => {
            print_usage();
            process::exit(0);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let mut engine = None;
    let mut suite = None;
    let mut filter = None;
    let mut output = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--engine" | "-e" => {
                if i + 1 < args.len() {
                    engine = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--suite" | "-s" => {
                if i + 1 < args.len() {
                    suite = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--filter" | "-f" => {
                if i + 1 < args.len() {
                    filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    (engine, suite, filter, output)
}

fn get_engine(name: &str) -> Option<XEngine> {
    match name {
        "xee" => Some(XEngine::with_backend(Backend::Xee)),
        "xrust" => Some(XEngine::with_backend(Backend::Xrust)),
        "xust" => Some(XEngine::with_backend(Backend::Xust)),
        _ => None,
    }
}

fn get_catalog_path(suite: &str) -> Option<PathBuf> {
    // Try to find the catalog relative to the workspace root
    let potential_paths = [
        // From the x-engine directory
        "../tests/qt3tests/catalog.xml",
        // From the workspace root
        "tests/qt3tests/catalog.xml",
        // Absolute path for testing
        "/Users/luukdewaalmalefijt/Code/paraplu/x-rs/tests/qt3tests/catalog.xml",
    ];

    match suite {
        "qt3" => {
            for path in &potential_paths {
                let p = PathBuf::from(path);
                if p.exists() {
                    return Some(p);
                }
            }
            // Default to the first one even if it doesn't exist
            Some(PathBuf::from(potential_paths[0]))
        }
        _ => None,
    }
}

fn run_tests(args: &[String]) {
    let (engine_name, suite, filter, _) = parse_args(args);

    let engine_name = match engine_name {
        Some(e) => e,
        None => {
            eprintln!("Error: --engine is required");
            process::exit(1);
        }
    };

    let suite = match suite {
        Some(s) => s,
        None => {
            eprintln!("Error: --suite is required");
            process::exit(1);
        }
    };

    let mut engine = match get_engine(&engine_name) {
        Some(e) => e,
        None => {
            eprintln!("Error: Unknown engine '{}'. Use xee, xrust, or xust.", engine_name);
            process::exit(1);
        }
    };

    let catalog_path = match get_catalog_path(&suite) {
        Some(p) => p,
        None => {
            eprintln!("Error: Unknown suite '{}'. Use qt3.", suite);
            process::exit(1);
        }
    };

    if !catalog_path.exists() {
        eprintln!("Error: Catalog not found at {:?}", catalog_path);
        eprintln!("Make sure you're running from the workspace root or x-engine directory.");
        process::exit(1);
    }

    println!("Running {} tests with {} engine...", suite, engine_name);
    println!("Catalog: {:?}", catalog_path);
    if let Some(ref f) = filter {
        println!("Filter: {}", f);
    }
    println!();

    let results = run_xpath_tests(&mut engine, &catalog_path, filter.as_deref());

    // Print summary
    let total = results.len();
    let passed = results.iter().filter(|r| r.outcome.is_pass()).count();
    let failed = results.iter().filter(|r| r.outcome.is_fail()).count();
    let errors = results.iter().filter(|r| r.outcome.is_error()).count();
    let not_applicable = results
        .iter()
        .filter(|r| matches!(r.outcome, x_engine::testdriver::TestOutcome::NotApplicable))
        .count();

    println!("Results:");
    println!("  Total:          {}", total);
    println!("  Passed:         {} ({:.1}%)", passed, if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 });
    println!("  Failed:         {}", failed);
    println!("  Errors:         {}", errors);
    println!("  Not Applicable: {}", not_applicable);
    println!();

    // Print first few failures
    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.outcome.is_fail() || r.outcome.is_error())
        .take(10)
        .collect();

    if !failures.is_empty() {
        println!("First {} failures:", failures.len());
        for r in failures {
            println!("  {}: {:?}", r.test_id, r.outcome);
        }
        println!();
    }
}

fn run_report(args: &[String]) {
    let (engine_name, suite, filter, output_format) = parse_args(args);

    let engine_name = match engine_name {
        Some(e) => e,
        None => {
            eprintln!("Error: --engine is required");
            process::exit(1);
        }
    };

    let suite = match suite {
        Some(s) => s,
        None => {
            eprintln!("Error: --suite is required");
            process::exit(1);
        }
    };

    let output_format = output_format.unwrap_or_else(|| "markdown".to_string());

    let mut engine = match get_engine(&engine_name) {
        Some(e) => e,
        None => {
            eprintln!("Error: Unknown engine '{}'. Use xee, xrust, or xust.", engine_name);
            process::exit(1);
        }
    };

    let catalog_path = match get_catalog_path(&suite) {
        Some(p) => p,
        None => {
            eprintln!("Error: Unknown suite '{}'. Use qt3.", suite);
            process::exit(1);
        }
    };

    if !catalog_path.exists() {
        eprintln!("Error: Catalog not found at {:?}", catalog_path);
        process::exit(1);
    }

    eprintln!("Running {} tests with {} engine...", suite, engine_name);

    let results = run_xpath_tests(&mut engine, &catalog_path, filter.as_deref());
    let report = ComplianceReport::new(&engine_name, &suite, results);

    match output_format.as_str() {
        "json" => println!("{}", report.to_json()),
        "markdown" | "md" => println!("{}", report.to_markdown()),
        _ => {
            eprintln!("Error: Unknown output format '{}'. Use json or markdown.", output_format);
            process::exit(1);
        }
    }
}

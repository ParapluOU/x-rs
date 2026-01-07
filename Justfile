# x-rs Justfile - Unified XML Engine Testing

# Default recipe
default:
    @just --list

# Build the conformance test runner in release mode
build:
    cargo build --release -p x-engine

# ==================== QT3 (XPath/XQuery) Tests ====================

# Run QT3 tests for a single engine
# Usage: just qt3 xee
qt3 engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3

# Run QT3 tests with filter
# Usage: just qt3-filter xee fn-abs
qt3-filter engine filter:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3 --filter "{{filter}}"

# Run QT3 tests and output JSON
qt3-json engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3 --output json

# Run QT3 tests and output CSV
qt3-csv engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3 --output csv

# ==================== XSLT 3.0 Tests ====================

# Run XSLT 3.0 tests for a single engine
# Usage: just xslt30 xee
xslt30 engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite xslt30

# Run XSLT 3.0 tests with filter
xslt30-filter engine filter:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite xslt30 --filter "{{filter}}"

# ==================== XSD Tests ====================

# Run XSD tests (only xust supports XSD)
# Usage: just xsd xust
xsd engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite xsd

# Run XSD tests with filter
xsd-filter engine filter:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite xsd --filter "{{filter}}"

# ==================== Full Test Runs ====================

# Run all QT3 tests for all engines in parallel (to /tmp)
qt3-all:
    @echo "Starting QT3 tests for all engines..."
    @mkdir -p results
    cargo run --release -p x-engine --bin conformance -- run --engine xee --suite qt3 > /tmp/xee_results.txt 2>&1 &
    cargo run --release -p x-engine --bin conformance -- run --engine xust --suite qt3 > /tmp/xust_results.txt 2>&1 &
    cargo run --release -p x-engine --bin conformance -- run --engine xrust --suite qt3 > /tmp/xrust_results.txt 2>&1 &
    @echo "All engines started. Monitor with: just progress"

# Run all QT3 tests sequentially and save detailed results
qt3-all-detailed:
    @echo "Running QT3 tests with detailed output..."
    @mkdir -p results/qt3
    cargo run --release -p x-engine --bin conformance -- run --engine xee --suite qt3 --output json > results/qt3/xee.json 2>/dev/null
    cargo run --release -p x-engine --bin conformance -- run --engine xust --suite qt3 --output json > results/qt3/xust.json 2>/dev/null
    cargo run --release -p x-engine --bin conformance -- run --engine xrust --suite qt3 --output json > results/qt3/xrust.json 2>/dev/null
    @echo "Results saved to results/qt3/"

# ==================== Quick Tests ====================

# Quick test - run a subset of QT3 tests
quick-test engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3 --filter "fn-abs"

# Quick test all engines with a single test set
quick-test-all filter="fn-abs":
    @echo "Quick test with filter: {{filter}}"
    @echo "=== xee ===" && cargo run --release -p x-engine --bin conformance -- run --engine xee --suite qt3 --filter "{{filter}}"
    @echo "=== xust ===" && cargo run --release -p x-engine --bin conformance -- run --engine xust --suite qt3 --filter "{{filter}}"
    @echo "=== xrust ===" && cargo run --release -p x-engine --bin conformance -- run --engine xrust --suite qt3 --filter "{{filter}}"

# ==================== Progress & Utilities ====================

# Show current test progress (from /tmp results)
progress:
    @echo "=== Current Progress ==="
    @echo "xee:" && tail -1 /tmp/xee_results.txt 2>/dev/null || echo "  No results yet"
    @echo "xust:" && tail -1 /tmp/xust_results.txt 2>/dev/null || echo "  No results yet"
    @echo "xrust:" && tail -1 /tmp/xrust_results.txt 2>/dev/null || echo "  No results yet"

# Clean build artifacts
clean:
    cargo clean -p x-engine

# Run unit tests for x-engine
unit-test:
    cargo test -p x-engine

# Check code compiles
check:
    cargo check -p x-engine

# ==================== Legacy Aliases ====================

# Alias for qt3 (backward compatibility)
test engine:
    @just qt3 {{engine}}

test-filter engine filter:
    @just qt3-filter {{engine}} {{filter}}

test-all:
    @just qt3-all

test-all-seq:
    @just qt3-all-detailed

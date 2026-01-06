# x-rs Justfile - Unified XML Engine Testing

# Default recipe
default:
    @just --list

# Build the conformance test runner in release mode
build:
    cargo build --release -p x-engine

# Run conformance tests for a single engine
# Usage: just test xee
test engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3

# Run conformance tests for a single engine with filter
# Usage: just test-filter xee fn-abs
test-filter engine filter:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3 --filter "{{filter}}"

# Run conformance tests for all engines in parallel
test-all:
    @echo "Starting conformance tests for all engines..."
    @mkdir -p results
    @echo "Running xee..."
    cargo run --release -p x-engine --bin conformance -- run --engine xee --suite qt3 > results/xee.txt 2>&1 &
    @echo "Running xust..."
    cargo run --release -p x-engine --bin conformance -- run --engine xust --suite qt3 > results/xust.txt 2>&1 &
    @echo "Running xrust..."
    cargo run --release -p x-engine --bin conformance -- run --engine xrust --suite qt3 > results/xrust.txt 2>&1 &
    @echo "All engines started. Check results/ directory for output."
    @echo "Monitor progress with: tail -f results/*.txt | grep Processing"

# Run conformance tests for all engines sequentially (better for comparison)
test-all-seq:
    @echo "Starting conformance tests sequentially..."
    @mkdir -p results
    @echo "Running xee..."
    cargo run --release -p x-engine --bin conformance -- run --engine xee --suite qt3 > results/xee.txt 2>&1
    @echo "Running xust..."
    cargo run --release -p x-engine --bin conformance -- run --engine xust --suite qt3 > results/xust.txt 2>&1
    @echo "Running xrust..."
    cargo run --release -p x-engine --bin conformance -- run --engine xrust --suite qt3 > results/xrust.txt 2>&1
    @echo "All tests complete."

# Generate comparison report from existing results
compare:
    cargo run --release -p x-engine --bin conformance -- compare \
        --xee results/xee.txt \
        --xust results/xust.txt \
        --xrust results/xrust.txt

# Run tests and generate comparison (sequential)
full-conformance:
    @just build
    @just test-all-seq
    @just compare

# Quick test - run a subset of tests for validation
quick-test engine:
    cargo run --release -p x-engine --bin conformance -- run --engine {{engine}} --suite qt3 --filter "fn-abs"

# Quick test all engines with a single test set
quick-test-all filter="fn-abs":
    @echo "Quick test with filter: {{filter}}"
    @echo "=== xee ===" && cargo run --release -p x-engine --bin conformance -- run --engine xee --suite qt3 --filter "{{filter}}"
    @echo "=== xust ===" && cargo run --release -p x-engine --bin conformance -- run --engine xust --suite qt3 --filter "{{filter}}"
    @echo "=== xrust ===" && cargo run --release -p x-engine --bin conformance -- run --engine xrust --suite qt3 --filter "{{filter}}"

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

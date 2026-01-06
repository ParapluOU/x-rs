# W3C QT3 Conformance Test Results

**Test Date:** 2026-01-06
**Test Suite:** QT3 (XPath 3.1 / XQuery 3.1 Functions and Operators)
**Total Tests:** 31,821

## Summary

| Engine | Version | Passed | Failed | Errors | N/A | Pass Rate |
|--------|---------|--------|--------|--------|-----|-----------|
| **xee** | XPath 3.1 | 19,687 | 5,252 | 53 | 6,829 | **61.9%** |
| **xust** | XQuery 3.1 | 11,623 | 14,792 | 5 | 5,401 | **36.5%** |
| **xrust** | XPath 1.0-ish | 4,920 | 18,235 | 178 | 8,488 | **15.5%** |

## Analysis

### xee (Best Performer)
- **Strengths:** Most complete XPath 3.1 implementation
- **Notable:** 53 errors (mostly engine panics caught by test harness)
- **Common failures:** Float/double formatting differences (scientific notation)

### xust
- **Strengths:** Few runtime errors (5 total)
- **Issues:** Many type-related failures, XPST errors on some types
- **Notable:** XQuery-based engine via XPath subset

### xrust
- **Strengths:** Basic XPath functionality works
- **Issues:** Many "no namespace for prefix" errors (namespace handling)
- **Notable:** Debug output (parser tracing) enabled in production builds

## Detailed Failure Analysis

### Common Failure Patterns

1. **Float/Double formatting** - All engines struggle with exact scientific notation formatting
2. **Type casting** - xs:integer, xs:decimal precision differences
3. **Namespace handling** - xrust has significant namespace resolution issues
4. **Higher-order functions** - Limited support in xrust

## Test Categories Breakdown

The QT3 test suite covers 428 test sets across these categories:
- `fn-*` - Function tests (abs, concat, contains, etc.)
- `op-*` - Operator tests (arithmetic, comparison, etc.)
- `prod-*` - Production tests (expressions, types, etc.)
- `misc-*` - Miscellaneous tests
- `app-*` - Application tests (FunctX, use cases, etc.)

## Running Tests

```bash
# Run all engines in parallel
just test-all

# Run single engine
just test xee

# Run specific test set
just test-filter xee fn-abs

# Check progress
just progress

# Generate comparison (after test-all completes)
just compare
```

## Methodology

Tests are run using the x-engine unified wrapper which:
1. Parses the QT3 catalog using XPath
2. Runs each test case against the specified engine
3. Compares results against expected outcomes
4. Handles engine panics gracefully via `catch_unwind`

Runtime: ~2 hours for all 3 engines (dominated by large test sets like `prod-CastExpr` with 5,556 tests)

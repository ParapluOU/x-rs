# x-rs

A collection of Rust XML/XPath/XSLT/XQuery libraries and W3C conformance test suites.

## Libraries

| Library | Description | Version | Status |
|---------|-------------|---------|--------|
| [xee](./xee) | XPath 3.1 / XSLT 3.0 implementation | 0.1.6 | Active |
| [xot](./xot) | XML tree manipulation library | 0.31.2 | Active |
| [xrust](./xrust) | XPath/XSLT processor | 1.2.4 | Active |
| [xust](./xust) | XQuery 3.1 / XSD 1.1 implementation | 0.1.0 | Early dev |

## Feature Comparison

### XPath Support

| Feature | xee | xrust | xust |
|---------|-----|-------|------|
| **XPath 1.0** | Via 3.1 | ~Equivalent | Via 3.1 |
| **XPath 2.0** | Via 3.1 | Partial (avg/min/max) | Via 3.1 |
| **XPath 3.0** | Via 3.1 | No | Via 3.1 |
| **XPath 3.1** | Yes | No | Yes |
| Functions (1.0) | All | All | All |
| Functions (2.0+) | ~150 implemented | 3 only | 100+ |
| FLWOR expressions | Yes | Yes | Yes |
| Maps & Arrays | Yes | No | Yes |
| Higher-order functions | Yes | No | Yes |

### XSLT Support

| Feature | xee | xrust | xust |
|---------|-----|-------|------|
| **XSLT 1.0** | Via 3.0 | ~Equivalent | No |
| **XSLT 2.0** | Via 3.0 | Partial | No |
| **XSLT 3.0** | Partial | No | No |
| Basic templates | Yes | Yes | - |
| xsl:for-each-group | No | Yes | - |
| xsl:function | No | Yes | - |
| Streaming | No | No | - |
| Modes | No | No | - |

### XQuery Support

| Feature | xee | xrust | xust |
|---------|-----|-------|------|
| **XQuery 1.0** | No | No | Via 3.1 |
| **XQuery 3.0** | No | No | Via 3.1 |
| **XQuery 3.1** | No | No | Yes |
| Module system | - | - | Yes |
| FLWOR expressions | - | - | Yes |

### XML Schema (XSD) Support

| Feature | xee | xrust | xust |
|---------|-----|-------|------|
| **XSD 1.0** | No | No | Via 1.1 |
| **XSD 1.1** | No | No | Yes |
| Schema validation | No | DTD only | Yes |
| Type system | Basic xs:* types | Basic types | Full |

### XML Parsing

| Feature | xee | xot | xrust | xust |
|---------|-----|-----|-------|------|
| XML 1.0 | Yes | Yes | Yes | Yes |
| Namespaces | Yes | Yes | Yes | Yes |
| DTD | No | No | Yes | Yes |
| Entity handling | Yes | Yes | Yes | Yes |

## Conformance Test Results

### XPath/XQuery (QT3 Test Suite)

| Library | Tests Run | Passed | Pass Rate |
|---------|-----------|--------|-----------|
| xee | 21,859 | 19,987 | 91% |
| xrust | - | - | - |
| xust | - | - | - |

### XML Schema (XSD Test Suite)

| Library | Tests Run | Status |
|---------|-----------|--------|
| xust | 38,530 | In progress |

### XML (Conformance Suite)

| Library | Tests Run | Status |
|---------|-----------|--------|
| xust | 2,585 | 10 failures |

## Recommendation Guide

| Use Case | Recommended | Reason |
|----------|-------------|--------|
| XPath 3.1 queries | **xee** | Best conformance (91%), most functions |
| XSLT 1.0 transforms | **xrust** | More XSLT features (grouping, functions) |
| XQuery processing | **xust** | Only XQuery implementation |
| XSD validation | **xust** | Only XSD 1.1 implementation |
| XML tree manipulation | **xot** | Lightweight, focused API |
| Simple XPath 1.0 | **xrust** | Stable, good performance |

## Test Suites

W3C conformance test suites included as submodules:

| Suite | Path | Description |
|-------|------|-------------|
| QT3 Tests | [tests/qt3tests](./tests/qt3tests) | XPath/XQuery 3.x (26,000+ tests) |
| XSLT 3.0 Tests | [tests/xslt30-test](./tests/xslt30-test) | XSLT 3.0 conformance |
| XSD Tests | [tests/xsdtests](./tests/xsdtests) | XML Schema test suite |

## Structure

```
x-rs/
├── xee/                    # XPath 3.1 / XSLT 3.0
├── xot/                    # XML tree library
├── xrust/                  # XPath/XSLT processor
├── xust/                   # XQuery 3.1 / XSD 1.1
└── tests/
    ├── qt3tests/           # W3C XPath/XQuery tests
    ├── xslt30-test/        # W3C XSLT 3.0 tests
    └── xsdtests/           # W3C XSD tests
```

## License

Each library maintains its own license. See individual directories for details.

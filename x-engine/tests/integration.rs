//! Integration tests for x-engine
//!
//! Tests basic functionality of all three engine wrappers.

use x_engine::engine_xee::XeeEngine;
use x_engine::engine_xrust::XrustEngine;
use x_engine::engine_xust::XustEngine;
use x_engine::traits::{QueryResult, XPathEngine, XQueryEngine, XmlParser, XsltEngine};

const SIMPLE_XML: &str = r#"<?xml version="1.0"?>
<root>
    <item id="1">First</item>
    <item id="2">Second</item>
    <item id="3">Third</item>
</root>"#;

// ============== XeeEngine Tests ==============

#[test]
fn xee_parse_xml() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse(SIMPLE_XML);
    assert!(doc.is_ok(), "XeeEngine should parse valid XML");
}

#[test]
fn xee_xpath_count() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.evaluate_xpath(&doc, "count(//item)").unwrap();
    assert_eq!(result.count(), 1, "count() should return a single value");
    // The result should be 3
    let items = result.items();
    assert!(!items.is_empty());
}

#[test]
fn xee_xpath_string() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.evaluate_xpath(&doc, "//item[@id='1']/text()").unwrap();
    assert!(!result.is_empty(), "XPath should find matching text");
}

#[test]
fn xee_xpath_select_nodes() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.evaluate_xpath(&doc, "//item").unwrap();
    assert_eq!(result.count(), 3, "Should find 3 item elements");
}

// ============== XrustEngine Tests ==============

#[test]
fn xrust_parse_xml() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse(SIMPLE_XML);
    assert!(doc.is_ok(), "XrustEngine should parse valid XML");
}

#[test]
fn xrust_xpath_count() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.evaluate_xpath(&doc, "count(//item)").unwrap();
    assert_eq!(result.count(), 1, "count() should return a single value");
}

#[test]
fn xrust_xpath_select_nodes() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.evaluate_xpath(&doc, "//item").unwrap();
    assert_eq!(result.count(), 3, "Should find 3 item elements");
}

#[test]
fn xrust_xslt_identity() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse("<root>Hello</root>").unwrap();
    let stylesheet = r#"<?xml version="1.0"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
    <xsl:template match="@*|node()">
        <xsl:copy>
            <xsl:apply-templates select="@*|node()"/>
        </xsl:copy>
    </xsl:template>
</xsl:stylesheet>"#;
    let result = engine.transform(&doc, stylesheet);
    assert!(result.is_ok(), "XrustEngine should handle identity XSLT");
}

// ============== XustEngine Tests ==============

#[test]
fn xust_parse_xml() {
    let mut engine = XustEngine::new();
    let doc = engine.parse(SIMPLE_XML);
    assert!(doc.is_ok(), "XustEngine should parse valid XML");
}

#[test]
fn xust_xpath_count() {
    let mut engine = XustEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    // XPath in xust is via XQuery
    let result = engine.evaluate_xpath(&doc, "count(//item)").unwrap();
    assert_eq!(result.count(), 1, "count() should return a single value");
}

#[test]
fn xust_xquery_flwor() {
    let mut engine = XustEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.execute_xquery(&doc, "for $i in //item return $i/@id").unwrap();
    assert_eq!(result.count(), 3, "XQuery FLWOR should find 3 ids");
}

#[test]
fn xust_xpath_select_nodes() {
    let mut engine = XustEngine::new();
    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.evaluate_xpath(&doc, "//item").unwrap();
    assert_eq!(result.count(), 3, "Should find 3 item elements");
}

// ============== Error Handling Tests ==============

#[test]
fn xee_invalid_xml() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse("<root><unclosed>");
    assert!(doc.is_err(), "Should fail on invalid XML");
}

#[test]
fn xrust_invalid_xml() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse("<root><unclosed>");
    assert!(doc.is_err(), "Should fail on invalid XML");
}

#[test]
fn xust_invalid_xml() {
    let mut engine = XustEngine::new();
    let doc = engine.parse("<root><unclosed>");
    assert!(doc.is_err(), "Should fail on invalid XML");
}

#[test]
fn xee_invalid_xpath() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.evaluate_xpath(&doc, "//[invalid");
    assert!(result.is_err(), "Should fail on invalid XPath");
}

#[test]
fn xrust_invalid_xpath() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.evaluate_xpath(&doc, "//[invalid");
    assert!(result.is_err(), "Should fail on invalid XPath");
}

#[test]
fn xust_invalid_xpath() {
    let mut engine = XustEngine::new();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.evaluate_xpath(&doc, "//[invalid");
    assert!(result.is_err(), "Should fail on invalid XPath");
}

// ============== Unsupported Feature Tests ==============

#[test]
fn xee_xquery_unsupported() {
    let mut engine = XeeEngine::new();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.execute_xquery(&doc, "//root");
    assert!(result.is_err(), "XeeEngine should not support XQuery");
}

#[test]
fn xrust_xquery_unsupported() {
    let mut engine = XrustEngine::new();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.execute_xquery(&doc, "//root");
    assert!(result.is_err(), "XrustEngine should not support XQuery");
}

#[test]
fn xust_xslt_unsupported() {
    let mut engine = XustEngine::new();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.transform(&doc, "<xsl:stylesheet/>");
    assert!(result.is_err(), "XustEngine should not support XSLT");
}

// ============== Unified XEngine API Tests ==============

use x_engine::{Backend, XEngine};

#[test]
fn unified_xee_parse_and_xpath() {
    let mut engine = XEngine::xee();
    assert_eq!(engine.backend(), Backend::Xee);

    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.xpath(&doc, "count(//item)").unwrap();
    assert_eq!(result.count(), 1);
}

#[test]
fn unified_xrust_parse_and_xpath() {
    let mut engine = XEngine::xrust();
    assert_eq!(engine.backend(), Backend::Xrust);

    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.xpath(&doc, "//item").unwrap();
    assert_eq!(result.count(), 3);
}

#[test]
fn unified_xust_parse_and_xquery() {
    let mut engine = XEngine::xust();
    assert_eq!(engine.backend(), Backend::Xust);

    let doc = engine.parse(SIMPLE_XML).unwrap();
    let result = engine.xquery(&doc, "for $i in //item return $i/@id").unwrap();
    assert_eq!(result.count(), 3);
}

#[test]
fn unified_with_backend() {
    let mut engine = XEngine::with_backend(Backend::Xee);
    assert_eq!(engine.backend(), Backend::Xee);

    let doc = engine.parse("<root>test</root>").unwrap();
    let result = engine.xpath(&doc, "/root/text()").unwrap();
    assert!(!result.is_empty());
}

#[test]
fn unified_xslt_with_xrust() {
    let mut engine = XEngine::xrust();
    let doc = engine.parse("<root>Hello</root>").unwrap();
    let stylesheet = r#"<?xml version="1.0"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
    <xsl:template match="@*|node()">
        <xsl:copy>
            <xsl:apply-templates select="@*|node()"/>
        </xsl:copy>
    </xsl:template>
</xsl:stylesheet>"#;
    let result = engine.xslt(&doc, stylesheet);
    assert!(result.is_ok(), "Unified xrust should support XSLT");
}

#[test]
fn unified_xquery_unsupported_on_xee() {
    let mut engine = XEngine::xee();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.xquery(&doc, "//root");
    assert!(result.is_err(), "Xee should not support XQuery via unified API");
}

#[test]
fn unified_xslt_unsupported_on_xust() {
    let mut engine = XEngine::xust();
    let doc = engine.parse("<root/>").unwrap();
    let result = engine.xslt(&doc, "<xsl:stylesheet/>");
    assert!(result.is_err(), "Xust should not support XSLT via unified API");
}

#[test]
fn unified_document_engine_mismatch() {
    let mut xee = XEngine::xee();
    let mut xrust = XEngine::xrust();

    let xee_doc = xee.parse("<root/>").unwrap();

    // Trying to use xee document with xrust engine should fail
    let result = xrust.xpath(&xee_doc, "//root");
    assert!(result.is_err(), "Should fail when document and engine mismatch");
}

#[test]
fn unified_default_is_xee() {
    let engine = XEngine::default();
    assert_eq!(engine.backend(), Backend::Xee);
}

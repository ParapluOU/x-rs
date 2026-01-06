//! Basic tests for xee-adapter

use xml_engine_traits::{tree::XmlTree, xpath::XPathEngine};
use xee_adapter::{XeeEngine, XotTreeWrapper};

#[test]
fn test_xot_wrapper_parse_xml() {
    let mut tree = XotTreeWrapper::new();
    let doc = tree.parse_xml("<root><item>test</item></root>").unwrap();

    // Get document element
    let root = tree.document_element(&doc).unwrap();

    // Check node name
    assert_eq!(tree.node_name(&root), Some("root".to_string()));

    // Get children
    let children = tree.children(&root);
    assert_eq!(children.len(), 1);

    // Check child name
    assert_eq!(tree.node_name(&children[0]), Some("item".to_string()));
}

#[test]
fn test_xot_wrapper_serialize() {
    let mut tree = XotTreeWrapper::new();
    let doc = tree.parse_xml("<root><item>test</item></root>").unwrap();

    let serialized = tree.serialize_document(&doc).unwrap();
    assert!(serialized.contains("root"));
    assert!(serialized.contains("item"));
    assert!(serialized.contains("test"));
}

#[test]
fn test_xee_engine_compile_xpath() {
    let engine = XeeEngine::new();

    // Compile a simple XPath expression
    let query = engine.compile_xpath("/root/item").unwrap();

    // Just checking that compilation works
    assert!(true);
}

#[test]
fn test_xee_engine_version() {
    let engine = XeeEngine::new();
    assert_eq!(engine.xpath_version(), "3.1");
}

#[test]
fn test_xee_engine_supported_features() {
    let engine = XeeEngine::new();
    let features = engine.supported_features();

    assert!(features.contains(&"xpath-3.1".to_string()));
    assert!(features.contains(&"higher-order-functions".to_string()));
}

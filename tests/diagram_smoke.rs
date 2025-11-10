// tests/diagram_smoke.rs
//! Minimal integration‑test proving that the parser compiles and registers a
//! single class in the *default namespace*.
//!
//! **Adjust the `mycrate` import path** to match the `[package]` name in your
//! `Cargo.toml` if it differs.

use mermaid_parser::types::DEFAULT_NAMESPACE;

#[test]
fn single_class_in_default_namespace() {
    let mermaid = r#"classDiagram
class Amy
class Chrissy
class Tom
class Marc
"#;

    let diagram = mermaid_parser::parserv2::parse_mermaid(mermaid).unwrap();
    println!("{:?}", diagram);
    let default_ns = diagram
        .namespaces
        .get(DEFAULT_NAMESPACE)
        .expect("default namespace should exist");

    assert!(default_ns.classes.contains_key("Chrissy"));
}

#[cfg(test)]
mod tests {
    use mermaid_parser::types::DEFAULT_NAMESPACE;
    #[test]
    fn parse_class_with_members() {
        let mermaid = include_str!("./mermaid/test.mmd");

        let diagram = mermaid_parser::parserv2::parse_mermaid(mermaid).unwrap();
        let ns = diagram.namespaces.get(DEFAULT_NAMESPACE).unwrap();
        println!("{:?}", diagram);
    }
}

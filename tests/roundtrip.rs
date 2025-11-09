#[cfg(test)]
mod tests {
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "../src/grammar/mermaid.pest"]
    pub struct ExplicitClassParser;

    #[test]
    fn test_valid_class_names() {
        let valid_names = vec![
            "Animal",
            "MyClass",
            "my_class",
            "my-class",
            "Fahrzeug123",
            "A_very_long_class_name_with_123_and_dashes",
            "`Animal Class!`",
            "`Car Class`",
            "`Test-Class!@#`",
            "`My Test Class`",
        ];
        for name in valid_names {
            let class_string = format!("class {}", name);
            let parsed = ExplicitClassParser::parse(Rule::test_class_explicit, &class_string);
            println!("{:#?}", parsed);
            assert!(
                parsed.is_ok(),
                "Parser failed to accept valid class name: '{}'", name
            );
        }
    }

    #[test]
    fn test_invalid_class_names() {
        let invalid_names = vec![
            "My Class", "!MyClass", "Class@Name", "#hashtag", "My$Class", "-StartingDash", "\"QuotedClass\"", "-", "Super Car", "Car!", "1startNumber"
        ];
        for name in invalid_names {
            let class_string = format!("class {}", name);
            let parsed = ExplicitClassParser::parse(Rule::test_class_explicit, &class_string);
            println!("{:#?}", parsed);
            assert!(parsed.is_err());
        }
    }

    #[test]
    fn test_valid_class_labels() {
        let valid_labels = vec![
            r#"class Animal["A simple label"]"#,
            r#"class Animal["Label with numbers 123"]"#,
            r#"class Animal["Special characters !@#$%^&*()"]"#,
            r#"class Animal["Label with: punctuation, dots. and commas,"]"#,
            r#"class Animal[""]"#, // Empty label
            r#"class Animal["Label with \n newline"]"#,
            r#"class Animal["Label with [brackets] and {braces}"]"#,
            r#"class Animal["Label with /slashes/ and \\backslashes\\"]"#,
            r#"class Animal["Label with emoji 🚀👍"]"#,
        ];
        for case in valid_labels {
            let parsed = ExplicitClassParser::parse(Rule::test_class_labels, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_ok());
        }
    }

    #[test]
    fn test_invalid_class_labels() {
        let invalid_labels = vec![
            r#"class Animal[No quotes label]"#,        // Missing quotes
            r#"class Animal["Unclosed label]"#,        // Missing closing quote
            r#"class Animal[Unopened label"]"#,        // Missing opening quote
            r#"class Animal[]"#,                       // Empty brackets, not quotes
            r#"class Animal"#,                         // Missing label section entirely
        ];
        for case in invalid_labels {
            let parsed = ExplicitClassParser::parse(Rule::test_class_labels, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_err());
        }
    }

    #[test]
    fn test_valid_class_properties() {
        let valid_cases = vec![
            r#"BankAccount : +String owner"#,            // Public property
            r#"BankAccount : -BigDecimal balance"#,      // Private property
            r#"BankAccount : #Date created"#,            // Protected property
            r#"BankAccount : ~String status"#,           // Package/internal property
            r#"BankAccount : +int id"#,                  // Simple type
            r#"BankAccount : +f64 amount"#,              // Rust type for diversity
            r#"BankAccount : +isActive"#,                // Property without explicit type
        ];
        for case in valid_cases {
            let parsed = ExplicitClassParser::parse(Rule::test_class_property, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_ok());
        }
    }

    #[test]
    fn test_valid_class_method() {
        let valid_cases = vec![
            r#"BankAccount : +String owner(testo testo)"#,            // Public property
        ];
        for case in valid_cases {
            let parsed = ExplicitClassParser::parse(Rule::test_class_method, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_ok());
        }
    }

    #[test]
    fn test_valid_comment() {
        let valid_cases = vec![
            r#"%% Moin das ist ein comment"#,            // Public property
        ];
        for case in valid_cases {
            let parsed = ExplicitClassParser::parse(Rule::test_comment, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_ok());
        }
    }

    #[test]
    fn test_valid_note() {
        let valid_cases = vec![
            r#"note "This is a general note""#,
            r#"note for Time "This was made because Date objects can't store time""#,
        ];
        for case in valid_cases {
            let parsed = ExplicitClassParser::parse(Rule::test_note, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_ok());
        }
    }

    #[test]
    fn test_valid_direction() {
        let valid_cases = vec![
            r#"direction TB"#,
            r#"direction TD"#,
            r#"direction BT"#,
            r#"direction RL"#,
            r#"direction LR"#,
        ];
        for case in valid_cases {
            let parsed = ExplicitClassParser::parse(Rule::test_direction, case);
            println!("{:#?}", parsed);
            assert!(parsed.is_ok());
        }
    }
}
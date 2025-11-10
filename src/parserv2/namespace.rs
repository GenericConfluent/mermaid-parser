use super::{IResult, Stmt};
use crate::types::{Direction, Namespace, Note};

pub fn namespace_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    todo!()
}

pub fn namespace_identifier<'source>(s: &'source str) -> IResult<&'source str, &'source str> {
    todo!()
}

pub fn namespace_name<'source>(s: &'source str) -> IResult<&'source str, &'source str> {
    todo!()
}

pub fn stmt_note<'source>(s: &'source str) -> IResult<&'source str, Note<'source>> {
    todo!()
}

pub fn stmt_direction(s: &str) -> IResult<&str, Direction> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_name() {
        // Test simple namespace name
        let (rem, name) = namespace_name("Animals").expect("Failed to parse simple name");
        assert!(rem.is_empty());
        assert_eq!(name, "Animals");

        // Test namespace name with underscores
        let (rem, name) = namespace_name("My_Namespace").expect("Failed to parse name with underscore");
        assert!(rem.is_empty());
        assert_eq!(name, "My_Namespace");

        // Test namespace name with numbers
        let (rem, name) = namespace_name("Namespace123").expect("Failed to parse name with numbers");
        assert!(rem.is_empty());
        assert_eq!(name, "Namespace123");

        // Test with whitespace
        let (rem, name) = namespace_name("  MyNamespace  ").expect("Failed to parse with whitespace");
        assert!(rem.trim().is_empty());
        assert_eq!(name, "MyNamespace");
    }

    #[test]
    fn test_namespace_identifier() {
        // Test namespace keyword followed by name
        let (rem, name) = namespace_identifier("namespace Animals")
            .expect("Failed to parse namespace identifier");
        assert!(rem.trim().is_empty());
        assert_eq!(name, "Animals");

        // Test with extra whitespace
        let (rem, name) = namespace_identifier("namespace   MyNamespace  ")
            .expect("Failed to parse with extra whitespace");
        assert!(rem.trim().is_empty());
        assert_eq!(name, "MyNamespace");

        // Test with newline after
        let (rem, name) = namespace_identifier("namespace Vehicles\n")
            .expect("Failed to parse with newline");
        assert_eq!(rem.trim(), "");
        assert_eq!(name, "Vehicles");
    }

    #[test]
    fn test_namespace_stmt_simple() {
        let input = r#"namespace Animals {
    class Dog
    class Cat
}"#;

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse simple namespace: {:?}", result.unwrap_err());

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.is_empty());
        assert_eq!(ns.name, "Animals");
        assert_eq!(ns.classes.len(), 2);
        assert!(ns.classes.contains_key("Dog"));
        assert!(ns.classes.contains_key("Cat"));
    }

    #[test]
    fn test_namespace_stmt_with_members() {
        let input = r#"namespace Vehicles {
    class Car
    Car : +speed: int
    Car : +drive(distance: int) void

    class Bike
    Bike : -gears: int
}"#;

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse namespace with members: {:?}", result.unwrap_err());

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.is_empty());
        assert_eq!(ns.name, "Vehicles");
        assert_eq!(ns.classes.len(), 2);

        let car = ns.classes.get("Car").expect("Car class should exist");
        assert_eq!(car.members.len(), 2);

        let bike = ns.classes.get("Bike").expect("Bike class should exist");
        assert_eq!(bike.members.len(), 1);
    }

    #[test]
    fn test_namespace_stmt_with_newline_after_brace() {
        let input = r#"namespace Test {

    class A
    class B
}"#;

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse with newline after opening brace");

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.is_empty());
        assert_eq!(ns.name, "Test");
        assert_eq!(ns.classes.len(), 2);
    }

    #[test]
    fn test_namespace_stmt_empty() {
        let input = "namespace Empty {\n}";

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse empty namespace");

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.is_empty());
        assert_eq!(ns.name, "Empty");
        assert_eq!(ns.classes.len(), 0);
    }

    #[test]
    fn test_namespace_stmt_with_comments() {
        let input = r#"namespace Test {
    class A
    %% This is a comment
    class B
    %% Another comment
}"#;

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse namespace with comments");

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.is_empty());
        assert_eq!(ns.name, "Test");
        assert_eq!(ns.classes.len(), 2);
    }

    #[test]
    fn test_namespace_stmt_complex() {
        let input = r#"namespace MyNamespace {
    class Animal
    Animal : -int age
    Animal : +name: String
    Animal : +move(distance: int) void

    class Vehicle
    Vehicle : +speed: int

    %% Comment about relationship
    class Person
    Person : +firstName: String
    Person : +lastName: String
}"#;

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse complex namespace");

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.is_empty());
        assert_eq!(ns.name, "MyNamespace");
        assert_eq!(ns.classes.len(), 3);

        let animal = ns.classes.get("Animal").expect("Animal should exist");
        assert_eq!(animal.members.len(), 3);

        let vehicle = ns.classes.get("Vehicle").expect("Vehicle should exist");
        assert_eq!(vehicle.members.len(), 1);

        let person = ns.classes.get("Person").expect("Person should exist");
        assert_eq!(person.members.len(), 2);
    }

    #[test]
    fn test_namespace_stmt_with_trailing_content() {
        let input = r#"namespace First {
    class A
}

class Outside"#;

        let result = namespace_stmt(input);
        assert!(result.is_ok(), "Failed to parse namespace with trailing content");

        let (rem, Stmt::Namespace(ns)) = result.unwrap() else {
            panic!("Expected Namespace statement");
        };

        assert!(rem.contains("class Outside"));
        assert_eq!(ns.name, "First");
        assert_eq!(ns.classes.len(), 1);
    }

    #[test]
    fn test_stmt_note_general() {
        // Test general note (not attached to a class)
        let (rem, note) = stmt_note("note \"This is a general note\"")
            .expect("Failed to parse general note");
        assert!(rem.is_empty());
        assert_eq!(note.text, "This is a general note");
        assert_eq!(note.class_name, None);
    }

    #[test]
    fn test_stmt_note_for_class() {
        // Test note attached to a specific class
        let (rem, note) = stmt_note("note for Vehicle \"Vehicles are fast\"")
            .expect("Failed to parse note for class");
        assert!(rem.is_empty());
        assert_eq!(note.text, "Vehicles are fast");
        assert_eq!(note.class_name, Some("Vehicle".into()));
    }

    #[test]
    fn test_stmt_note_multiline() {
        // Test note with escaped newlines or longer text
        let (rem, note) = stmt_note(r#"note "This is a longer note with some details""#)
            .expect("Failed to parse longer note");
        assert!(rem.is_empty());
        assert_eq!(note.text, "This is a longer note with some details");
    }

    #[test]
    fn test_stmt_note_with_special_chars() {
        // Test note with special characters
        let (rem, note) = stmt_note(r#"note "Note with symbols: !@#$%""#)
            .expect("Failed to parse note with special chars");
        assert!(rem.is_empty());
        assert_eq!(note.text, "Note with symbols: !@#$%");
    }

    #[test]
    fn test_stmt_direction() {
        // Test all direction values
        let (rem, dir) = stmt_direction("direction TB").expect("Failed to parse TB direction");
        assert!(rem.is_empty());
        assert_eq!(dir, Direction::TopBottom);

        let (rem, dir) = stmt_direction("direction BT").expect("Failed to parse BT direction");
        assert!(rem.is_empty());
        assert_eq!(dir, Direction::BottomTop);

        let (rem, dir) = stmt_direction("direction LR").expect("Failed to parse LR direction");
        assert!(rem.is_empty());
        assert_eq!(dir, Direction::LeftRight);

        let (rem, dir) = stmt_direction("direction RL").expect("Failed to parse RL direction");
        assert!(rem.is_empty());
        assert_eq!(dir, Direction::RightLeft);
    }

    #[test]
    fn test_stmt_direction_with_whitespace() {
        let (rem, dir) = stmt_direction("  direction   LR  ")
            .expect("Failed to parse direction with whitespace");
        assert!(rem.trim().is_empty());
        assert_eq!(dir, Direction::LeftRight);
    }
}

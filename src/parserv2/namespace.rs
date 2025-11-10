use std::borrow::Cow;
use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace0, space0, space1},
    combinator::{opt, recognize},
    sequence::{delimited, pair, preceded},
    Parser,
};

use super::{class, IResult, MermaidParseError, Stmt};
use crate::types::{Class, Direction, Member, Namespace, Note};

pub fn namespace_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    let (s, _) = multispace0.parse(s)?;

    // Parse "namespace Name"
    let (s, name) = namespace_identifier(s)?;

    // Parse opening brace
    let (s, _) = multispace0.parse(s)?;
    let (s, _) = char('{').parse(s)?;
    let (s, _) = multispace0.parse(s)?;

    // Parse class declarations and member statements within the namespace
    let mut classes: HashMap<Cow<'source, str>, Class<'source>> = HashMap::new();
    let mut s = s;

    loop {
        // Skip whitespace
        let (s_new, _) = multispace0.parse(s)?;
        s = s_new;

        // Check for closing brace
        if let Ok((s_new, _)) = char::<_, nom::error::Error<_>>('}').parse(s) {
            let (s_new, _) = multispace0.parse(s_new)?;
            s = s_new;
            break;
        }

        // Check for comment line (starts with %%)
        if let Ok((s_new, _)) = tag::<_, _, nom::error::Error<_>>("%%").parse(s) {
            // Skip the rest of the line
            let (s_new, _) = take_while(|c| c != '\n' && c != '\r').parse(s_new)?;
            s = s_new;
            continue;
        }

        // Try to parse "class ClassName" declaration
        if let Ok((s_new, _)) = tag::<_, _, nom::error::Error<_>>("class").parse(s) {
            let (s_new, _) = space1.parse(s_new)?;
            let (s_new, class_name) = class::class_name(s_new)?;

            // Insert empty class if it doesn't exist
            classes.entry(Cow::Borrowed(class_name)).or_insert_with(|| Class {
                name: Cow::Borrowed(class_name),
                annotations: Vec::new(),
                members: Vec::new(),
            });

            s = s_new;
            continue;
        }

        // Try to parse "ClassName : member" statement
        if let Ok((s_new, class_name)) = class::class_name(s) {
            let (s_new, _) = space0.parse(s_new)?;
            if let Ok((s_new2, _)) = char::<_, MermaidParseError>(':').parse(s_new) {
                // Parse the member
                let (s_new3, _) = space0.parse(s_new2)?;
                if let Ok((s_new4, member)) = class::class_member_stmt(s_new3) {
                    // Add member to the class
                    if let Some(class) = classes.get_mut(&Cow::Borrowed(class_name)) {
                        class.members.push(member);
                    }
                    s = s_new4;
                    continue;
                }
            }
        }

        // If we can't parse anything, skip to the next line
        if let Ok((s_new, _)) =
            take_while::<_, _, nom::error::Error<_>>(|c| c != '\n' && c != '\r').parse(s)
        {
            s = s_new;
        } else {
            break;
        }
    }

    Ok((
        s,
        Stmt::Namespace(Namespace {
            name: Cow::Borrowed(name),
            classes,
            children: HashMap::new(),
        }),
    ))
}

pub fn namespace_identifier<'source>(s: &'source str) -> IResult<&'source str, &'source str> {
    preceded((multispace0, tag("namespace"), space1), namespace_name).parse(s)
}

pub fn namespace_name<'source>(s: &'source str) -> IResult<&'source str, &'source str> {
    let (s, _) = multispace0.parse(s)?;

    // Parse identifier: alphanumeric, underscore, dash
    let (s, name) = recognize(pair(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    ))
    .parse(s)?;

    let (s, _) = multispace0.parse(s)?;

    Ok((s, name))
}

pub fn stmt_note<'source>(s: &'source str) -> IResult<&'source str, Note<'source>> {
    let (s, _) = multispace0.parse(s)?;

    // Try to parse "note for ClassName "text""
    if let Ok((s, _)) = tag::<_, _, nom::error::Error<_>>("note").parse(s) {
        let (s, _) = space1.parse(s)?;

        // Check if it's "for ClassName"
        if let Ok((s, _)) = tag::<_, _, nom::error::Error<_>>("for").parse(s) {
            let (s, _) = space1.parse(s)?;

            // Parse class name (can use class_name parser)
            let (s, class_name) = class::class_name(s)?;
            let (s, _) = space0.parse(s)?;

            // Parse the note text in quotes
            let (s, text) = delimited(char('"'), take_while(|c| c != '"'), char('"')).parse(s)?;

            return Ok((
                s,
                Note {
                    text: Cow::Borrowed(text),
                    target_class: Some(Cow::Borrowed(class_name)),
                },
            ));
        }

        // Otherwise it's a general note: "note "text""
        let (s, text) = delimited(char('"'), take_while(|c| c != '"'), char('"')).parse(s)?;

        return Ok((
            s,
            Note {
                text: Cow::Borrowed(text),
                target_class: None,
            },
        ));
    }

    Err(nom::Err::Error(MermaidParseError::ExpectedStmt))
}

pub fn stmt_direction(s: &str) -> IResult<&str, Direction> {
    let (s, _) = multispace0.parse(s)?;
    let (s, _) = tag("direction").parse(s)?;
    let (s, _) = space1.parse(s)?;

    let (s, dir_str) = alt((tag("TB"), tag("TD"), tag("BT"), tag("LR"), tag("RL"))).parse(s)?;

    let direction = match dir_str {
        "TB" | "TD" => Direction::TopBottom,
        "BT" => Direction::BottomTop,
        "LR" => Direction::LeftRight,
        "RL" => Direction::RightLeft,
        _ => unreachable!(),
    };

    let (s, _) = multispace0.parse(s)?;

    Ok((s, direction))
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
        assert_eq!(note.target_class, None);
    }

    #[test]
    fn test_stmt_note_for_class() {
        // Test note attached to a specific class
        let (rem, note) = stmt_note("note for Vehicle \"Vehicles are fast\"")
            .expect("Failed to parse note for class");
        assert!(rem.is_empty());
        assert_eq!(note.text, "Vehicles are fast");
        assert_eq!(note.target_class, Some("Vehicle".into()));
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

use std::{borrow::Cow, collections::HashMap};

use nom::{
    self, PResult, Parser,
    branch::alt,
    bytes::complete::*,
    character::{
        complete::{char, line_ending, multispace0, space0},
        none_of,
    },
    combinator::opt,
    error::ParseError,
    sequence::delimited,
};

use crate::types::{self, Class, Diagram, Direction, Namespace, Note, Relation};

pub mod class;
pub mod frontmatter;
pub mod namespace;
pub mod relation;

#[derive(thiserror::Error, Debug, derive_more::From)]
pub enum MermaidParseError {
    #[error("{0:?}")]
    Nom(nom::error::ErrorKind),
    #[error("{0}")]
    SerdeYml(serde_yml::Error),
    #[error("")]
    ExpectedClassDiagram,
    #[error("")]
    ExpectedStmt,
}

impl<I> ParseError<I> for MermaidParseError {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        MermaidParseError::Nom(kind)
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

type IResult<I, O> = nom::IResult<I, O, MermaidParseError>;

#[derive(Debug)]
pub enum Stmt<'source> {
    Class(Class<'source>),
    Namespace(Namespace<'source>),
    Relation(Relation<'source>),
    Note(Note<'source>),
    Direction(Direction),
}

/// Parse mermaid line by line, keeping lines we failed to parse so they can be copied to the
/// output. This parser has three contexts: - Frontmatter - Namespace - Class We start out in
/// Namespace (DEFAULT_NAMESPACE). From this context we can enter into a nested namespace, a class,
/// or the frontmatter context. In the class context we aren't able to enter any other contexts. In
/// the frontmatter context we aren't able to enter any other contexts. In a nested namespace
/// context we can only enter the class context.
///
/// This parser was maded referencing version 11.12.0 of the Mermaid CLI. If there is a frontmatter
pub fn parse_mermaid(source: &str) -> IResult<(), Diagram> {
    // First line MUST be --- unindented if we have a frontmatter
    let (mut document, yaml) = frontmatter::frontmatter(source)?;

    // Then we can have comments until a diagram definition
    while let Ok((rem, _)) = ws(comment).parse(document) {
        document = rem;
    }

    let Ok((mut body, _)) = class_diagram(document) else {
        return Err(nom::Err::Failure(MermaidParseError::ExpectedClassDiagram));
    };

    // Then we can parse the body of the diagram
    let mut namespaces: HashMap<Cow<str>, Namespace> = HashMap::new();
    // Initialize the default namespace
    namespaces.insert(
        Cow::Borrowed(types::DEFAULT_NAMESPACE),
        Namespace {
            name: Cow::Borrowed(types::DEFAULT_NAMESPACE),
            classes: HashMap::new(),
            children: HashMap::new(),
        },
    );
    let mut relations = Vec::new();
    let mut notes = Vec::new();
    let mut direction = None;

    while !body.is_empty() {
        // Skip whitespace
        match multispace0::<_, nom::error::Error<_>>(body) {
            Ok((rem, _)) => body = rem,
            Err(_) => break,
        }

        if body.is_empty() {
            break;
        }

        // Try to parse "ClassName : member" statement first
        if let Ok((s_new, class_name)) = class::class_name(body) {
            let s_new2_result = space0::<_, nom::error::Error<_>>(s_new);
            if let Ok((s_new2, _)) = s_new2_result {
                if let Ok((s_new3, _)) = char::<_, nom::error::Error<_>>(':')(s_new2) {
                    let (s_new4, _) =
                        space0::<_, nom::error::Error<_>>(s_new3).unwrap_or((s_new3, ""));
                    if let Ok((s_new5, member)) = class::class_member_stmt(s_new4) {
                        // Add member to the class in default namespace
                        if let Some(class) = namespaces
                            .get_mut(types::DEFAULT_NAMESPACE)
                            .and_then(|ns| ns.classes.get_mut(&Cow::Borrowed(class_name)))
                        {
                            class.members.push(member);
                        }
                        body = s_new5;
                        continue;
                    }
                }
            }
        }

        // NOTE: For this combinator to implement parse we actually need the same output type on
        // all out stmts. Which is why the enum exists.
        let result = alt((
            class::class_stmt,
            namespace::namespace_stmt,
            relation::relation_stmt,
            note_stmt,
            direction_stmt,
        ))
        .parse_complete(body);

        match result.map(|(rem, stmt)| {
            body = rem;
            stmt
        }) {
            Err(_why) => {
                return Err(nom::Err::Failure(MermaidParseError::ExpectedStmt));
            }
            Ok(Stmt::Class(class)) => {
                namespaces
                    .get_mut(types::DEFAULT_NAMESPACE)
                    .expect("This should exist")
                    .classes
                    .insert(class.name.clone(), class);
            }
            Ok(Stmt::Namespace(ns)) => {
                namespaces.insert(ns.name.clone(), ns);
            }
            Ok(Stmt::Relation(rl)) => relations.push(rl),
            Ok(Stmt::Note(note)) => notes.push(note),
            Ok(Stmt::Direction(dir)) => direction = Some(dir),
        }
    }

    Ok(((), Diagram {
        namespaces,
        relations,
        notes,
        direction,
        yaml,
    }))
}

fn delete_match<I, O>(val: (I, O)) -> (I, ()) {
    (val.0, ())
}

fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, opt(multispace0))
}

pub fn class_diagram(s: &str) -> IResult<&str, ()> {
    ws(alt((tag("classDiagram-v2"), tag("classDiagram"))))
        .parse_complete(s)
        .map(delete_match)
}

// Original parsing for these are done with the following two regex:
// - \%\%[^\n]*(\r?\n)*
// - \%\%(?!\{)*[^\n]*(\r?\n?)+
pub fn comment(s: &str) -> IResult<&str, ()> {
    (tag("%%"), opt(is_not("\r\n")), opt(line_ending))
        .parse(s)
        .map(delete_match)
}

pub fn note_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    let (s, note) = namespace::stmt_note(s)?;
    Ok((s, Stmt::Note(note)))
}

pub fn direction_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    let (s, direction) = namespace::stmt_direction(s)?;
    Ok((s, Stmt::Direction(direction)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        // Invalid comment
        let result = comment("% This is an invalid comment");
        assert!(result.is_err());

        // EOF
        let result = comment("%% This is a valid comment");
        assert!(result.is_ok());
        let (remainder, _) = result.unwrap();
        assert_eq!(
            remainder, "",
            "Comment ended at EOF. There should be nothing left"
        );

        // Windows Newline
        let result = comment("%% This is a comment on windows\r\nclassDiagram");
        assert!(result.is_ok());
        let (remainder, _) = result.unwrap();
        assert_eq!(remainder, "classDiagram", "We should strip the endline.");

        // Linux Newline
        let result = comment("%% This is a comment on windows\nclassDiagram");
        assert!(result.is_ok());
        let (remainder, _) = result.unwrap();
        assert_eq!(remainder, "classDiagram", "We should strip the endline.");
    }

    #[test]
    fn test_direction_stmt() {
        // Test all direction values
        let (rem, Stmt::Direction(dir)) =
            direction_stmt("direction TB").expect("Failed to parse TB direction")
        else {
            panic!("Expected Direction statement");
        };
        assert!(rem.is_empty());
        assert_eq!(dir, types::Direction::TopBottom);

        let (rem, Stmt::Direction(dir)) =
            direction_stmt("direction BT").expect("Failed to parse BT direction")
        else {
            panic!("Expected Direction statement");
        };
        assert!(rem.is_empty());
        assert_eq!(dir, types::Direction::BottomTop);

        let (rem, Stmt::Direction(dir)) =
            direction_stmt("direction LR").expect("Failed to parse LR direction")
        else {
            panic!("Expected Direction statement");
        };
        assert!(rem.is_empty());
        assert_eq!(dir, types::Direction::LeftRight);

        let (rem, Stmt::Direction(dir)) =
            direction_stmt("direction RL").expect("Failed to parse RL direction")
        else {
            panic!("Expected Direction statement");
        };
        assert!(rem.is_empty());
        assert_eq!(dir, types::Direction::RightLeft);

        // Test with whitespace
        let (rem, Stmt::Direction(dir)) = direction_stmt("  direction   LR  ")
            .expect("Failed to parse direction with whitespace")
        else {
            panic!("Expected Direction statement");
        };
        assert!(rem.trim().is_empty());
        assert_eq!(dir, types::Direction::LeftRight);
    }

    #[test]
    fn test_note_stmt() {
        // Test general note (not attached to a class)
        let (rem, Stmt::Note(note)) =
            note_stmt("note \"This is a general note\"").expect("Failed to parse general note")
        else {
            panic!("Expected Note statement");
        };
        assert!(rem.is_empty());
        assert_eq!(note.text, "This is a general note");
        assert_eq!(note.target_class, None);

        // Test note attached to a specific class
        let (rem, Stmt::Note(note)) = note_stmt("note for Vehicle \"Vehicles are fast\"")
            .expect("Failed to parse note for class")
        else {
            panic!("Expected Note statement");
        };
        assert!(rem.is_empty());
        assert_eq!(note.text, "Vehicles are fast");
        assert_eq!(note.target_class, Some("Vehicle".into()));

        // Test note with longer text
        let (rem, Stmt::Note(note)) =
            note_stmt(r#"note "This is a longer note with some details""#)
                .expect("Failed to parse longer note")
        else {
            panic!("Expected Note statement");
        };
        assert!(rem.is_empty());
        assert_eq!(note.text, "This is a longer note with some details");

        // Test note with special characters
        let (rem, Stmt::Note(note)) = note_stmt(r#"note "Note with symbols: !@#$%""#)
            .expect("Failed to parse note with special chars")
        else {
            panic!("Expected Note statement");
        };
        assert!(rem.is_empty());
        assert_eq!(note.text, "Note with symbols: !@#$%");
    }
}

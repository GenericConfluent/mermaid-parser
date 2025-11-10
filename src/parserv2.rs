use std::collections::HashMap;

use nom::{
    self, PResult, Parser,
    branch::alt,
    bytes::complete::*,
    character::{
        complete::{line_ending, multispace0},
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

pub enum Stmt {
    Class(Class),
    Namespace(Namespace),
    Relation(Relation),
    Note(Note),
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
pub fn parse_mermaid(text: &str) -> Result<Diagram, MermaidParseError> {
    // First line MUST be --- unindented if we have a frontmatter
    let (mut document, yaml) = if let Ok((rem, yaml)) = frontmatter::frontmatter(text) {
        (rem, Some(yaml))
    } else {
        (text, None)
    };

    // Then we can have comments until a diagram definition
    while let Ok((rem, _)) = ws(comment).parse(text) {
        document = rem;
    }

    let Ok((mut body, _)) = class_diagram(document) else {
        return Err(MermaidParseError::ExpectedClassDiagram);
    };

    // Then we can parse the body of the diagram
    let mut namespaces: HashMap<String, Namespace> = HashMap::new();
    let mut relations = Vec::new();
    let mut notes = Vec::new();
    let mut direction = None;

    while !body.is_empty() {
        // NOTE: For this combinator to implement parse we actually need the same output type on
        // all out stmts. Which is why the enum exists.
        let result = alt((
            class::class_stmt,
            namespace::namespace_stmt,
            relation_stmt,
            note_stmt,
            direction_stmt,
        ))
        .parse_complete(body);

        match result.map(|(rem, stmt)| {
            body = rem;
            stmt
        }) {
            Err(_why) => {
                return Err(MermaidParseError::ExpectedStmt);
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

    Ok(Diagram {
        namespaces,
        relations,
        notes,
        direction,
        yaml,
    })
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

pub fn relation_stmt(s: &str) -> IResult<&str, Stmt> {
    todo!()
}

pub fn note_stmt(s: &str) -> IResult<&str, Stmt> {
    todo!()
}

pub fn direction_stmt(s: &str) -> IResult<&str, Stmt> {
    todo!()
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
}

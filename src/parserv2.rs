use std::collections::HashMap;

use nom::{
    self, PResult, Parser,
    branch::alt,
    bytes::complete::*,
    character::{complete::line_ending, none_of},
    combinator::opt,
    error::ParseError,
};

use crate::types::{Diagram, Direction, Note};

pub mod class;
pub mod frontmatter;
pub mod namespace;

#[derive(thiserror::Error, Debug, derive_more::From)]
pub enum MermaidParseError {
    #[error("{0:?}")]
    Nom(nom::error::ErrorKind),
    #[error("{0}")]
    SerdeYml(serde_yml::Error),
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
    let (text, yaml) = if let Ok((rem, yaml)) = frontmatter::frontmatter(text) {
        (rem, Some(yaml))
    } else {
        (text, None)
    };

    // Then we can have comments until a diagram definition

    // Then we can parse the body of the diagram
    let mut namespaces = HashMap::new();
    let mut relations = Vec::new();
    let mut notes = Vec::new();
    let mut direction = None;
    alt((class::class_stmt, namespace::namespace_stmt));

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

// Original parsing for these are done with the following two regex:
// - \%\%[^\n]*(\r?\n)*
// - \%\%(?!\{)*[^\n]*(\r?\n?)+
pub fn stmt_comment(s: &str) -> IResult<&str, ()> {
    (tag("%%"), opt(is_not("\r\n")), opt(line_ending))
        .parse(s)
        .map(delete_match)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stmt_comment() {
        // Invalid comment
        let result = stmt_comment("% This is an invalid comment");
        assert!(result.is_err());

        // EOF
        let result = stmt_comment("%% This is a valid comment");
        assert!(result.is_ok());
        let (remainder, _) = result.unwrap();
        assert_eq!(
            remainder, "",
            "Comment ended at EOF. There should be nothing left"
        );

        // Windows Newline
        let result = stmt_comment("%% This is a comment on windows\r\nclassDiagram");
        assert!(result.is_ok());
        let (remainder, _) = result.unwrap();
        assert_eq!(remainder, "classDiagram", "We should strip the endline.");

        // Linux Newline
        let result = stmt_comment("%% This is a comment on windows\nclassDiagram");
        assert!(result.is_ok());
        let (remainder, _) = result.unwrap();
        assert_eq!(remainder, "classDiagram", "We should strip the endline.");
    }
}

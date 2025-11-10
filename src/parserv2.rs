use nom::{
    self, IResult, PResult, Parser,
    branch::alt,
    bytes::complete::*,
    character::{complete::line_ending, none_of},
    combinator::opt,
};

use crate::types::{Diagram, Direction, Note};

pub mod class;
pub mod frontmatter;
pub mod namespace;

/// Parse mermaid line by line, keeping lines we failed to parse so they can be copied to the
/// output. This parser has three contexts: - Frontmatter - Namespace - Class We start out in
/// Namespace (DEFAULT_NAMESPACE). From this context we can enter into a nested namespace, a class,
/// or the frontmatter context. In the class context we aren't able to enter any other contexts. In
/// the frontmatter context we aren't able to enter any other contexts. In a nested namespace
/// context we can only enter the class context.
///
/// This parser was maded referencing version 11.12.0 of the Mermaid CLI. If there is a frontmatter
/// the first line of the file MUST be "---" unindented. We cannot put comments before it. When the
/// frontmatter ends we can have either comments or a declaration of the diagram type.
pub fn parse_mermaid(mut text: &str) -> IResult<Diagram, &str> {
    // First line MUST be --- unindented if we have a frontmatter
    text = if let Ok((rem, yaml)) = frontmatter::frontmatter(text) {
        // TODO: Insert YAML
        rem
    } else {
        text
    };

    // Then we can have comments until a diagram definition

    todo!()
}

pub fn delete_match<I, O>(val: (I, O)) -> (I, ()) {
    (val.0, ())
}

// Orignal parsing for these are done with the following two regex:
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

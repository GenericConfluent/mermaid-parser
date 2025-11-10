use super::{IResult, MermaidParseError};
use nom::{
    Err, Parser,
    bytes::complete::{tag, take_until},
    character::complete::line_ending,
    combinator::opt,
    sequence::delimited,
};

/// # Parse the Yaml frontmatter
///
/// The first line of the file MUST be "---" unindented. We cannot put comments before it. When the
/// frontmatter ends we can have either comments or a declaration of the diagram type.
pub fn frontmatter(s: &str) -> IResult<&str, serde_yml::Value> {
    // We can skip consuming the first line ending since `serde_yml` can handle it.
    // Still need to consume the last though.
    let (rem, yaml) = delimited(
        tag("---"),
        take_until("---"),
        (tag("---"), opt(line_ending)),
    )
    .parse(s)?;

    let (_, value) = frontmatter_context(yaml)?;

    Ok((rem, value))
}

/// Parse Yaml with `serde_yml`. BE AWARE: this function needs a complete
/// valid yaml string as input.
pub fn frontmatter_context(yaml: &str) -> IResult<(), serde_yml::Value> {
    Ok(serde_yml::from_str(yaml)
        .map_err(MermaidParseError::SerdeYml)
        .map_err(Err::Failure)?)
}

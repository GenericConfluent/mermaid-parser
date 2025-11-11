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
///
/// Any document beginning with --- will be assumed to have frontmatter. If it doesn't then we say
/// it has no frontmatter, if it has this and we fail to parse that is considered a failure to parse
/// the frontmatter.
pub fn frontmatter(s: &str) -> IResult<&str, Option<serde_yml::Value>> {
    // Detection to distinguish between having no frontmatter and a failure to
    // parse it.
    if !s.starts_with("---") {
        return Ok((s, None));
    }

    // We can skip consuming the first line ending since `serde_yml` can handle it.
    // Still need to consume the last though.
    let (rem, yaml) = delimited(
        tag("---"),
        take_until("---"),
        (tag("---"), opt(line_ending)),
    )
    .parse(s)?;

    Ok((rem, Some(frontmatter_context(yaml)?)))
}

/// Parse Yaml with `serde_yml`. BE AWARE: this function needs a complete
/// valid yaml string as input.
pub fn frontmatter_context(
    yaml: &str,
) -> Result<serde_yml::Value, nom::Err<super::MermaidParseError>> {
    Ok(serde_yml::from_str::<serde_yml::Value>(yaml)
        .map_err(MermaidParseError::SerdeYml)
        .map_err(Err::Failure)?)
}

use nom::{IResult, Parser, bytes::tag, character::complete::line_ending, combinator::opt};

pub fn frontmatter(s: &str) -> IResult<&str, serde_yml::Value> {
    let val = (
        tag("---"),
        line_ending,
        frontmatter_context,
        tag("---"),
        opt(line_ending),
    )
        .parse(s)?;

    Ok((val.0, val.1.2))
}

pub fn frontmatter_context(s: &str) -> IResult<&str, serde_yml::Value> {
    todo!()
}

use nom::{
    Parser,
    bytes::complete::tag,
    character::complete::{multispace0, space1},
    combinator::opt,
    sequence::preceded,
};

use crate::{parserv2::ws, types::Class};

use super::{IResult, Stmt};

pub fn class_stmt(s: &str) -> IResult<&str, Stmt> {
    let (s, name) = preceded((multispace0, tag("class"), space1), class_name).parse_complete(s)?;

    // classStatements
    //      : classStatement                            {$$=[[$1], []]}
    //      | classStatement NEWLINE                    {$$=[[$1], []]}
    //      | classStatement NEWLINE classStatements    {$3[0].unshift($1); $$=$3}
    //      | noteStatement                             {$$=[[], [$1]]}
    //      | noteStatement NEWLINE                     {$$=[[], [$1]]}
    //      | noteStatement NEWLINE classStatements     {$3[1].unshift($1); $$=$3}
    //      ;
    let mut members = Vec::new();

    Ok((
        s,
        Stmt::Class(Class {
            name: name.to_string(),
            generic: None,
            annotations: Vec::new(),
            members,
            namespace: "".to_string(),
        }),
    ))
}

pub fn class_member_stmt(s: &str) -> IResult<&str, Class> {
    todo!()
}

// Originally this is:
// className
//     : alphaNumToken { $$=$1; }
//     | alphaNumToken DOT className { $$=$1+'.'+$3; }
//     | classLiteralName { $$=$1; }
//     | alphaNumToken className { $$=$1+$2; }
//     | alphaNumToken GENERICTYPE { $$=$1+'~'+$2+'~'; }
//     | classLiteralName GENERICTYPE { $$=$1+'~'+$2+'~'; }
//     ;
// We don't care about generic though.
pub fn class_name(s: &str) -> IResult<&str, &str> {
    todo!()
}

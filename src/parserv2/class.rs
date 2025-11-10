use std::borrow::Cow;

use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, space1},
    combinator::opt,
    sequence::{delimited, preceded},
};

use crate::{
    parserv2::ws,
    types::{Attribute, Class, Member, Method, Parameter, Visibility},
};

use super::{IResult, Stmt};

pub fn class_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    let (s, name) = preceded((multispace0, tag("class"), space1), class_name).parse_complete(s)?;

    // classStatements
    //      : classStatement                            {$$=[[$1], []]}
    //      | classStatement NEWLINE                    {$$=[[$1], []]}
    //      | classStatement NEWLINE classStatements    {$3[0].unshift($1); $$=$3}
    //      | noteStatement                             {$$=[[], [$1]]}
    //      | noteStatement NEWLINE                     {$$=[[], [$1]]}
    //      | noteStatement NEWLINE classStatements     {$3[1].unshift($1); $$=$3}
    //      ;
    //

    // members
    //     : MEMBER { $$ = [$1]; }
    //     | MEMBER members { $2.push($1);$$=$2;}
    //     ;
    //  mermaid doesn't actually care about the structure of the class members too much. But we do
    //  So we need parsing logic for them.
    let members = Vec::new();

    todo!();

    Ok((
        s,
        Stmt::Class(Class {
            name: Cow::Borrowed(name),
            annotations: Vec::new(),
            members,
        }),
    ))
}

pub fn class_member_stmt(s: &str) -> IResult<&str, Member> {
    todo!()
}

pub fn class_visibility(s: &str) -> IResult<&str, Visibility> {
    todo!()
}

pub fn class_attribute(s: &str) -> IResult<&str, Attribute> {
    todo!()
}

pub fn class_method(s: &str) -> IResult<&str, Method> {
    todo!()
}

pub fn class_method_param(s: &str) -> IResult<&str, Parameter> {
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
// NOTE: alphaNumToken  : UNICODE_TEXT | NUM | ALPHA | MINUS;
pub fn class_name(s: &str) -> IResult<&str, &str> {
    use nom::{bytes::complete::take_while, combinator::recognize, sequence::pair};

    // Skip leading whitespace
    let (s, _) = multispace0.parse(s)?;

    // Parse either backtick-escaped name or regular name
    let (s, name) = alt((
        // Backtick-escaped name (for special characters)
        delimited(
            char('`'),
            take_while1(|c: char| c != '`'),
            char('`'),
        ),
        // Regular alphanumeric name: must start with alphanumeric or underscore,
        // can continue with alphanumeric, underscore, or dash
        recognize(pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )),
    ))
    .parse(s)?;

    // Skip trailing whitespace
    let (s, _) = multispace0.parse(s)?;

    Ok((s, name))
}

#[cfg(test)]
mod tests {
    use crate::types::{Attribute, Member, Method, Parameter, TypeNotation, Visibility};

    use super::*;

    #[test]
    fn test_class_name() {
        let (rem, name) =
            class_name("Normal23Class-Name").expect("Failed to parse alpha num tokens");
        assert!(rem.is_empty());
        assert_eq!(name, "Normal23Class-Name");

        let (rem, name) = class_name("\t \t Whitespace  ").expect("Failed to parse whitespace");
        assert!(rem.is_empty());
        assert_eq!(name, "Whitespace");
    }

    #[test]
    fn test_class_stmt() {
        let class = "
    \r\n
class Dolphin {
    - int age
    +   name: String

+ void swim(distance: int)
    -  digest(Food food) void
    %% Very important comment

    sleep(time: Time, Hemisphere hemisphere) Int

    %% Beans
}
\r\n

class Next";

        eprintln!("Test class: \n{class}");
        let result = class_stmt(class);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.unwrap_err());
        let (rem, Stmt::Class(class)) = result.unwrap() else {
            panic!("Returned a non class statement");
        };
        assert_eq!(rem, "class Next");
        assert_eq!(class.name, "Dolphin", "Class names don't match");
        assert_eq!(class.members.len(), 5, "Parsed the wrong number of members");

        let age = Member::Attribute(Attribute {
            visibility: Visibility::Private,
            name: "age".into(),
            data_type: Some("int".into()),
            is_static: false,
            type_notation: TypeNotation::Prefix,
        });

        let name = Member::Attribute(Attribute {
            visibility: Visibility::Public,
            name: "name".into(),
            data_type: Some("String".into()),
            is_static: false,
            type_notation: TypeNotation::Postfix,
        });

        let swim = Member::Method(Method {
            visibility: Visibility::Public,
            name: "swim".into(),
            parameters: vec![Parameter {
                name: "distance".into(),
                data_type: Some("int".into()),
                type_notation: TypeNotation::Postfix,
            }],
            return_type: Some("void".into()),
            is_static: false,
            is_abstract: false,
            return_type_notation: TypeNotation::Prefix,
        });

        let digest = Member::Method(Method {
            visibility: Visibility::Private,
            name: "digest".into(),
            parameters: vec![Parameter {
                name: "food".into(),
                data_type: Some("Food".into()),
                type_notation: TypeNotation::Prefix,
            }],
            return_type: Some("void".into()),
            is_static: false,
            is_abstract: false,
            return_type_notation: TypeNotation::Postfix,
        });

        let sleep = Member::Method(Method {
            visibility: Visibility::Unspecified,
            name: "sleep".into(),
            parameters: vec![
                Parameter {
                    name: "time".into(),
                    data_type: Some("Time".into()),
                    type_notation: TypeNotation::Postfix,
                },
                Parameter {
                    name: "hemisphere".into(),
                    data_type: Some("Hemisphere".into()),
                    type_notation: TypeNotation::Prefix,
                },
            ],
            return_type: Some("Int".into()),
            is_static: false,
            is_abstract: false,
            return_type_notation: TypeNotation::Postfix,
        });

        let expected_members = vec![age, name, swim, digest, sleep];

        for (i, (expected, actual)) in expected_members
            .iter()
            .zip(class.members.iter())
            .enumerate()
        {
            assert_eq!(
                expected, actual,
                "Member at index {} does not match. Expected: {:?}, Got: {:?}",
                i, expected, actual
            );
        }
    }
}

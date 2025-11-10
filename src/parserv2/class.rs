use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, space1},
    combinator::opt,
    sequence::{delimited, preceded},
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
    //

    // members
    //     : MEMBER { $$ = [$1]; }
    //     | MEMBER members { $2.push($1);$$=$2;}
    //     ;
    //  mermaid doesn't actually care about the structure of the class members too much. But we do
    //  So we need parsing logic for them.
    let mut members = Vec::new();

    todo!();

    Ok((
        s,
        // NOTE: We don't want to go as far as parsing type generics, annotations, and we can't
        // store namespace.
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
    use crate::types::{Attribute, Member, Method, TypeNotation, Visibility};

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
            name: "age".to_string(),
            data_type: Some("int"),
            is_static: false,
            type_notation: TypeNotation::Prefix,
        });

        let name = Member::Attribute(Attribute {
            visibility: Visibility::Public,
            name: "name".to_string(),
            data_type: Some("String"),
            is_static: false,
            type_notation: TypeNotation::Postfix,
        });

        let swim = Member::Method(Method {
            visibility: Visibility::Public,
            name: "swim".to_string(),
            parameters: vec![],
            return_type: "void",
            is_static: false,
            is_abstract: false,
            return_type_notation: TypeNotation::Prefix,
        });

        let digest = Member::Method(Method {
            visibility: Visibility::Private,
            name: "digest".to_string(),
            parameters: vec![],
            return_type: "void",
            is_static: false,
            is_abstract: false,
            return_type_notation: TypeNotation::Postfix,
        });

        let sleep = Member::Method(Method {
            visibility: Visibility::Unspecified,
            name: "sleep".to_string(),
            parameters: vec![],
            return_type: (),
            is_static: (),
            is_abstract: (),
            return_type_notation: (),
        });

        for (name, member) in ["age", "name", "swim", "digest", "sleep"]
            .iter()
            .zip(class.members)
        {
            assert_eq!()
        }
    }
}

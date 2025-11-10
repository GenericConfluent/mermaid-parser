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
        delimited(char('`'), take_while1(|c: char| c != '`'), char('`')),
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
    fn test_class_visibility() {
        // Test public visibility
        let (rem, vis) = class_visibility("+").expect("Failed to parse public visibility");
        assert!(rem.is_empty());
        assert_eq!(vis, Visibility::Public);

        // Test private visibility
        let (rem, vis) = class_visibility("-").expect("Failed to parse private visibility");
        assert!(rem.is_empty());
        assert_eq!(vis, Visibility::Private);

        // Test protected visibility
        let (rem, vis) = class_visibility("#").expect("Failed to parse protected visibility");
        assert!(rem.is_empty());
        assert_eq!(vis, Visibility::Protected);

        // Test package/internal visibility
        let (rem, vis) = class_visibility("~").expect("Failed to parse package visibility");
        assert!(rem.is_empty());
        assert_eq!(vis, Visibility::Package);

        // Test with whitespace
        let (rem, vis) = class_visibility("  +  ").expect("Failed to parse with whitespace");
        assert_eq!(rem.trim(), "");
        assert_eq!(vis, Visibility::Public);
    }

    #[test]
    fn test_class_method_param() {
        // Test postfix notation: name: Type
        let (rem, param) =
            class_method_param("distance: int").expect("Failed to parse postfix parameter");
        assert!(rem.is_empty());
        assert_eq!(param.name, "distance");
        assert_eq!(param.data_type, Some("int".into()));
        assert_eq!(param.type_notation, TypeNotation::Postfix);

        // Test prefix notation: Type name
        let (rem, param) =
            class_method_param("Food food").expect("Failed to parse prefix parameter");
        assert!(rem.is_empty());
        assert_eq!(param.name, "food");
        assert_eq!(param.data_type, Some("Food".into()));
        assert_eq!(param.type_notation, TypeNotation::Prefix);

        // Test parameter with no type
        let (rem, param) =
            class_method_param("param").expect("Failed to parse parameter without type");
        assert!(rem.is_empty());
        assert_eq!(param.name, "param");
        assert_eq!(param.data_type, None);
        assert_eq!(param.type_notation, TypeNotation::None);

        // Test with extra whitespace
        let (rem, param) = class_method_param("  time  :  Time  ")
            .expect("Failed to parse parameter with whitespace");
        assert!(rem.trim().is_empty());
        assert_eq!(param.name, "time");
        assert_eq!(param.data_type, Some("Time".into()));
        assert_eq!(param.type_notation, TypeNotation::Postfix);
    }

    #[test]
    fn test_class_attribute() {
        // Test private attribute with prefix notation: - int age
        let (rem, attr) =
            class_attribute("- int age").expect("Failed to parse private prefix attribute");
        assert!(rem.is_empty());
        assert_eq!(attr.visibility, Visibility::Private);
        assert_eq!(attr.name, "age");
        assert_eq!(attr.data_type, Some("int".into()));
        assert_eq!(attr.is_static, false);
        assert_eq!(attr.type_notation, TypeNotation::Prefix);

        // Test public attribute with postfix notation: + name: String
        let (rem, attr) =
            class_attribute("+   name: String").expect("Failed to parse public postfix attribute");
        assert!(rem.is_empty());
        assert_eq!(attr.visibility, Visibility::Public);
        assert_eq!(attr.name, "name");
        assert_eq!(attr.data_type, Some("String".into()));
        assert_eq!(attr.is_static, false);
        assert_eq!(attr.type_notation, TypeNotation::Postfix);

        // Test static attribute: + $ count: int
        let (rem, attr) =
            class_attribute("+ $ count: int").expect("Failed to parse static attribute");
        assert!(rem.is_empty());
        assert_eq!(attr.visibility, Visibility::Public);
        assert_eq!(attr.name, "count");
        assert_eq!(attr.data_type, Some("int".into()));
        assert_eq!(attr.is_static, true);
        assert_eq!(attr.type_notation, TypeNotation::Postfix);

        // Test attribute without type: # id
        let (rem, attr) = class_attribute("# id").expect("Failed to parse attribute without type");
        assert!(rem.is_empty());
        assert_eq!(attr.visibility, Visibility::Protected);
        assert_eq!(attr.name, "id");
        assert_eq!(attr.data_type, None);
        assert_eq!(attr.is_static, false);
        assert_eq!(attr.type_notation, TypeNotation::None);

        // Test attribute without visibility (unspecified)
        let (rem, attr) =
            class_attribute("value: double").expect("Failed to parse attribute without visibility");
        assert!(rem.is_empty());
        assert_eq!(attr.visibility, Visibility::Unspecified);
        assert_eq!(attr.name, "value");
        assert_eq!(attr.data_type, Some("double".into()));
        assert_eq!(attr.type_notation, TypeNotation::Postfix);
    }

    #[test]
    fn test_class_method() {
        // Test public method with prefix return and parameter: + void swim(distance: int)
        let (rem, method) = class_method("+ void swim(distance: int)")
            .expect("Failed to parse method with prefix return");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Public);
        assert_eq!(method.name, "swim");
        assert_eq!(method.parameters.len(), 1);
        assert_eq!(method.parameters[0].name, "distance");
        assert_eq!(method.parameters[0].data_type, Some("int".into()));
        assert_eq!(method.return_type, Some("void".into()));
        assert_eq!(method.is_static, false);
        assert_eq!(method.is_abstract, false);
        assert_eq!(method.return_type_notation, TypeNotation::Prefix);

        // Test private method with postfix return: - digest(Food food) void
        let (rem, method) =
            class_method("-  digest(Food food) void").expect("Failed to parse method with postfix return");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Private);
        assert_eq!(method.name, "digest");
        assert_eq!(method.parameters.len(), 1);
        assert_eq!(method.parameters[0].name, "food");
        assert_eq!(method.parameters[0].data_type, Some("Food".into()));
        assert_eq!(method.return_type, Some("void".into()));
        assert_eq!(method.is_static, false);
        assert_eq!(method.is_abstract, false);
        assert_eq!(method.return_type_notation, TypeNotation::Postfix);

        // Test method without visibility and multiple parameters
        let (rem, method) = class_method("sleep(time: Time, Hemisphere hemisphere) Int")
            .expect("Failed to parse method with multiple parameters");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Unspecified);
        assert_eq!(method.name, "sleep");
        assert_eq!(method.parameters.len(), 2);
        assert_eq!(method.parameters[0].name, "time");
        assert_eq!(method.parameters[0].data_type, Some("Time".into()));
        assert_eq!(method.parameters[1].name, "hemisphere");
        assert_eq!(method.parameters[1].data_type, Some("Hemisphere".into()));
        assert_eq!(method.return_type, Some("Int".into()));
        assert_eq!(method.return_type_notation, TypeNotation::Postfix);

        // Test static method: + $ getInstance() Singleton
        let (rem, method) =
            class_method("+ $ getInstance() Singleton").expect("Failed to parse static method");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Public);
        assert_eq!(method.name, "getInstance");
        assert_eq!(method.parameters.len(), 0);
        assert_eq!(method.return_type, Some("Singleton".into()));
        assert_eq!(method.is_static, true);
        assert_eq!(method.is_abstract, false);

        // Test abstract method: + * calculate() void
        let (rem, method) =
            class_method("+ * calculate() void").expect("Failed to parse abstract method");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Public);
        assert_eq!(method.name, "calculate");
        assert_eq!(method.is_static, false);
        assert_eq!(method.is_abstract, true);

        // Test method without return type: # process(data)
        let (rem, method) =
            class_method("# process(data)").expect("Failed to parse method without return type");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Protected);
        assert_eq!(method.name, "process");
        assert_eq!(method.parameters.len(), 1);
        assert_eq!(method.parameters[0].name, "data");
        assert_eq!(method.return_type, None);
        assert_eq!(method.return_type_notation, TypeNotation::None);

        // Test method with no parameters: ~ getValue() int
        let (rem, method) =
            class_method("~ getValue() int").expect("Failed to parse method with no parameters");
        assert!(rem.is_empty());
        assert_eq!(method.visibility, Visibility::Package);
        assert_eq!(method.name, "getValue");
        assert_eq!(method.parameters.len(), 0);
        assert_eq!(method.return_type, Some("int".into()));
    }

    #[test]
    fn test_class_member_stmt() {
        // Test parsing an attribute member
        let (rem, member) =
            class_member_stmt("- int age").expect("Failed to parse attribute member");
        assert!(rem.is_empty());
        match member {
            Member::Attribute(attr) => {
                assert_eq!(attr.visibility, Visibility::Private);
                assert_eq!(attr.name, "age");
                assert_eq!(attr.data_type, Some("int".into()));
            }
            _ => panic!("Expected Attribute member"),
        }

        // Test parsing a method member
        let (rem, member) =
            class_member_stmt("+ void swim(distance: int)").expect("Failed to parse method member");
        assert!(rem.is_empty());
        match member {
            Member::Method(method) => {
                assert_eq!(method.visibility, Visibility::Public);
                assert_eq!(method.name, "swim");
                assert_eq!(method.parameters.len(), 1);
            }
            _ => panic!("Expected Method member"),
        }

        // Test with leading whitespace
        let (rem, member) =
            class_member_stmt("    + name: String").expect("Failed to parse member with whitespace");
        assert!(rem.trim().is_empty());
        match member {
            Member::Attribute(attr) => {
                assert_eq!(attr.name, "name");
            }
            _ => panic!("Expected Attribute member"),
        }
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

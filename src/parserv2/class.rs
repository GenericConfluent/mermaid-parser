use std::borrow::Cow;

use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, space1},
    combinator::opt,
    sequence::{delimited, preceded},
};

use crate::types::{Attribute, Class, Member, Method, Parameter, TypeNotation, Visibility};

use super::{IResult, Stmt};

pub fn class_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    use nom::{
        bytes::complete::{take_until, take_while},
        character::complete::{char, line_ending},
    };

    let (s, name) = preceded((multispace0, tag("class"), space1), class_name).parse_complete(s)?;

    let (s, _) = multispace0.parse(s)?;

    // Check if there's an opening brace - if not, this is a bare class declaration
    if let Err(_) = char::<_, nom::error::Error<_>>('{').parse(s) {
        // Bare class declaration - just return empty class
        return Ok((
            s,
            Stmt::Class(Class {
                name: Cow::Borrowed(name),
                annotation: None,
                members: Vec::new(),
            }),
        ));
    }

    // Parse opening brace
    let (s, _) = char('{').parse(s)?;
    let (s, _) = multispace0.parse(s)?;

    // Parse members, handling comments and whitespace
    let mut members = Vec::new();
    let mut s = s;

    loop {
        // Skip whitespace
        let (s_new, _) = multispace0.parse(s)?;
        s = s_new;

        // Check for closing brace
        if let Ok((s_new, _)) = char::<_, nom::error::Error<_>>('}').parse(s) {
            // Consume trailing whitespace after closing brace
            let (s_new, _) = multispace0.parse(s_new)?;
            s = s_new;
            break;
        }

        // Check for comment line (starts with %%)
        if let Ok((s_new, _)) = tag::<_, _, nom::error::Error<_>>("%%").parse(s) {
            // Skip the rest of the line
            let (s_new, _) = take_while(|c| c != '\n' && c != '\r').parse(s_new)?;
            s = s_new;
            continue;
        }

        // Try to parse a member
        match class_member_stmt(s) {
            Ok((s_new, member)) => {
                members.push(member);
                s = s_new;
            }
            Err(_) => {
                // If we can't parse a member, skip to the next line
                if let Ok((s_new, _)) = take_while::<_, _, nom::error::Error<_>>(|c| c != '\n' && c != '\r').parse(s) {
                    s = s_new;
                } else {
                    break;
                }
            }
        }
    }

    Ok((
        s,
        Stmt::Class(Class {
            name: Cow::Borrowed(name),
            annotation: None,
            members,
        }),
    ))
}

pub fn class_member_stmt<'source>(s: &'source str) -> IResult<&'source str, Member<'source>> {
    // Try to parse as a method first (methods have parentheses), then fallback to attribute
    alt((
        |s| class_method(s).map(|(rem, method)| (rem, Member::Method(method))),
        |s| class_attribute(s).map(|(rem, attr)| (rem, Member::Attribute(attr))),
    ))
    .parse(s)
}

pub fn class_visibility(s: &str) -> IResult<&str, Visibility> {
    use nom::character::complete::one_of;

    let (s, _) = multispace0.parse(s)?;

    let (s, vis_char) = one_of("+-#~").parse(s)?;

    let visibility = match vis_char {
        '+' => Visibility::Public,
        '-' => Visibility::Private,
        '#' => Visibility::Protected,
        '~' => Visibility::Package,
        _ => unreachable!(),
    };

    let (s, _) = multispace0.parse(s)?;

    Ok((s, visibility))
}

pub fn class_attribute<'source>(s: &'source str) -> IResult<&'source str, Attribute<'source>> {
    use nom::{
        bytes::complete::take_while,
        character::complete::{char, space0},
        combinator::recognize,
        sequence::pair,
    };

    let (s, _) = multispace0.parse(s)?;

    // Optional visibility
    let (s, visibility) = opt(class_visibility).parse(s)?;
    let visibility = visibility.unwrap_or(Visibility::Unspecified);

    let (s, _) = space0.parse(s)?;

    // Optional static modifier ($)
    let (s, is_static) = opt(|s| {
        let (s, _) = char('$').parse(s)?;
        let (s, _) = space0.parse(s)?;
        Ok((s, true))
    })
    .parse(s)?;
    let is_static = is_static.unwrap_or(false);

    let (s, _) = space0.parse(s)?;

    // Try to parse as postfix notation (name: Type) or prefix notation (Type name) or just name
    // First, get the first identifier
    let (s, first_token) = recognize(pair(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    ))
    .parse(s)?;

    let (s, _) = space0.parse(s)?;

    // Check if there's a colon (postfix notation)
    let (s, has_colon) = opt(char(':')).parse(s)?;

    if has_colon.is_some() {
        // Postfix notation: name: Type
        let (s, _) = space0.parse(s)?;
        let (s, type_token) = opt(recognize(pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )))
        .parse(s)?;

        Ok((
            s,
            Attribute {
                visibility,
                name: Cow::Borrowed(first_token),
                data_type: type_token.map(Cow::Borrowed),
                is_static,
                type_notation: if type_token.is_some() {
                    TypeNotation::Postfix
                } else {
                    TypeNotation::None
                },
            },
        ))
    } else {
        // Check if there's a second token (prefix notation: Type name)
        let (s, second_token) = opt(recognize(pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )))
        .parse(s)?;

        if let Some(name_token) = second_token {
            // Prefix notation: Type name
            Ok((
                s,
                Attribute {
                    visibility,
                    name: Cow::Borrowed(name_token),
                    data_type: Some(Cow::Borrowed(first_token)),
                    is_static,
                    type_notation: TypeNotation::Prefix,
                },
            ))
        } else {
            // Just a name with no type
            Ok((
                s,
                Attribute {
                    visibility,
                    name: Cow::Borrowed(first_token),
                    data_type: None,
                    is_static,
                    type_notation: TypeNotation::None,
                },
            ))
        }
    }
}

pub fn class_method<'source>(s: &'source str) -> IResult<&'source str, Method<'source>> {
    use nom::{
        bytes::complete::take_while,
        character::complete::{char, space0},
        combinator::recognize,
        multi::separated_list0,
        sequence::pair,
    };

    let (s, _) = multispace0.parse(s)?;

    // Optional visibility
    let (s, visibility) = opt(class_visibility).parse(s)?;
    let visibility = visibility.unwrap_or(Visibility::Unspecified);

    let (s, _) = space0.parse(s)?;

    // Optional static modifier ($)
    let (s, is_static) = opt(|s| {
        let (s, _) = char('$').parse(s)?;
        let (s, _) = space0.parse(s)?;
        Ok((s, true))
    })
    .parse(s)?;
    let is_static = is_static.unwrap_or(false);

    let (s, _) = space0.parse(s)?;

    // Optional abstract modifier (*)
    let (s, is_abstract) = opt(|s| {
        let (s, _) = char('*').parse(s)?;
        let (s, _) = space0.parse(s)?;
        Ok((s, true))
    })
    .parse(s)?;
    let is_abstract = is_abstract.unwrap_or(false);

    let (s, _) = space0.parse(s)?;

    // Check if there's a return type before the method name (prefix notation)
    // We need to look ahead to see if there's an identifier followed by '('
    // Let's try to parse: [Type] name(params) [ReturnType]

    // Try to get first token
    let (s, first_token) = recognize(pair(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    ))
    .parse(s)?;

    let (s, _) = space0.parse(s)?;

    // Check if next char is '(' - if so, first_token is the method name
    let (s, is_paren) = opt(char('(')).parse(s)?;

    let (s, prefix_return_type, method_name) = if is_paren.is_some() {
        // No prefix return type, first_token is method name
        (s, None, first_token)
    } else {
        // first_token might be a return type, get the next token as method name
        let (s, name_token) = recognize(pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        ))
        .parse(s)?;

        let (s, _) = space0.parse(s)?;
        let (s, _) = char('(').parse(s)?;

        (s, Some(first_token), name_token)
    };

    // Parse parameters
    let (s, _) = space0.parse(s)?;
    let (s, parameters) = separated_list0(
        (space0, char(','), space0),
        class_method_param,
    )
    .parse(s)?;

    let (s, _) = space0.parse(s)?;
    let (s, _) = char(')').parse(s)?;
    let (s, _) = space0.parse(s)?;

    // Check for postfix return type
    let (s, postfix_return_type) = opt(recognize(pair(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    )))
    .parse(s)?;

    // Determine return type and notation
    let (return_type, return_type_notation) = if let Some(prefix_type) = prefix_return_type {
        (Some(Cow::Borrowed(prefix_type)), TypeNotation::Prefix)
    } else if let Some(postfix_type) = postfix_return_type {
        (Some(Cow::Borrowed(postfix_type)), TypeNotation::Postfix)
    } else {
        (None, TypeNotation::None)
    };

    Ok((
        s,
        Method {
            visibility,
            name: Cow::Borrowed(method_name),
            parameters,
            return_type,
            is_static,
            is_abstract,
            return_type_notation,
        },
    ))
}

pub fn class_method_param<'source>(
    s: &'source str,
) -> IResult<&'source str, Parameter<'source>> {
    use nom::{
        bytes::complete::take_while,
        character::complete::{char, space0},
        combinator::recognize,
        sequence::pair,
    };

    let (s, _) = space0.parse(s)?;

    // Get first identifier
    let (s, first_token) = recognize(pair(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    ))
    .parse(s)?;

    let (s, _) = space0.parse(s)?;

    // Check if there's a colon (postfix notation: name: Type)
    let (s, has_colon) = opt(char(':')).parse(s)?;

    if has_colon.is_some() {
        // Postfix notation
        let (s, _) = space0.parse(s)?;
        let (s, type_token) = opt(recognize(pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )))
        .parse(s)?;

        Ok((
            s,
            Parameter {
                name: Cow::Borrowed(first_token),
                data_type: type_token.map(Cow::Borrowed),
                type_notation: if type_token.is_some() {
                    TypeNotation::Postfix
                } else {
                    TypeNotation::None
                },
            },
        ))
    } else {
        // Check for second token (prefix notation: Type name)
        let (s, second_token) = opt(recognize(pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )))
        .parse(s)?;

        if let Some(name_token) = second_token {
            // Prefix notation: Type name
            Ok((
                s,
                Parameter {
                    name: Cow::Borrowed(name_token),
                    data_type: Some(Cow::Borrowed(first_token)),
                    type_notation: TypeNotation::Prefix,
                },
            ))
        } else {
            // Just a name with no type
            Ok((
                s,
                Parameter {
                    name: Cow::Borrowed(first_token),
                    data_type: None,
                    type_notation: TypeNotation::None,
                },
            ))
        }
    }
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

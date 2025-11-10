use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use thiserror::Error;

use crate::types::{
    Attribute, Class, DEFAULT_NAMESPACE, Diagram, Direction, LineStyle, Member, Method, Namespace,
    Note, Parameter, Relation, RelationKind, TypeNotation, Visibility,
};

#[derive(Parser)]
#[grammar = "grammar/mermaid.pest"]
struct MermaidParser;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("pest: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("{0}")]
    Custom(String),
}

/// Minimal typed AST node per top‑level statement
enum Stmt {
    Class(Class),
    Member { target: String, member: Member },
    Relation(Relation),
    Note(Note),
    Direction(Direction),
}

// ────────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────────

pub fn parse(src: &str) -> Result<Diagram, ParseError> {
    // 0) Extract YAML frontmatter if present
    let (yaml_value, mermaid_src) = extract_yaml_frontmatter(src)?;

    // 1) let Pest build a rich tree (inc. all tokens)
    let mut outer = MermaidParser::parse(Rule::diagram, mermaid_src)?;
    let diagram_pair = outer
        .next()
        .ok_or_else(|| ParseError::Custom("diagram pair missing".into()))?;

    // 2) fold every top‑level pair into a Stmt enum – zero manual slicing
    let mut stmts = Vec::<Stmt>::new();
    for pair in diagram_pair.into_inner() {
        collect_stmt(pair, &mut stmts)?;
    }

    // 3) second pass – build the final Diagram
    let mut diagram = Diagram::default();
    diagram.yaml = yaml_value;
    for stmt in stmts {
        apply_stmt(stmt, &mut diagram);
    }
    Ok(diagram)
}

/// Extract YAML frontmatter delimited by --- at the start of the file
/// Returns (Option<yaml_value>, remaining_mermaid_source)
fn extract_yaml_frontmatter(src: &str) -> Result<(Option<serde_yml::Value>, &str), ParseError> {
    let trimmed = src.trim_start();

    // Check if the file starts with ---
    if !trimmed.starts_with("---") {
        return Ok((None, src));
    }

    // Find the closing ---
    let after_opening = &trimmed[3..];
    if let Some(closing_pos) = after_opening.find("\n---") {
        let yaml_content = &after_opening[..closing_pos];
        let remaining = &after_opening[closing_pos + 4..]; // Skip \n---

        // Parse the YAML
        let yaml_value: serde_yml::Value = serde_yml::from_str(yaml_content)
            .map_err(|e| ParseError::Custom(format!("YAML parse error: {}", e)))?;

        Ok((Some(yaml_value), remaining.trim_start()))
    } else {
        // No closing ---, treat the whole thing as mermaid
        Ok((None, src))
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// First pass: build lightweight statement enums
// ────────────────────────────────────────────────────────────────────────────────

fn collect_stmt(pair: Pair<Rule>, out: &mut Vec<Stmt>) -> Result<(), ParseError> {
    match pair.as_rule() {
        Rule::class => out.push(Stmt::Class(scan_class(pair)?)),
        Rule::member_stmt => out.push(scan_member_stmt(pair)?),
        Rule::relation_stmt => out.push(Stmt::Relation(scan_relation(pair)?)),
        Rule::note => out.push(Stmt::Note(scan_note(pair)?)),
        Rule::direction => out.push(Stmt::Direction(scan_direction(pair)?)),
        Rule::namespace_block => {
            // Parse namespace block and add namespace prefix to all classes inside
            scan_namespace_block(pair, out)?;
        }
        _ => {
            for inner in pair.into_inner() {
                collect_stmt(inner, out)?;
            }
        }
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────────
// Class
// ────────────────────────────────────────────────────────────────────────────────

fn scan_class(pair: Pair<Rule>) -> Result<Class, ParseError> {
    let mut id: Option<String> = None;
    let mut members = Vec::<Member>::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::class_identifier => id = Some(strip_backticks(inner.as_str())),
            Rule::member_stmt => {
                if let Stmt::Member { member, .. } = scan_member_stmt(inner)? {
                    members.push(member)
                }
            }
            _ => {}
        }
    }

    let fq_name = id.ok_or_else(|| ParseError::Custom("class id missing".into()))?;
    let (ns, _) = split_namespace(&fq_name);

    Ok(Class {
        name: fq_name.clone(),
        generic: None,
        annotations: Vec::new(),
        members,
        namespace: ns.to_owned(),
    })
}

// ────────────────────────────────────────────────────────────────────────────────
// Member statement
// ────────────────────────────────────────────────────────────────────────────────

fn scan_member_stmt(pair: Pair<Rule>) -> Result<Stmt, ParseError> {
    // grammar: class_identifier ':' member_decl
    let mut inner = pair.into_inner();
    let target = strip_backticks(
        inner
            .next()
            .ok_or_else(|| ParseError::Custom("member: target missing".into()))?
            .as_str()
            .trim(),
    );
    let member_decl = inner
        .next()
        .ok_or_else(|| ParseError::Custom("member: decl missing".into()))?;

    let member = build_member(member_decl)?;

    Ok(Stmt::Member { target, member })
}

fn build_member(decl: Pair<Rule>) -> Result<Member, ParseError> {
    let mut is_static = false;
    let mut is_abstract = false;
    let mut core: Option<Member> = None;

    for part in decl.into_inner() {
        match part.as_rule() {
            // classifier
            Rule::classifier => match part.as_str() {
                "$" => is_static = true,
                "*" => is_abstract = true,
                _ => {}
            },

            // attribute vs method
            Rule::class_property_decl => {
                let attribute = parse_attribute(part, is_static)?;
                core = Some(Member::Attribute(attribute));
            }
            Rule::class_method_decl => {
                let method = parse_method(part, is_static, is_abstract)?;
                core = Some(Member::Method(method));
            }
            _ => {}
        }
    }
    core.ok_or_else(|| ParseError::Custom("member core missing".into()))
}

// -----------------------------------------------------------------------------
// Attribute
// -----------------------------------------------------------------------------

fn parse_attribute(attr: Pair<Rule>, is_static: bool) -> Result<Attribute, ParseError> {
    let mut visibility = Visibility::Unspecified;
    let mut name: Option<String> = None;
    let mut ty: Option<String> = None;
    let mut type_notation = TypeNotation::None;

    for p in attr.into_inner() {
        match p.as_rule() {
            Rule::visibility => visibility = Visibility::from(p.as_str().chars().next().unwrap()),
            Rule::property_type => {
                // Parse the property_type rule to determine notation
                let mut parts = p.into_inner();
                let first = parts.next();
                let second = parts.next();

                match (first, second) {
                    (Some(f), Some(s)) if f.as_rule() == Rule::class_identifier => {
                        // Prefix: Type Name
                        ty = Some(strip_backticks(f.as_str()));
                        name = Some(s.as_str().to_owned());
                        type_notation = TypeNotation::Prefix;
                    }
                    (Some(f), Some(s)) if f.as_rule() == Rule::variable_identifier => {
                        // Postfix: Name: Type
                        name = Some(f.as_str().to_owned());
                        ty = Some(strip_backticks(s.as_str()));
                        type_notation = TypeNotation::Postfix;
                    }
                    (Some(f), None) if f.as_rule() == Rule::variable_identifier => {
                        // No type
                        name = Some(f.as_str().to_owned());
                        type_notation = TypeNotation::None;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(Attribute {
        visibility,
        name: name.ok_or_else(|| ParseError::Custom("attr name missing".into()))?,
        data_type: ty,
        is_static,
        type_notation,
    })
}

// -----------------------------------------------------------------------------
// Method
// -----------------------------------------------------------------------------

fn parse_method(
    meth: Pair<Rule>,
    is_static: bool,
    is_abstract: bool,
) -> Result<Method, ParseError> {
    let mut visibility = Visibility::Unspecified;
    let mut name: Option<String> = None;
    let mut params: Vec<Parameter> = Vec::new();
    let mut return_type: Option<String> = None;
    let mut return_type_notation = TypeNotation::None;

    for p in meth.into_inner() {
        match p.as_rule() {
            Rule::visibility => visibility = Visibility::from(p.as_str().chars().next().unwrap()),
            Rule::method_identifier => name = Some(p.as_str().to_owned()),
            Rule::method_parameter => params = parse_parameters(p)?,
            Rule::class_identifier => {
                return_type = Some(strip_backticks(p.as_str()));
                // If there's a return type, it's postfix notation (no colon in grammar now)
                return_type_notation = TypeNotation::Postfix;
            }
            _ => {}
        }
    }

    Ok(Method {
        visibility,
        name: name.ok_or_else(|| ParseError::Custom("method name missing".into()))?,
        parameters: params,
        return_type,
        is_static,
        is_abstract,
        return_type_notation,
    })
}

fn parse_parameters(list: Pair<Rule>) -> Result<Vec<Parameter>, ParseError> {
    let mut v = Vec::<Parameter>::new();
    for p in list.into_inner() {
        // method_parameter → parameter_list → many parameter
        match p.as_rule() {
            Rule::parameter_list => {
                // Descend into parameter_list
                for param in p.into_inner() {
                    if param.as_rule() == Rule::parameter {
                        v.push(parse_parameter(param)?);
                    }
                }
            }
            Rule::parameter => {
                // Direct parameter (shouldn't happen with current grammar)
                v.push(parse_parameter(p)?);
            }
            _ => {}
        }
    }
    Ok(v)
}

fn parse_parameter(p: Pair<Rule>) -> Result<Parameter, ParseError> {
    let mut ty: Option<String> = None;
    let mut name: Option<String> = None;
    let mut type_notation = TypeNotation::None;

    let mut parts = p.into_inner();
    let first = parts.next();
    let second = parts.next();

    match (first, second) {
        (Some(f), Some(s)) if f.as_rule() == Rule::class_identifier => {
            // Prefix: Type Name
            ty = Some(strip_backticks(f.as_str()));
            name = Some(s.as_str().to_owned());
            type_notation = TypeNotation::Prefix;
        }
        (Some(f), Some(s)) if f.as_rule() == Rule::variable_identifier => {
            // Postfix: Name: Type
            name = Some(f.as_str().to_owned());
            ty = Some(strip_backticks(s.as_str()));
            type_notation = TypeNotation::Postfix;
        }
        (Some(f), None) if f.as_rule() == Rule::variable_identifier => {
            // No type
            name = Some(f.as_str().to_owned());
            type_notation = TypeNotation::None;
        }
        _ => {}
    }

    Ok(Parameter {
        name: name.ok_or_else(|| ParseError::Custom("param name missing".into()))?,
        data_type: ty,
        type_notation,
    })
}

// ────────────────────────────────────────────────────────────────────────────────
// Relation statement
// ────────────────────────────────────────────────────────────────────────────────

fn scan_relation(pair: Pair<Rule>) -> Result<Relation, ParseError> {
    let mut inner = pair.into_inner();

    // Parse: class_identifier ~ cardinality? ~ relation ~ cardinality? ~ class_identifier ~ relation_label?
    let first_class = strip_backticks(
        inner
            .next()
            .ok_or_else(|| ParseError::Custom("relation: from class missing".into()))?
            .as_str()
            .trim(),
    );

    let mut cardinality1: Option<String> = None;
    let mut arrow_rule: Option<Pair<Rule>> = None;
    let mut cardinality2: Option<String> = None;
    let mut second_class: Option<String> = None;
    let mut label: Option<String> = None;

    // Parse the remaining parts in order
    for part in inner {
        match part.as_rule() {
            Rule::cardinality => {
                // Extract text between quotes
                let text = part.as_str().trim();
                let unquoted = text.trim_matches('"');
                if arrow_rule.is_none() {
                    cardinality1 = Some(unquoted.to_owned());
                } else {
                    cardinality2 = Some(unquoted.to_owned());
                }
            }
            Rule::class_identifier => {
                second_class = Some(strip_backticks(part.as_str().trim()));
            }
            Rule::relation_label => {
                // Skip the ":" and trim
                let text = part.as_str().trim();
                label = Some(text.trim_start_matches(':').trim().to_owned());
            }
            // All the arrow types
            Rule::aggregation_left
            | Rule::aggregation_right
            | Rule::composition_left
            | Rule::composition_right
            | Rule::inheritance_left
            | Rule::inheritance_right
            | Rule::realization_left
            | Rule::realization_right
            | Rule::association_left
            | Rule::association_right
            | Rule::dependency_left
            | Rule::dependency_right
            | Rule::link => {
                arrow_rule = Some(part);
            }
            _ => {}
        }
    }

    let arrow = arrow_rule.ok_or_else(|| ParseError::Custom("relation: arrow missing".into()))?;
    let second =
        second_class.ok_or_else(|| ParseError::Custom("relation: to class missing".into()))?;

    // Determine kind, line style, and whether arrow points left (swap from/to)
    let (kind, line, points_left) = match arrow.as_rule() {
        Rule::aggregation_left => (RelationKind::Aggregation, LineStyle::Solid, true),
        Rule::aggregation_right => (RelationKind::Aggregation, LineStyle::Solid, false),
        Rule::composition_left => (RelationKind::Composition, LineStyle::Solid, true),
        Rule::composition_right => (RelationKind::Composition, LineStyle::Solid, false),
        Rule::inheritance_left => (RelationKind::Inheritance, LineStyle::Solid, true),
        Rule::inheritance_right => (RelationKind::Inheritance, LineStyle::Solid, false),
        Rule::realization_left => (RelationKind::Dependency, LineStyle::Dotted, true),
        Rule::realization_right => (RelationKind::Dependency, LineStyle::Dotted, false),
        Rule::association_left => (RelationKind::Dependency, LineStyle::Solid, true),
        Rule::association_right => (RelationKind::Dependency, LineStyle::Solid, false),
        Rule::dependency_left => (RelationKind::Dependency, LineStyle::Dotted, true),
        Rule::dependency_right => (RelationKind::Dependency, LineStyle::Dotted, false),
        Rule::link => (RelationKind::Dependency, LineStyle::Solid, false),
        _ => (RelationKind::Dependency, LineStyle::Solid, false),
    };

    // If arrow points left, swap from/to AND swap cardinalities
    let (from, to, cardinality_from, cardinality_to) = if points_left {
        (second, first_class, cardinality2, cardinality1)
    } else {
        (first_class, second, cardinality1, cardinality2)
    };

    Ok(Relation {
        from,
        to,
        kind,
        line,
        cardinality_from,
        cardinality_to,
        label,
    })
}

// ────────────────────────────────────────────────────────────────────────────────
// Note statement
// ────────────────────────────────────────────────────────────────────────────────

fn scan_note(pair: Pair<Rule>) -> Result<Note, ParseError> {
    let mut target_class: Option<String> = None;
    let mut text: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::class_identifier => target_class = Some(strip_backticks(inner.as_str())),
            Rule::quoted_text => {
                // Remove surrounding quotes
                let raw = inner.as_str();
                text = Some(raw.trim_matches('"').to_owned());
            }
            _ => {}
        }
    }

    Ok(Note {
        text: text.ok_or_else(|| ParseError::Custom("note text missing".into()))?,
        target_class,
    })
}

// ────────────────────────────────────────────────────────────────────────────────
// Direction statement
// ────────────────────────────────────────────────────────────────────────────────

fn scan_direction(pair: Pair<Rule>) -> Result<Direction, ParseError> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::direction_value {
            return match inner.as_str() {
                "TB" | "TD" => Ok(Direction::TopBottom),
                "BT" => Ok(Direction::BottomTop),
                "RL" => Ok(Direction::RightLeft),
                "LR" => Ok(Direction::LeftRight),
                _ => Err(ParseError::Custom("invalid direction value".into())),
            };
        }
    }
    Err(ParseError::Custom("direction value missing".into()))
}

// ────────────────────────────────────────────────────────────────────────────────
// Namespace block
// ────────────────────────────────────────────────────────────────────────────────

fn scan_namespace_block(pair: Pair<Rule>, out: &mut Vec<Stmt>) -> Result<(), ParseError> {
    let mut namespace_name: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::namespace_name => {
                namespace_name = Some(strip_backticks(inner.as_str()));
            }
            Rule::class => {
                // Parse class and prefix with namespace
                let mut class = scan_class(inner)?;
                if let Some(ref ns) = namespace_name {
                    // Add namespace prefix to class name
                    class.name = format!("{}::{}", ns, class.name);
                    class.namespace = ns.clone();
                }
                out.push(Stmt::Class(class));
            }
            Rule::member_stmt => {
                // Parse member and prefix target with namespace
                if let Stmt::Member { mut target, member } = scan_member_stmt(inner)? {
                    if let Some(ref ns) = namespace_name {
                        target = format!("{}::{}", ns, target);
                    }
                    out.push(Stmt::Member { target, member });
                }
            }
            Rule::note => {
                // Parse note and prefix target class with namespace if present
                let mut note = scan_note(inner)?;
                if let Some(ref ns) = namespace_name {
                    if let Some(ref target_class) = note.target_class {
                        note.target_class = Some(format!("{}::{}", ns, target_class));
                    }
                }
                out.push(Stmt::Note(note));
            }
            _ => {}
        }
    }

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────────
// Second pass: apply
// ────────────────────────────────────────────────────────────────────────────────

fn apply_stmt(stmt: Stmt, diagram: &mut Diagram) {
    match stmt {
        Stmt::Class(c) => {
            let (ns, name) = split_namespace(&c.name);
            diagram
                .namespaces
                .entry(ns.to_owned())
                .or_insert_with(|| Namespace {
                    name: ns.to_owned(),
                    ..Default::default()
                })
                .classes
                .insert(name.to_owned(), c);
        }
        Stmt::Member { target, member } => {
            let (ns, name) = split_namespace(&target);
            let class = diagram
                .namespaces
                .entry(ns.to_owned())
                .or_insert_with(|| Namespace {
                    name: ns.to_owned(),
                    ..Default::default()
                })
                .classes
                .entry(name.to_owned())
                .or_insert_with(|| Class {
                    name: target.clone(),
                    generic: None,
                    annotations: Vec::new(),
                    members: Vec::new(),
                    namespace: ns.to_owned(),
                });
            class.members.push(member);
        }
        Stmt::Relation(r) => diagram.relations.push(r),
        Stmt::Note(n) => diagram.notes.push(n),
        Stmt::Direction(d) => diagram.direction = Some(d),
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Strip backticks from identifiers if present
fn strip_backticks(s: &str) -> String {
    if s.starts_with('`') && s.ends_with('`') && s.len() > 1 {
        s[1..s.len() - 1].to_owned()
    } else {
        s.to_owned()
    }
}

fn split_namespace(fq: &str) -> (&str, &str) {
    fq.rfind("::")
        .map(|idx| (&fq[..idx], &fq[idx + 2..]))
        .unwrap_or((DEFAULT_NAMESPACE, fq))
}

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::types::{
    Attribute, Class, Diagram, LineStyle, Member, Method, Namespace, Parameter, Relation,
    RelationKind, Visibility, DEFAULT_NAMESPACE,
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
}

// ────────────────────────────────────────────────────────────────────────────────
// Public entry point                                                             
// ────────────────────────────────────────────────────────────────────────────────

pub fn parse(src: &str) -> Result<Diagram, ParseError> {
    // 1) let Pest build a rich tree (inc. all tokens)
    let mut outer = MermaidParser::parse(Rule::diagram, src)?;
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
    for stmt in stmts {
        apply_stmt(stmt, &mut diagram);
    }
    Ok(diagram)
}

// ────────────────────────────────────────────────────────────────────────────────
// First pass: build lightweight statement enums                                  
// ────────────────────────────────────────────────────────────────────────────────

fn collect_stmt(pair: Pair<Rule>, out: &mut Vec<Stmt>) -> Result<(), ParseError> {
    match pair.as_rule() {
        Rule::class => out.push(Stmt::Class(scan_class(pair)?)),
        Rule::member_stmt => out.push(scan_member_stmt(pair)?),
        Rule::relation_stmt => out.push(Stmt::Relation(scan_relation(pair)?)),
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
            Rule::class_identifier => id = Some(inner.as_str().to_owned()),
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
    let target = inner
        .next()
        .ok_or_else(|| ParseError::Custom("member: target missing".into()))?
        .as_str()
        .trim()
        .to_owned();
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

fn parse_attribute(
    attr: Pair<Rule>,
    is_static: bool,
) -> Result<Attribute, ParseError> {
    let mut visibility = Visibility::Unspecified;
    let mut name: Option<String> = None;
    let mut ty: Option<String> = None;
    for p in attr.into_inner() {
        match p.as_rule() {
            Rule::visibility => visibility = Visibility::from(p.as_str().chars().next().unwrap()),
            Rule::variable_identifier => name = Some(p.as_str().to_owned()),
            Rule::class_identifier => ty = Some(p.as_str().to_owned()),
            _ => {}
        }
    }
    Ok(Attribute {
        visibility,
        name: name.ok_or_else(|| ParseError::Custom("attr name missing".into()))?,
        data_type: ty,
        is_static,
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

    for p in meth.into_inner() {
        match p.as_rule() {
            Rule::visibility => visibility = Visibility::from(p.as_str().chars().next().unwrap()),
            Rule::method_identifier => name = Some(p.as_str().to_owned()),
            Rule::method_parameter => params = parse_parameters(p)?,
            Rule::class_identifier => return_type = Some(p.as_str().to_owned()),
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
    })
}

fn parse_parameters(list: Pair<Rule>) -> Result<Vec<Parameter>, ParseError> {
    let mut v = Vec::<Parameter>::new();
    for p in list.into_inner() { // parameter_list → many parameter
        if p.as_rule() == Rule::parameter {
            v.push(parse_parameter(p)?);
        }
    }
    Ok(v)
}

fn parse_parameter(p: Pair<Rule>) -> Result<Parameter, ParseError> {
    let mut ty: Option<String> = None;
    let mut name: Option<String> = None;
    for part in p.into_inner() {
        match part.as_rule() {
            Rule::class_identifier => ty = Some(part.as_str().to_owned()),
            Rule::variable_identifier => name = Some(part.as_str().to_owned()),
            _ => {}
        }
    }
    Ok(Parameter {
        name: name.ok_or_else(|| ParseError::Custom("param name missing".into()))?,
        data_type: ty,
    })
}

// ────────────────────────────────────────────────────────────────────────────────
// Relation statement                                                             
// ────────────────────────────────────────────────────────────────────────────────

fn scan_relation(pair: Pair<Rule>) -> Result<Relation, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| ParseError::Custom("relation: from missing".into()))?
        .as_str()
        .trim()
        .to_owned();
    let arrow = inner
        .next()
        .ok_or_else(|| ParseError::Custom("relation: arrow missing".into()))?;
    let second = inner
        .next()
        .ok_or_else(|| ParseError::Custom("relation: to missing".into()))?
        .as_str()
        .trim()
        .to_owned();

    // Determine kind, line style, and whether arrow points left (swap from/to)
    let (kind, line, points_left) = match arrow.as_rule() {
        Rule::aggregation_left => (RelationKind::Aggregation, LineStyle::Solid, true),
        Rule::aggregation_right => (RelationKind::Aggregation, LineStyle::Solid, false),
        Rule::composition_left => (RelationKind::Composition, LineStyle::Solid, true),
        Rule::composition_right => (RelationKind::Composition, LineStyle::Solid, false),
        Rule::inheritance_left => (RelationKind::Extension, LineStyle::Solid, true),
        Rule::inheritance_right => (RelationKind::Extension, LineStyle::Solid, false),
        Rule::realization_left => (RelationKind::Dependency, LineStyle::Dotted, true),
        Rule::realization_right => (RelationKind::Dependency, LineStyle::Dotted, false),
        Rule::association_left => (RelationKind::Dependency, LineStyle::Solid, true),
        Rule::association_right => (RelationKind::Dependency, LineStyle::Solid, false),
        Rule::dependency_left => (RelationKind::Dependency, LineStyle::Dotted, true),
        Rule::dependency_right => (RelationKind::Dependency, LineStyle::Dotted, false),
        Rule::link => (RelationKind::Dependency, LineStyle::Solid, false),
        _ => (RelationKind::Dependency, LineStyle::Solid, false),
    };

    // If arrow points left, swap from and to
    let (from, to) = if points_left {
        (second, first)
    } else {
        (first, second)
    };

    Ok(Relation {
        from,
        to,
        kind,
        line,
        label_from: None,
        label_to: None,
    })
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
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Helpers                                                                        
// ────────────────────────────────────────────────────────────────────────────────

fn split_namespace(fq: &str) -> (&str, &str) {
    fq.rfind("::")
        .map(|idx| (&fq[..idx], &fq[idx + 2..]))
        .unwrap_or((DEFAULT_NAMESPACE, fq))
}

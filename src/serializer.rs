//! Serialize Mermaid diagram structures back to text format

use crate::types::{
    Class, DEFAULT_NAMESPACE, Diagram, Direction, LineStyle, Member, Note, Relation, RelationKind,
    TypeNotation, Visibility,
};
use std::fmt::Write;

/// Convert visibility to Mermaid symbol
fn visibility_symbol(vis: Visibility) -> &'static str {
    match vis {
        Visibility::Public => "+",
        Visibility::Private => "-",
        Visibility::Protected => "#",
        Visibility::Package => "~",
        Visibility::Unspecified => "",
    }
}

/// Escape class name with backticks if it contains special characters
fn escape_class_name(name: &str) -> String {
    // Check if name needs backtick escaping (contains spaces or special chars)
    if name.contains(|c: char| c.is_whitespace() || "!@#$%^&*()".contains(c)) {
        format!("`{}`", name)
    } else {
        name.to_string()
    }
}

/// Serialize a single member (attribute or method)
fn serialize_member(member: &Member, output: &mut String) {
    match member {
        Member::Attribute(attr) => {
            write!(output, "{}", visibility_symbol(attr.visibility)).unwrap();
            if attr.is_static {
                output.push('$');
            }

            // Use the notation style that was parsed
            match attr.type_notation {
                TypeNotation::Prefix => {
                    // Type Name
                    if let Some(data_type) = &attr.data_type {
                        write!(output, "{} {}", escape_class_name(data_type), attr.name).unwrap();
                    } else {
                        write!(output, "{}", attr.name).unwrap();
                    }
                }
                TypeNotation::Postfix => {
                    // Name: Type
                    write!(output, "{}", attr.name).unwrap();
                    if let Some(data_type) = &attr.data_type {
                        write!(output, ": {}", escape_class_name(data_type)).unwrap();
                    }
                }
                TypeNotation::None => {
                    write!(output, "{}", attr.name).unwrap();
                }
            }
        }
        Member::Method(method) => {
            write!(output, "{}", visibility_symbol(method.visibility)).unwrap();
            if method.is_static {
                output.push('$');
            }
            if method.is_abstract {
                output.push('*');
            }

            write!(output, "{}(", method.name).unwrap();

            // Parameters
            for (i, param) in method.parameters.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }

                match param.type_notation {
                    TypeNotation::Prefix => {
                        // Type Name
                        if let Some(data_type) = &param.data_type {
                            write!(output, "{} {}", escape_class_name(data_type), param.name)
                                .unwrap();
                        } else {
                            write!(output, "{}", param.name).unwrap();
                        }
                    }
                    TypeNotation::Postfix => {
                        // Name: Type
                        write!(output, "{}", param.name).unwrap();
                        if let Some(data_type) = &param.data_type {
                            write!(output, ": {}", escape_class_name(data_type)).unwrap();
                        }
                    }
                    TypeNotation::None => {
                        write!(output, "{}", param.name).unwrap();
                    }
                }
            }
            output.push(')');

            // Return type (always postfix in mermaid - no colon)
            if let Some(return_type) = &method.return_type {
                write!(output, " {}", escape_class_name(return_type)).unwrap();
            }
        }
    }
}

/// Serialize a single class to Mermaid format (one statement per line)
fn serialize_class(class: &Class, output: &mut String) {
    let class_name = escape_class_name(&class.name);

    // Class declaration
    writeln!(output, "class {}", class_name).unwrap();

    // Members - one per line with ClassName : member syntax
    for member in &class.members {
        write!(output, "{} : ", class_name).unwrap();
        serialize_member(member, output);
        output.push('\n');
    }
}

/// Serialize a relation to Mermaid format
fn serialize_relation(relation: &Relation, output: &mut String) {
    let from_name = escape_class_name(&relation.tail);
    let to_name = escape_class_name(&relation.head);

    write!(output, "{}", from_name).unwrap();

    // Add cardinality_from if present
    if let Some(card) = &relation.cardinality_tail {
        write!(output, " \"{}\"", card).unwrap();
    }

    output.push(' ');

    // Build the relation symbol (always right-pointing since parser normalizes)
    match (relation.kind, relation.line) {
        (RelationKind::Aggregation, LineStyle::Solid) => output.push_str("--o"),
        (RelationKind::Aggregation, LineStyle::Dotted) => output.push_str("..o"),
        (RelationKind::Composition, LineStyle::Solid) => output.push_str("--*"),
        (RelationKind::Composition, LineStyle::Dotted) => output.push_str("..*"),
        (RelationKind::Inheritance, LineStyle::Solid) => output.push_str("--|>"),
        (RelationKind::Inheritance, LineStyle::Dotted) => output.push_str("..|>"),
        (RelationKind::Dependency, LineStyle::Solid) => output.push_str("-->"),
        (RelationKind::Dependency, LineStyle::Dotted) => output.push_str("..>"),
        (RelationKind::Lollipop, _) => output.push_str("--o"),
    }

    // Add cardinality_to if present
    if let Some(card) = &relation.cardinality_head {
        write!(output, " \"{}\"", card).unwrap();
    }

    write!(output, " {}", to_name).unwrap();

    // Add label if present
    if let Some(label) = &relation.label {
        write!(output, " : {}", label).unwrap();
    }

    output.push('\n');
}

/// Serialize a note to Mermaid format
fn serialize_note(note: &Note, output: &mut String) {
    if let Some(target_class) = &note.target_class {
        writeln!(
            output,
            "note for {} \"{}\"",
            escape_class_name(target_class),
            note.text
        )
        .unwrap();
    } else {
        writeln!(output, "note \"{}\"", note.text).unwrap();
    }
}

/// Serialize direction to Mermaid format
fn serialize_direction(direction: Direction, output: &mut String) {
    let dir_str = match direction {
        Direction::TopBottom => "TB",
        Direction::BottomTop => "BT",
        Direction::RightLeft => "RL",
        Direction::LeftRight => "LR",
    };
    writeln!(output, "direction {}", dir_str).unwrap();
}

/// Serialize entire diagram to Mermaid text format
/// Each statement is on its own line (except for quoted strings in notes and backtick-escaped names)
pub fn serialize_diagram(diagram: &Diagram) -> String {
    let mut output = String::new();

    // Serialize YAML frontmatter if present
    if let Some(yaml) = &diagram.yaml {
        output.push_str("---\n");
        output.push_str(&serde_yml::to_string(yaml).unwrap_or_default());
        output.push_str("---\n");
    }

    output.push_str("classDiagram\n");

    // Serialize direction if present
    if let Some(direction) = diagram.direction {
        serialize_direction(direction, &mut output);
    }

    // Separate default namespace from named namespaces
    let mut default_classes = Vec::new();
    let mut namespaced_classes: Vec<(&String, &crate::types::Namespace)> = Vec::new();

    for (namespace_name, namespace) in &diagram.namespaces {
        if namespace_name == DEFAULT_NAMESPACE || namespace_name.is_empty() {
            for class in namespace.classes.values() {
                default_classes.push(class);
            }
        } else {
            namespaced_classes.push((namespace_name, namespace));
        }
    }

    // Serialize default namespace classes
    for class in default_classes {
        serialize_class(class, &mut output);
    }

    // Serialize namespaced classes in namespace blocks
    for (namespace_name, namespace) in namespaced_classes {
        writeln!(output, "namespace {} {{", escape_class_name(namespace_name)).unwrap();
        for class in namespace.classes.values() {
            // Serialize class without namespace prefix (it's already in the block context)
            let class_name_only = class
                .name
                .strip_prefix(&format!("{}::", namespace_name))
                .unwrap_or(&class.name);
            let class_name = escape_class_name(class_name_only);

            // Class declaration
            writeln!(output, "class {}", class_name).unwrap();

            // Members
            for member in &class.members {
                write!(output, "{} : ", class_name).unwrap();
                serialize_member(member, &mut output);
                output.push('\n');
            }
        }
        output.push_str("}\n");
    }

    // Serialize relations
    for relation in &diagram.relations {
        serialize_relation(relation, &mut output);
    }

    // Serialize notes
    for note in &diagram.notes {
        serialize_note(note, &mut output);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_serialize_simple_class() {
        let mermaid = "classDiagram\nclass Animal\n";
        let diagram = parse(mermaid).unwrap();
        let serialized = serialize_diagram(&diagram);
        assert!(serialized.contains("class Animal"));
    }

    #[test]
    fn test_serialize_backtick_names() {
        let mermaid = "classDiagram\nclass `Animal Class!`\n";
        let diagram = parse(mermaid).unwrap();
        let serialized = serialize_diagram(&diagram);
        assert!(serialized.contains("`Animal Class!`"));
    }

    #[test]
    fn test_serialize_with_direction() {
        let mermaid = "classDiagram\ndirection RL\nclass Test\n";
        let diagram = parse(mermaid).unwrap();
        let serialized = serialize_diagram(&diagram);
        assert!(serialized.contains("direction RL"));
    }

    #[test]
    fn test_serialize_note() {
        let mermaid = "classDiagram\nclass Test\nnote \"General note\"\n";
        let diagram = parse(mermaid).unwrap();
        let serialized = serialize_diagram(&diagram);
        assert!(serialized.contains("note \"General note\""));
    }

    #[test]
    fn test_serialize_note_for_class() {
        let mermaid = "classDiagram\nclass Test\nnote for Test \"Class note\"\n";
        let diagram = parse(mermaid).unwrap();
        let serialized = serialize_diagram(&diagram);
        assert!(serialized.contains("note for Test \"Class note\""));
    }
}

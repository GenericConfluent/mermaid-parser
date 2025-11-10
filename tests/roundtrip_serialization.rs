use mermaid_parser::parserv2::parse_mermaid as parse;
use mermaid_parser::serializer::serialize_diagram;

#[test]
fn test_roundtrip_simple_class() {
    let input = "classDiagram\nclass Animal\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    // Parse the output again
    let diagram2 = parse(&output).unwrap();

    // Should have same number of classes
    assert_eq!(
        diagram
            .namespaces
            .values()
            .map(|ns| ns.classes.len())
            .sum::<usize>(),
        diagram2
            .namespaces
            .values()
            .map(|ns| ns.classes.len())
            .sum::<usize>()
    );
}

#[test]
fn test_roundtrip_backtick_names() {
    let input =
        "classDiagram\nclass `Animal Class!`\nclass `Car Class`\n`Animal Class!` --> `Car Class`\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Input:\n{}", input);
    println!("Output:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    assert_eq!(diagram.relations.len(), diagram2.relations.len());
}

#[test]
fn test_roundtrip_members_prefix_notation() {
    let input = "classDiagram\nclass Test\nTest : +int x\nTest : +method(int a) int\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Output:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    // Check member count
    let class1 = diagram
        .namespaces
        .values()
        .flat_map(|ns| ns.classes.values())
        .next()
        .unwrap();
    let class2 = diagram2
        .namespaces
        .values()
        .flat_map(|ns| ns.classes.values())
        .next()
        .unwrap();

    assert_eq!(class1.members.len(), class2.members.len());
}

#[test]
fn test_roundtrip_members_postfix_notation() {
    let input = "classDiagram\nclass Test\nTest : +x: int\nTest : +method(a: int) String\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Output:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    let class1 = diagram
        .namespaces
        .values()
        .flat_map(|ns| ns.classes.values())
        .next()
        .unwrap();
    let class2 = diagram2
        .namespaces
        .values()
        .flat_map(|ns| ns.classes.values())
        .next()
        .unwrap();

    assert_eq!(class1.members.len(), class2.members.len());
}

#[test]
fn test_roundtrip_relations_with_cardinality() {
    let input = "classDiagram\nclass A\nclass B\nA \"1\" --> \"*\" B : uses\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Output:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    assert_eq!(diagram.relations.len(), diagram2.relations.len());
    assert_eq!(
        diagram.relations[0].cardinality_tail,
        diagram2.relations[0].cardinality_tail
    );
    assert_eq!(
        diagram.relations[0].cardinality_head,
        diagram2.relations[0].cardinality_head
    );
}

#[test]
fn test_roundtrip_direction() {
    let input = "classDiagram\ndirection RL\nclass Test\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    let diagram2 = parse(&output).unwrap();

    assert_eq!(diagram.direction, diagram2.direction);
}

#[test]
fn test_roundtrip_notes() {
    let input = "classDiagram\nclass Test\nnote \"General note\"\nnote for Test \"Class note\"\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Output:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    assert_eq!(diagram.notes.len(), diagram2.notes.len());
}

#[test]
fn test_roundtrip_namespace() {
    let input =
        "classDiagram\nnamespace MyNamespace {\nclass Test\nTest : +int x\n}\nclass OutsideClass\n";
    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Input:\n{}", input);
    println!("Output:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    // Should have same number of namespaces
    assert_eq!(diagram.namespaces.len(), diagram2.namespaces.len());
    assert!(diagram.namespaces.contains_key("MyNamespace"));
    assert!(diagram2.namespaces.contains_key("MyNamespace"));

    // Check classes in namespace
    let ns1 = diagram.namespaces.get("MyNamespace").unwrap();
    let ns2 = diagram2.namespaces.get("MyNamespace").unwrap();
    assert_eq!(ns1.classes.len(), ns2.classes.len());
}

#[test]
fn test_roundtrip_complex_diagram() {
    let input = r#"classDiagram
direction RL
class `Animal Class!`
class Vehicle
`Animal Class!` : +int age
`Animal Class!` : +name: String
`Animal Class!` : +move(int distance) void
Vehicle : +speed: int
Vehicle : +drive(a: int, b: String) int
`Animal Class!` "1" --> "*" Vehicle : owns
note "This is a test diagram"
note for Vehicle "Vehicles are fast"
"#;

    let diagram = parse(input).unwrap();
    let output = serialize_diagram(&diagram);

    println!("Original:\n{}", input);
    println!("Serialized:\n{}", output);

    let diagram2 = parse(&output).unwrap();

    // Verify all major components
    assert_eq!(diagram.direction, diagram2.direction);
    assert_eq!(diagram.relations.len(), diagram2.relations.len());
    assert_eq!(diagram.notes.len(), diagram2.notes.len());

    let total_classes1: usize = diagram.namespaces.values().map(|ns| ns.classes.len()).sum();
    let total_classes2: usize = diagram2
        .namespaces
        .values()
        .map(|ns| ns.classes.len())
        .sum();

    assert_eq!(total_classes1, total_classes2);
}

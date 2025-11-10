use mermaid_parser::parserv2::parse_mermaid as parse;
use mermaid_parser::serializer::serialize_diagram;

fn main() {
    // Example 1: Parse and serialize a simple diagram
    let input = r#"classDiagram
direction RL
class `Animal Class!`
class Vehicle
`Animal Class!` : +int age
`Animal Class!` : +name: String
`Animal Class!` : +move(distance: int) void
Vehicle : +speed: int
Vehicle : +drive(int a, String b) int
`Animal Class!` "1" --> "*" Vehicle : owns
note "This is a test diagram"
note for Vehicle "Vehicles are fast"
"#;

    println!("Original Input:");
    println!("{}", input);
    println!("\n{}", "=".repeat(60));

    // Parse the diagram
    let diagram = parse(input).expect("Failed to parse diagram");

    // Serialize it back
    let output = serialize_diagram(&diagram);

    println!("\nSerialized Output:");
    println!("{}", output);
    println!("\n{}", "=".repeat(60));

    // Verify round-trip
    let diagram2 = parse(&output).expect("Failed to parse serialized output");
    println!("\n✓ Round-trip successful!");
    println!("  Classes: {}", diagram2.namespaces.values().map(|ns| ns.classes.len()).sum::<usize>());
    println!("  Relations: {}", diagram2.relations.len());
    println!("  Notes: {}", diagram2.notes.len());
    println!("  Direction: {:?}", diagram2.direction);
}

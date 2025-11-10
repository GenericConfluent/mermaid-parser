use mermaid_parser::parserv2::parse_mermaid as parse;
use mermaid_parser::serializer::serialize_diagram;

fn main() {
    let input = r#"classDiagram
direction LR
namespace Animals {
class Dog
class Cat
Dog : +String name
Dog : +bark() void
Cat : +name: String
Cat : +meow() void
}
namespace Vehicles {
class Car
class Bike
Car : +speed: int
Car : +drive(distance: int) void
Bike : +int gears
Bike : +ride() void
}
Animals::Dog "1" --> "*" Vehicles::Car : chases
Animals::Cat --> Vehicles::Bike : ignores
note for Animals::Cat "Cats are independent"
note "Complex namespace example"
"#;

    println!("Input Diagram:");
    println!("{}", input);
    println!("\n{}", "=".repeat(70));

    let diagram = parse(input).expect("Failed to parse");

    println!("\nParsed Structure:");
    println!("  Direction: {:?}", diagram.direction);
    println!("  Namespaces: {}", diagram.namespaces.len());
    for (ns_name, ns) in &diagram.namespaces {
        if ns_name.is_empty() {
            println!("    - Default namespace: {} classes", ns.classes.len());
        } else {
            println!("    - '{}': {} classes", ns_name, ns.classes.len());
        }
    }
    println!("  Relations: {}", diagram.relations.len());
    println!("  Notes: {}", diagram.notes.len());

    println!("\n{}", "=".repeat(70));
    println!("\nSerialized Output:");
    let output = serialize_diagram(&diagram);
    println!("{}", output);

    println!("{}", "=".repeat(70));

    // Verify round-trip
    let diagram2 = parse(&output).expect("Round-trip parsing failed");

    println!("\n✓ Round-trip verification passed!");
    println!(
        "  Namespaces: {} -> {}",
        diagram.namespaces.len(),
        diagram2.namespaces.len()
    );
    println!(
        "  Relations: {} -> {}",
        diagram.relations.len(),
        diagram2.relations.len()
    );
    println!(
        "  Notes: {} -> {}",
        diagram.notes.len(),
        diagram2.notes.len()
    );
}

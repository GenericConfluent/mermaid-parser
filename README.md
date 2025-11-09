# Mermaid Class Diagram Parser & Serializer

A Rust library for parsing and serializing Mermaid class diagrams. This library provides:
- Full parsing of Mermaid class diagram syntax (using Pest parser)
- Serialization back to Mermaid format (round-trip support)
- Support for both prefix (`int x`) and postfix (`x: int`) type notation
- Each field/method/parameter stores which type notation was used
- Support for backtick-escaped class names with special characters

## Usage

### Parsing
```rust
use mermaid_parser::parser::parse;

let mermaid = r#"classDiagram
class Animal
Animal : +int age
Animal : +move(distance: int) void
"#;

let diagram = parse(mermaid).expect("Failed to parse");
```

### Serialization
```rust
use mermaid_parser::serializer::serialize_diagram;

let output = serialize_diagram(&diagram);
println!("{}", output);
```

### Round-trip Example
```rust
use mermaid_parser::parser::parse;
use mermaid_parser::serializer::serialize_diagram;

let original = "classDiagram\nclass Animal\n";
let diagram = parse(original).unwrap();
let serialized = serialize_diagram(&diagram);
let diagram2 = parse(&serialized).unwrap();
// diagram and diagram2 are equivalent
```

See `examples/serialize.rs` for a complete example.

# Supported Syntax 
- [X] Frontmatter YAML
- [ ] Class Definition
  - [X] Plain
  - [ ] Annotations (`<<interface>>`, `<<abstract>>`, ...)
  - [ ] Class Labels
  - [X] Backtick Escape
- [X] Member Definition
  - [X] Visibility 
- [ ] Relationships 
  - [X] One Way (`<|--`, `*--`, `o--`, `--`, `..|>`, `..`, and mirror images)
  - [X] Labels
  - [ ] Two way
  - [ ] Lolipop interfaces
- [X] Namespaces
  - [X] Namespace blocks (`namespace Name { ... }`)
  - [X] Fully qualified names (`Namespace::ClassName`)
- [X] Cardinality/Multiplicity
- [X] Comments
- [X] Diagram Direction
- [ ] Interaction 
- [X] Notes
  - [X] Plain
  - [X] Class Notes
- [ ] Styling

# Credit 
- https://github.com/mermaid-js/mermaid/blob/develop/packages/mermaid/src/diagrams/class/parser/classDiagram.jison

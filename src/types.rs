use std::collections::HashMap;

/// “default” (no explicit namespace in the diagram)  
pub const DEFAULT_NAMESPACE: &str = "";

/// Public/Private/… like in Mermaid (# + ~ - or empty)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,     // +
    Private,    // -
    Protected,  // #
    Package,    // ~
    Unspecified,
}

impl From<char> for Visibility {
    fn from(c: char) -> Self {
        match c {
            '+' => Visibility::Public,
            '-' => Visibility::Private,
            '#' => Visibility::Protected,
            '~' => Visibility::Package,
            _   => Visibility::Unspecified,
        }
    }
}

/// A single parameter in a method signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub data_type: Option<String>,     // `None` if omitted in the diagram
}

/// A member inside a class box
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Member {
    /// `+fieldName: Type`
    Attribute(Attribute),

    /// `+methodName(arg: Type): ReturnType`
    Method(Method),
}

/// Data that only an **attribute** has
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub visibility: Visibility,
    pub name: String,
    pub data_type: Option<String>,
    pub is_static: bool,               // “$” in Mermaid
}

/// Data that only a **method** has
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    pub visibility: Visibility,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub is_static: bool,               // “$” in Mermaid
    pub is_abstract: bool,             // “*” in Mermaid
}

/// A single class or interface in the diagram
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,                 // Fully-qualified (incl. namespace)
    pub generic: Option<String>,      // the “~T” from `Foo~T~`
    pub annotations: Vec<String>,     // <<interface>>, <<service>> …
    pub members: Vec<Member>,         // <── was Vec<ClassMember>
    pub namespace: String,            // DEFAULT_NAMESPACE if missing
}

/// Solid vs dotted line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle { Solid, Dotted }

/// Mermaid’s five relation arrow-heads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Aggregation,   // o--
    Composition,   // *--
    Extension,     // <|--
    Dependency,    // <.. 
    Lollipop,      // o()
}

/// Edge between two classes
#[derive(Debug, Clone)]
pub struct Relation {
    pub from: String,                    // fully-qualified class names
    pub to: String,
    pub kind: RelationKind,
    pub line: LineStyle,
    pub cardinality_from: Option<String>, // e.g., "1", "*", "1..*"
    pub cardinality_to: Option<String>,   // e.g., "1", "*", "1..*"
    pub label: Option<String>,            // relationship label text
}

/// Recursive namespace tree
#[derive(Debug, Default)]
pub struct Namespace {
    pub name: String,
    pub classes: HashMap<String, Class>,      // name ➜ class
    pub children: HashMap<String, Namespace>, // nested namespaces
}

/// Whole diagram
#[derive(Debug, Default)]
pub struct Diagram {
    pub namespaces: HashMap<String, Namespace>,
    pub relations:  Vec<Relation>,
    pub yaml: Option<serde_yml::Value>,
}

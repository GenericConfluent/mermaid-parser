use std::{borrow::Cow, collections::HashMap};

/// "default" (no explicit namespace in the diagram)
pub const DEFAULT_NAMESPACE: &str = "";

type Sym<'a> = Cow<'a, str>;
type OptSym<'a> = Option<Sym<'a>>;

/// Direction of the diagram layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    TopBottom, // TB or TD
    BottomTop, // BT
    RightLeft, // RL
    LeftRight, // LR
}

/// Type annotation notation style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeNotation {
    Prefix,  // Type Name (e.g., "int x")
    Postfix, // Name: Type (e.g., "x: int")
    None,    // No type specified
}

/// Public/Private/… like in Mermaid (# + ~ - or empty)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
    Unspecified,
}

impl From<char> for Visibility {
    fn from(c: char) -> Self {
        match c {
            '+' => Visibility::Public,
            '-' => Visibility::Private,
            '#' => Visibility::Protected,
            '~' => Visibility::Package,
            _ => Visibility::Unspecified,
        }
    }
}

/// A single parameter in a method signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter<'source> {
    pub name: Sym<'source>,
    pub data_type: OptSym<'source>, // `None` if omitted in the diagram
    pub type_notation: TypeNotation, // Prefix, Postfix, or None
}

/// A member inside a class box
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Member<'source> {
    /// `+fieldName: Type`
    Attribute(Attribute<'source>),

    /// `+methodName(arg: Type): ReturnType`
    Method(Method<'source>),
}

/// Data that only an **attribute** has
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute<'source> {
    pub visibility: Visibility,
    pub name: Sym<'source>,
    pub data_type: OptSym<'source>,
    pub is_static: bool,             // "$" in Mermaid
    pub type_notation: TypeNotation, // Prefix, Postfix, or None
}

/// Data that only a **method** has
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method<'source> {
    pub visibility: Visibility,
    pub name: Sym<'source>,
    pub parameters: Vec<Parameter<'source>>,
    pub return_type: OptSym<'source>,
    pub is_static: bool,                    // "$" in Mermaid
    pub is_abstract: bool,                  // "*" in Mermaid
    pub return_type_notation: TypeNotation, // Prefix, Postfix, or None
}

/// A single class or interface in the diagram
#[derive(Debug, Clone)]
pub struct Class<'source> {
    pub name: Sym<'source>,             // Fully-qualified (incl. namespace)
    pub annotations: Vec<Sym<'source>>, // <<interface>>, <<service>> …
    pub members: Vec<Member<'source>>,  // <── was Vec<ClassMember>
}

/// Mermaid’s five relation arrow-heads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Inheritance, // <|--
    Composition, // *--
    Aggregation, // o--
    Association, // -->
    SolidLink,   // --
    Dependency,  // <..
    Realization, // ..|>
    DashLink,    // ..
    Lollipop,    // --()
}

/// Edge between two classes
#[derive(Debug, Clone)]
pub struct Relation<'source> {
    /// The class name which the tail comes FROM.
    pub tail: Sym<'source>, // fully-qualified class names
    /// The class name which the head is attached TO
    pub head: Sym<'source>,
    pub kind: RelationKind,
    pub cardinality_tail: OptSym<'source>, // e.g., "1", "*", "1..*"
    pub cardinality_head: OptSym<'source>, // e.g., "1", "*", "1..*"
    pub label: OptSym<'source>,            // relationship label text
}

/// A note in the diagram - either general or attached to a specific class
#[derive(Debug, Clone)]
pub struct Note<'source> {
    pub text: Sym<'source>,            // the note content
    pub target_class: OptSym<'source>, // None for general notes, Some(class) for "note for ClassName"
}

/// Recursive namespace tree
#[derive(Debug, Default)]
pub struct Namespace<'source> {
    pub name: Sym<'source>,
    pub classes: HashMap<Sym<'source>, Class<'source>>, // name ➜ class
    pub children: HashMap<Sym<'source>, Namespace<'source>>, // nested namespaces
}

/// Whole diagram
#[derive(Debug, Default)]
pub struct Diagram<'source> {
    pub namespaces: HashMap<Sym<'source>, Namespace<'source>>,
    pub relations: Vec<Relation<'source>>,
    pub notes: Vec<Note<'source>>,
    pub direction: Option<Direction>,
    pub yaml: Option<serde_yml::Value>,
}

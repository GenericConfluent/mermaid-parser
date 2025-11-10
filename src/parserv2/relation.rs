use crate::types::RelationKind;

use super::{IResult, Stmt};

use nom::{
    self, PResult, Parser,
    branch::alt,
    bytes::complete::*,
    character::{
        complete::{line_ending, multispace0},
        none_of,
    },
    combinator::opt,
    error::ParseError,
    sequence::delimited,
};

enum Direction {
    Forward,
    Backward,
}

pub fn relation_stmt(s: &str) -> IResult<&str, Stmt> {
    todo!()
}

pub fn relation_kind(s: &str) -> IResult<&str, (RelationKind, Direction)> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::types::RelationKind;

    use super::*;

    fn check_relation_kind(
        lhs: &str,
        lhs_mult: Option<&str>,
        op: &str,
        rhs_mult: Option<&str>,
        rhs: &str,
        label: Option<&str>,
        expect_from: &str,
        expect_to: &str,
        expect_op: RelationKind,
    ) {
        let mut s = String::new();
        s.push_str(lhs);
        s.push(' ');
        if let Some(lhs_mult) = lhs_mult {
            s.push('"');
            s.push_str(lhs_mult);
            s.push('"');
            s.push(' ');
        }

        s.push_str(op);

        if let Some(rhs_mult) = rhs_mult {
            s.push('"');
            s.push_str(rhs_mult);
            s.push('"');
            s.push(' ');
        }
        s.push_str(rhs);

        if let Some(label) = label {
            s.push_str(" : ");
            s.push_str(label);
        }

        let (rem, Stmt::Relation(rel)) = relation_stmt(&s).expect("Failed to parse") else {
            panic!("We should only be returning Stmt::Relation");
        };
        assert!(rem.is_empty(), "There should be nothing left");
        assert_eq!(rel.to, expect_to, "Wrong target");
        assert_eq!(rel.from, expect_from, "Wrong source");
        assert_eq!(rel.kind, expect_op, "Wrong relationship kind");
    }

    fn check_from_to(forward_op: &str, kind: RelationKind) {
        let backward_op = forward_op.chars().rev().collect::<String>();
        check_relation_kind(
            "from", None, forward_op, None, "to", None, "from", "to", kind,
        );
        check_relation_kind(
            "to",
            None,
            &backward_op,
            None,
            "from",
            None,
            "from",
            "to",
            kind,
        );
    }

    fn check_backtick_escape(forward_op: &str, kind: RelationKind) {
        let lhs = "`Hello world. $! `";
        let rhs = "`A.:!#neat`";
        check_relation_kind(
            lhs,
            None,
            forward_op,
            None,
            rhs,
            None,
            lhs.trim_matches('`'),
            rhs.trim_matches('`'),
            kind,
        );
    }

    // <|--	Inheritance
    // *--	Composition
    // o--	Aggregation
    // -->	Association
    // --	Link (Solid)
    // ..>	Dependency
    // ..|>	Realization
    // ..	Link (Dashed)
    #[test]
    fn test_relation_stmt_inheritance() {
        check_from_to("--|>", RelationKind::Inheritance);
        check_backtick_escape("--|>", RelationKind::Inheritance);
    }

    #[test]
    fn test_relation_stmt_composition() {
        check_from_to("--*", RelationKind::Inheritance);
        check_backtick_escape("--*", RelationKind::Inheritance);
    }

    #[test]
    fn test_relation_stmt_aggregation() {
        check_from_to("--o", RelationKind::Inheritance);
        check_backtick_escape("--o", RelationKind::Inheritance);
    }

    #[test]
    fn test_relation_stmt_link_solid() {
        check_from_to("--", RelationKind::SolidLink);
        check_backtick_escape("--", RelationKind::SolidLink);
    }

    #[test]
    fn test_relation_stmt_dependency() {
        check_from_to("..>", RelationKind::Dependency);
        check_backtick_escape("..>", RelationKind::Dependency);
    }

    #[test]
    fn test_relation_stmt_realization() {
        check_from_to("..>", RelationKind::Dependency);
        check_backtick_escape("..>", RelationKind::Dependency);
    }

    #[test]
    fn test_relation_stmt_link_dash() {
        check_from_to("..", RelationKind::SolidLink);
        check_backtick_escape("..", RelationKind::SolidLink);
    }
}

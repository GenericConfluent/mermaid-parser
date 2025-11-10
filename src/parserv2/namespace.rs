use super::IResult;

use crate::types::{Direction, Namespace, Note};

pub fn namespace_stmt(s: &str) -> IResult<&str, Namespace> {
    todo!()
}

pub fn namespace_context() {
    todo!()
}

pub fn stmt_note(s: &str) -> IResult<&str, Note> {
    todo!()
}

pub fn stmt_direction(s: &str) -> IResult<&str, Direction> {
    todo!()
}

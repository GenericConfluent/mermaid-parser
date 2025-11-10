use super::{IResult, Stmt};
use crate::types::{Direction, Namespace, Note};

pub fn namespace_stmt<'source>(s: &'source str) -> IResult<&'source str, Stmt<'source>> {
    todo!()
}

pub fn namespace_context() {
    todo!()
}

pub fn stmt_note<'source>(s: &'source str) -> IResult<&'source str, Note<'source>> {
    todo!()
}

pub fn stmt_direction(s: &str) -> IResult<&str, Direction> {
    todo!()
}

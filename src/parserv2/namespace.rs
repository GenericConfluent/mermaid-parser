use nom::IResult;

use crate::types::{Direction, Note};

pub fn namespace_context() {
    todo!()
}

pub fn stmt_note(s: &str) -> IResult<&str, Note> {
    todo!()
}

pub fn stmt_direction(s: &str) -> IResult<&str, Direction> {
    todo!()
}

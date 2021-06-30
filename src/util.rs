use crate::expression::Value;

pub const EX_USAGE: i32 = 64;
pub const EX_DATAERR: i32 = 65;
pub const EX_SOFTWARE: i32 = 70;
pub const MAXIMUM_PARAMETER_COUNT: usize = 255;

pub enum UnwindType {
    Error,
    Return(Value),
}

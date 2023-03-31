use sbor::*;

#[derive(Sbor, Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Minute,
}

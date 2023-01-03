use sbor::*;

#[derive(Encode, Decode, TypeId, Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Minute,
}

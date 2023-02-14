use sbor::*;

#[derive(Encode, Decode, Categorize, Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Minute,
}

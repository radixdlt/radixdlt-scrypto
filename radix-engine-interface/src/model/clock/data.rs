use sbor::*;

#[derive(Encode, Decode, TypeId, Copy, Clone, Debug)]
pub enum TimePrecision {
    Minute,
}

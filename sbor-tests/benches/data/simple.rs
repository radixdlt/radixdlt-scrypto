use sbor::{Decode, Encode, TypeId};
use serde::{Deserialize, Serialize};

#[derive(TypeId, Encode, Decode, Serialize, Deserialize)]
pub enum SimpleEnum {
    Unit,
    Unamed(u32),
    Named { x: u32, y: u32 },
}

#[derive(TypeId, Encode, Decode, Serialize, Deserialize)]
pub struct SimpleStruct {
    pub number: u64,
    pub string: String,
    pub vector1: Vec<u8>,
    pub vector2: Vec<u16>,
    pub enumeration: SimpleEnum,
}

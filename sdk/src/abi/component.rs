extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentInput {
    pub method: String,
    pub args: Vec<SerializedValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentOutput {
    pub rtn: SerializedValue,
}

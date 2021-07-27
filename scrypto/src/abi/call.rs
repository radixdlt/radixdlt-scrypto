extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CallInput {
    pub method: String,
    pub args: Vec<SerializedValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallOutput {
    pub rtn: SerializedValue,
}

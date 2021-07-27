extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CallInput {
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallOutput {
    pub rtn: Vec<u8>,
}

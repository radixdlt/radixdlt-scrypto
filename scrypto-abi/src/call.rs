extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use sbor::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct CallInput {
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Encode, Decode)]
pub struct CallOutput {
    pub rtn: Vec<u8>,
}

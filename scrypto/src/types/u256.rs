extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

use sbor::{Decode, Encode};
use uint::construct_uint;

construct_uint! {
    #[derive(Encode, Decode)]
    pub struct U256(8);
}

impl From<String> for U256 {
    fn from(s: String) -> Self {
        U256::from_dec_str(s.as_str()).unwrap()
    }
}

impl Into<String> for U256 {
    fn into(self) -> String {
        self.to_string()
    }
}

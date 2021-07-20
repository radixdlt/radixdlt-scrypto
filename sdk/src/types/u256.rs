extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

use serde::{Deserialize, Serialize};
use uint::construct_uint;

construct_uint! {
    #[derive(Serialize, Deserialize)]
    #[serde(from = "String")]
    #[serde(into = "String")]
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

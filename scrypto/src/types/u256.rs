use sbor::{Decode, Describe, Encode};
use uint::construct_uint;

construct_uint! {
    #[derive(Describe, Encode, Decode)]
    pub struct U256(8);
}

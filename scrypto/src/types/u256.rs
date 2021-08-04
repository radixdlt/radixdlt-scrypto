use sbor::{Decode, Encode};
use uint::construct_uint;

construct_uint! {
    #[derive(Encode, Decode)]
    pub struct U256(8);
}

use crate::*;

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    Hash,
)]
pub enum Role {
    Primary,
    Recovery,
    Confirmation,
}

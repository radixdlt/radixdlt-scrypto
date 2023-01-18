use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum Role {
    Primary,
    Recovery,
    Confirmation,
}

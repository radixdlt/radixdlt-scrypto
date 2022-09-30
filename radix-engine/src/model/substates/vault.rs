use crate::model::Resource;
use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VaultSubstate(pub Resource);

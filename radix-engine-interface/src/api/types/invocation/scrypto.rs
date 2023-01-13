use crate::api::types::*;
use crate::model::*;
use crate::scrypto;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub enum Receiver {
    Global(ComponentAddress),
    Component(ComponentId),
}

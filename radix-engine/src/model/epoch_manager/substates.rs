use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerSubstate {
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorSetSubstate {
    pub validator_set: Vec<EcdsaSecp256k1PublicKey>,
}

use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerSubstate {
    pub address: SystemAddress, // TODO: Does it make sense for this to be stored here?
    pub epoch: u64,
    pub round: u64,
    pub rounds_per_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorSetSubstate {
    pub validator_set: HashSet<EcdsaSecp256k1PublicKey>,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorSubstate {
    pub manager: SystemAddress,
    pub key: EcdsaSecp256k1PublicKey,
}

use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerSubstate {
    pub epoch: u64,
    pub round: u64,
    pub rounds_per_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ValidatorSetSubstate {
    pub validator_set: HashSet<EcdsaSecp256k1PublicKey>,
    pub epoch: u64,
}

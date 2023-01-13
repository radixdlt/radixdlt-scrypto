use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerSubstate {
    pub address: SystemAddress, // TODO: Does it make sense for this to be stored here?
    pub epoch: u64,
    pub round: u64,
    pub rounds_per_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct Validator {
    pub address: SystemAddress,
    pub key: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorSetSubstate {
    pub validator_set: BTreeSet<Validator>,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorSubstate {
    pub manager: SystemAddress,
    pub address: SystemAddress,
    pub key: EcdsaSecp256k1PublicKey,
}

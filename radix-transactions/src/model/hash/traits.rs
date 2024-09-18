use crate::internal_prelude::*;
use radix_common::prelude::*;

pub trait IsTransactionHashWithStaticHrp: IsHash {
    fn static_hrp<'h>(hrp_set: &'h HrpSet) -> &'h str;
}

pub enum HashCreationError {
    InvalidHrp,
}

pub trait IsTransactionHash: Sized {
    fn hrp<'h>(&self, hrp_set: &'h HrpSet) -> &'h str;
    fn create_from_hrp_and_hash(
        hrp: &str,
        hash: Hash,
        hrp_set: &HrpSet,
    ) -> Result<Self, HashCreationError>;
    fn as_inner_hash(&self) -> &Hash;
}

impl<H: IsTransactionHashWithStaticHrp> IsTransactionHash for H {
    fn hrp<'h>(&self, hrp_set: &'h HrpSet) -> &'h str {
        Self::static_hrp(hrp_set)
    }

    fn create_from_hrp_and_hash(
        hrp: &str,
        hash: Hash,
        hrp_set: &HrpSet,
    ) -> Result<Self, HashCreationError> {
        if Self::static_hrp(hrp_set) == hrp {
            Ok(Self::from(hash))
        } else {
            Err(HashCreationError::InvalidHrp)
        }
    }

    fn as_inner_hash(&self) -> &Hash {
        self.as_hash()
    }
}

impl IsTransactionHashWithStaticHrp for TransactionIntentHash {
    fn static_hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.transaction_intent
    }
}

impl IsTransactionHashWithStaticHrp for SignedTransactionIntentHash {
    fn static_hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.signed_transaction_intent
    }
}

impl IsTransactionHashWithStaticHrp for SubintentHash {
    fn static_hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.subintent
    }
}

impl IsTransactionHash for IntentHash {
    fn hrp<'h>(&self, hrp_set: &'h HrpSet) -> &'h str {
        match self {
            IntentHash::Transaction(_) => TransactionIntentHash::static_hrp(hrp_set),
            IntentHash::Subintent(_) => SubintentHash::static_hrp(hrp_set),
        }
    }

    fn create_from_hrp_and_hash(
        hrp: &str,
        hash: Hash,
        hrp_set: &HrpSet,
    ) -> Result<Self, HashCreationError> {
        if hrp == TransactionIntentHash::static_hrp(hrp_set) {
            Ok(IntentHash::Transaction(TransactionIntentHash::from(hash)))
        } else if hrp == SubintentHash::static_hrp(hrp_set) {
            Ok(IntentHash::Subintent(SubintentHash::from(hash)))
        } else {
            Err(HashCreationError::InvalidHrp)
        }
    }

    fn as_inner_hash(&self) -> &Hash {
        match self {
            IntentHash::Transaction(inner) => inner.as_hash(),
            IntentHash::Subintent(inner) => inner.as_hash(),
        }
    }
}

impl IsTransactionHashWithStaticHrp for NotarizedTransactionHash {
    fn static_hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.notarized_transaction
    }
}

impl IsTransactionHashWithStaticHrp for SystemTransactionHash {
    fn static_hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.system_transaction
    }
}

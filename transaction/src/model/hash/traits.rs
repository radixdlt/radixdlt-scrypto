use crate::prelude::*;
use radix_engine_common::prelude::*;

pub trait HashHasHrp
where
    Self: IsHash,
{
    fn hrp<'h>(hrp_set: &'h HrpSet) -> &'h str;
}

impl HashHasHrp for IntentHash {
    fn hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.transaction_intent
    }
}

impl HashHasHrp for SignedIntentHash {
    fn hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.signed_transaction_intent
    }
}

impl HashHasHrp for NotarizedTransactionHash {
    fn hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.notarized_transaction
    }
}

impl HashHasHrp for SystemTransactionHash {
    fn hrp<'h>(hrp_set: &'h HrpSet) -> &'h str {
        &hrp_set.system_transaction
    }
}

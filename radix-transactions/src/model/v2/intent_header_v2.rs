use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct IntentHeaderV2 {
    pub network_id: u8,

    // Nullifier stuff
    pub start_epoch_inclusive: Epoch,
    pub end_epoch_exclusive: Epoch,
    pub min_proposer_timestamp_inclusive: Option<Instant>,
    pub max_proposer_timestamp_exclusive: Option<Instant>,

    /// This field is intended to enable a network user to generate an identical
    /// intent with a new hash. Users can simply set this randomly if they wish to.
    /// A u64 is large enough to avoid any risk of collision over the course of a
    /// single epoch anyway.
    ///
    /// This field's name `intent_discriminator` is the new name for what was the
    /// `nonce` field in `IntentV1`. This was poorly named, as it caused confusion with an Ethereum-style nonce.
    pub intent_discriminator: u64,
}

impl TransactionPartialPrepare for IntentHeaderV2 {
    type Prepared = PreparedIntentHeaderV2;
}

pub type PreparedIntentHeaderV2 = SummarizedRawValueBody<IntentHeaderV2>;

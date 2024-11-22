// NOTE:
// This file is only included if #[cfg(test)] is present

use crate::internal_prelude::*;

pub(crate) fn unsigned_v1_builder(notary_public_key: PublicKey) -> TransactionV1Builder {
    TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(10),
            nonce: 0,
            notary_public_key,
            notary_is_signatory: false,
            tip_percentage: 5,
        })
        .manifest(ManifestBuilder::new().drop_auth_zone_proofs().build())
}

pub(crate) fn unsigned_v2_builder(notary_public_key: PublicKey) -> TransactionV2Builder {
    TransactionBuilder::new_v2()
        .transaction_header(TransactionHeaderV2 {
            notary_public_key,
            notary_is_signatory: false,
            tip_basis_points: 5,
        })
        .intent_header(IntentHeaderV2 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(10),
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
            intent_discriminator: 0,
        })
        .manifest(ManifestBuilder::new_v2().drop_auth_zone_proofs().build())
}

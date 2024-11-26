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

// All of these are only added when in #[cfg(test)]
impl TransactionV2Builder {
    pub fn testing_default_notary() -> Ed25519PrivateKey {
        Ed25519PrivateKey::from_u64(1337).unwrap()
    }

    pub fn testing_default_intent_header() -> IntentHeaderV2 {
        IntentHeaderV2 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::of(0),
            end_epoch_exclusive: Epoch::of(1),
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
            intent_discriminator: 0,
        }
    }

    pub fn testing_default_transaction_header() -> TransactionHeaderV2 {
        TransactionHeaderV2 {
            notary_public_key: Self::testing_default_notary().public_key().into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        }
    }

    pub fn new_with_test_defaults() -> Self {
        Self::new()
            .intent_header(Self::testing_default_intent_header())
            .transaction_header(Self::testing_default_transaction_header())
    }

    pub fn add_children<T: ResolvableSignedPartialTransaction>(
        mut self,
        children: impl IntoIterator<Item = T>,
    ) -> Self {
        for (i, child) in children.into_iter().enumerate() {
            let child_name = format!("child_{i}");
            self = self.add_signed_child(child_name, child);
        }
        self
    }

    pub fn add_trivial_manifest(self) -> Self {
        self.manifest(
            ManifestBuilder::new_v2()
                .drop_named_proofs()
                .build_no_validate(),
        )
    }

    /// It calls into each child once
    pub fn add_manifest_calling_each_child_once(self) -> Self {
        let child_names = self
            .child_partial_transactions
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        self.manifest_builder(|mut builder| {
            builder = builder.lock_fee_from_faucet();
            for child_name in child_names {
                builder = builder.yield_to_child(child_name, ());
            }
            builder
        })
    }

    pub fn transaction_header_mut(&mut self) -> &mut TransactionHeaderV2 {
        self.transaction_header.as_mut().expect(
            "Transaction Header should already have been set, e.g. via new_with_test_defaults()",
        )
    }

    pub fn notary_is_signatory(mut self, notary_is_signatory: bool) -> Self {
        self.transaction_header_mut().notary_is_signatory = notary_is_signatory;
        self
    }

    pub fn notary_public_key(mut self, notary_public_key: impl Into<PublicKey>) -> Self {
        self.transaction_header_mut().notary_public_key = notary_public_key.into();
        self
    }

    pub fn tip_basis_points(mut self, tip_basis_points: u32) -> Self {
        self.transaction_header_mut().tip_basis_points = tip_basis_points;
        self
    }

    pub fn intent_header_mut(&mut self) -> &mut IntentHeaderV2 {
        self.transaction_intent_header
            .as_mut()
            .expect("Intent Header should already have been set, e.g. via new_with_test_defaults()")
    }

    pub fn network_id(mut self, network_id: u8) -> Self {
        self.intent_header_mut().network_id = network_id;
        self
    }

    pub fn start_epoch_inclusive(mut self, start_epoch_inclusive: Epoch) -> Self {
        self.intent_header_mut().start_epoch_inclusive = start_epoch_inclusive;
        self
    }

    pub fn end_epoch_exclusive(mut self, end_epoch_exclusive: Epoch) -> Self {
        self.intent_header_mut().end_epoch_exclusive = end_epoch_exclusive;
        self
    }

    pub fn min_proposer_timestamp_inclusive(
        mut self,
        min_proposer_timestamp_inclusive: Option<Instant>,
    ) -> Self {
        self.intent_header_mut().min_proposer_timestamp_inclusive =
            min_proposer_timestamp_inclusive;
        self
    }

    pub fn max_proposer_timestamp_exclusive(
        mut self,
        max_proposer_timestamp_exclusive: Option<Instant>,
    ) -> Self {
        self.intent_header_mut().max_proposer_timestamp_exclusive =
            max_proposer_timestamp_exclusive;
        self
    }

    pub fn intent_discriminator(mut self, intent_discriminator: u64) -> Self {
        self.intent_header_mut().intent_discriminator = intent_discriminator;
        self
    }

    pub fn default_notarize(self) -> Self {
        self.notarize(&Self::testing_default_notary())
    }

    pub fn default_notarize_and_validate(
        self,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        self.default_notarize()
            .build_minimal_no_validate()
            .prepare_and_validate(&TransactionValidator::new_for_latest_simulator())
    }
}

// All of these are only added when in #[cfg(test)]
impl PartialTransactionV2Builder {
    pub fn new_with_test_defaults() -> Self {
        Self::new().intent_header(IntentHeaderV2 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::of(0),
            end_epoch_exclusive: Epoch::of(1),
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
            intent_discriminator: 0,
        })
    }

    pub fn add_children<T: ResolvableSignedPartialTransaction>(
        mut self,
        children: impl IntoIterator<Item = T>,
    ) -> Self {
        for (i, child) in children.into_iter().enumerate() {
            let child_name = format!("child_{i}");
            self = self.add_signed_child(child_name, child);
        }
        self
    }

    pub fn add_trivial_manifest(self) -> Self {
        self.manifest(
            ManifestBuilder::new_subintent_v2()
                .yield_to_parent(())
                .build_no_validate(),
        )
    }

    /// It calls into each child once
    pub fn add_manifest_calling_each_child_once(self) -> Self {
        let child_names = self
            .child_partial_transactions
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        self.manifest_builder(|mut builder| {
            for child_name in child_names {
                builder = builder.yield_to_child(child_name, ());
            }
            builder
        })
    }

    pub fn intent_header_mut(&mut self) -> &mut IntentHeaderV2 {
        self.root_subintent_header
            .as_mut()
            .expect("Intent Header should already have been set, e.g. via new_with_test_defaults()")
    }

    pub fn network_id(mut self, network_id: u8) -> Self {
        self.intent_header_mut().network_id = network_id;
        self
    }

    pub fn start_epoch_inclusive(mut self, start_epoch_inclusive: Epoch) -> Self {
        self.intent_header_mut().start_epoch_inclusive = start_epoch_inclusive;
        self
    }

    pub fn end_epoch_exclusive(mut self, end_epoch_exclusive: Epoch) -> Self {
        self.intent_header_mut().end_epoch_exclusive = end_epoch_exclusive;
        self
    }

    pub fn min_proposer_timestamp_inclusive(
        mut self,
        min_proposer_timestamp_inclusive: Option<Instant>,
    ) -> Self {
        self.intent_header_mut().min_proposer_timestamp_inclusive =
            min_proposer_timestamp_inclusive;
        self
    }

    pub fn max_proposer_timestamp_exclusive(
        mut self,
        max_proposer_timestamp_exclusive: Option<Instant>,
    ) -> Self {
        self.intent_header_mut().max_proposer_timestamp_exclusive =
            max_proposer_timestamp_exclusive;
        self
    }

    pub fn intent_discriminator(mut self, intent_discriminator: u64) -> Self {
        self.intent_header_mut().intent_discriminator = intent_discriminator;
        self
    }
}

pub(crate) fn create_leaf_partial_transaction(
    intent_discriminator: u64,
    num_signatures: usize,
) -> DetailedSignedPartialTransactionV2 {
    PartialTransactionV2Builder::new_with_test_defaults()
        .intent_discriminator(intent_discriminator)
        .add_trivial_manifest()
        .multi_sign((0..num_signatures).into_iter().map(|i| {
            Secp256k1PrivateKey::from_u64((intent_discriminator + 1) * 1000 + (i as u64)).unwrap()
        }))
        .build()
}

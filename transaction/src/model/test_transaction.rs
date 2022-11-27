use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::crypto::{EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature};
use radix_engine_interface::model::*;

use sbor::rust::vec::Vec;
use std::collections::BTreeSet;

use crate::builder::TransactionBuilder;
use crate::model::*;

pub struct TestTransaction {
    transaction: NotarizedTransaction,
}

impl TestTransaction {
    pub fn new(manifest: TransactionManifest, nonce: u64) -> Self {
        let transaction = TransactionBuilder::new()
            .header(TransactionHeader {
                version: TRANSACTION_VERSION_V1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce,
                notary_public_key: EcdsaSecp256k1PublicKey([0u8; 33]).into(),
                notary_as_signatory: false,
                cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
                tip_percentage: 5,
            })
            .manifest(manifest)
            .notary_signature(EcdsaSecp256k1Signature([0u8; 65]).into())
            .build();

        Self { transaction }
    }

    pub fn get_executable<'a>(&'a self, initial_proofs: Vec<NonFungibleAddress>) -> Executable<'a> {
        let transaction_hash = self.transaction.hash().unwrap();
        let intent = &self.transaction.signed_intent.intent;

        Executable::new(
            &intent.manifest.instructions,
            &intent.manifest.blobs,
            ExecutionContext {
                transaction_hash,
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtualizable_proofs_resource_addresses: BTreeSet::new(),
                },
                fee_payment: FeePayment {
                    cost_unit_limit: intent.header.cost_unit_limit,
                    tip_percentage: intent.header.tip_percentage,
                },
                runtime_validations: vec![],
            },
        )
    }
}

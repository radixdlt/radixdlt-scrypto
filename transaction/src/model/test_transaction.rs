use sbor::rust::vec::Vec;
use scrypto::core::NetworkDefinition;
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

use crate::builder::TransactionBuilder;
use crate::model::*;

pub struct TestTransaction {}

impl TestTransaction {
    pub fn new(
        manifest: TransactionManifest,
        nonce: u64,
        initial_proofs: Vec<NonFungibleAddress>,
    ) -> Validated {
        let transaction = TransactionBuilder::new()
            .header(TransactionHeader {
                version: TRANSACTION_VERSION_V1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce,
                notary_public_key: EcdsaSecp256k1PublicKey([0u8; 33]).into(),
                notary_as_signatory: false,
                cost_unit_limit: 10_000_000,
                tip_percentage: 5,
            })
            .manifest(manifest)
            .notary_signature(EcdsaSecp256k1Signature([0u8; 65]).into())
            .build();

        let transaction_hash = transaction.hash();

        Validated {
            transaction_hash,
            instructions: transaction.signed_intent.intent.manifest.instructions,
            initial_proofs,
            cost_unit_limit: transaction.signed_intent.intent.header.cost_unit_limit,
            tip_percentage: transaction.signed_intent.intent.header.cost_unit_limit,
            blobs: transaction.signed_intent.intent.manifest.blobs,
        }
    }
}

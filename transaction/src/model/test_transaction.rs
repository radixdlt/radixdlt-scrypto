use sbor::rust::vec::Vec;
use scrypto::buffer::scrypto_encode;
use scrypto::core::NetworkDefinition;
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

use crate::builder::TransactionBuilder;
use crate::model::*;

/// Represents a test transaction, for testing/simulation purpose only.
pub struct TestTransaction {
    pub transaction: NotarizedTransaction,
    pub initial_proofs: Vec<NonFungibleAddress>,
}

impl TestTransaction {
    pub fn new(
        manifest: TransactionManifest,
        nonce: u64,
        initial_proofs: Vec<NonFungibleAddress>,
    ) -> Self {
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

        Self {
            transaction,
            initial_proofs,
        }
    }
}

impl ExecutableTransaction for TestTransaction {
    fn transaction_hash(&self) -> Hash {
        self.transaction.hash()
    }

    fn manifest_instructions_size(&self) -> u32 {
        scrypto_encode(&self.transaction.signed_intent.intent.manifest.instructions).len() as u32
    }

    fn cost_unit_limit(&self) -> u32 {
        self.transaction.signed_intent.intent.header.cost_unit_limit
    }

    fn tip_percentage(&self) -> u32 {
        self.transaction.signed_intent.intent.header.tip_percentage
    }

    fn instructions(&self) -> &[Instruction] {
        &self.transaction.signed_intent.intent.manifest.instructions
    }

    fn initial_proofs(&self) -> Vec<NonFungibleAddress> {
        self.initial_proofs.clone()
    }

    fn blobs(&self) -> &[Vec<u8>] {
        &self.transaction.signed_intent.intent.manifest.blobs
    }
}

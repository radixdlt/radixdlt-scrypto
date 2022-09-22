use sbor::rust::vec::Vec;
use scrypto::buffer::scrypto_encode;
use scrypto::core::NetworkDefinition;
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

use crate::builder::TransactionBuilder;
use crate::model::*;

pub enum TestTransactionActor {
    User(Vec<PublicKey>),
    Superuser,
}

/// Represents a test transaction, for testing/simulation purpose only.
pub struct TestTransaction {
    pub transaction: NotarizedTransaction,
    pub actor: TestTransactionActor,
}

impl TestTransaction {
    pub fn new(manifest: TransactionManifest, nonce: u64, actor: TestTransactionActor) -> Self {
        let builder = TransactionBuilder::new()
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
            .manifest(manifest);
        let builder = match &actor {
            TestTransactionActor::User(signer_public_keys) => builder.signer_signatures(
                signer_public_keys
                    .iter()
                    .cloned()
                    .map(|pk| match pk {
                        PublicKey::EcdsaSecp256k1(_) => EcdsaSecp256k1Signature([0u8; 65]).into(),
                        PublicKey::EddsaEd25519(pk) => {
                            (pk, EddsaEd25519Signature([0u8; 64])).into()
                        }
                    })
                    .collect(),
            ),
            TestTransactionActor::Superuser => builder.as_supervisor(),
        };
        let transaction = builder
            .notary_signature(EcdsaSecp256k1Signature([0u8; 65]).into())
            .build();

        Self { transaction, actor }
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
        match &self.actor {
            TestTransactionActor::User(signer_public_keys) => {
                AuthModule::signer_keys_to_non_fungibles(signer_public_keys)
            }
            TestTransactionActor::Superuser => vec![AuthModule::supervisor_address()],
        }
    }

    fn blobs(&self) -> &[Vec<u8>] {
        &self.transaction.signed_intent.intent.manifest.blobs
    }
}

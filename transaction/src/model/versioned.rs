use super::v1::*;
use crate::internal_prelude::*;

/// Note - some of these are reserved for use in the node.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TransactionDiscriminator {
    V1Intent = V1_INTENT,
    V1SignedIntent = V1_SIGNED_INTENT,
    V1Notarized = V1_NOTARIZED_TRANSACTION,
    V1System = V1_SYSTEM_TRANSACTION,
    V1Consensus = V1_CONSENSUS_TRANSACTION,
    V1Preview = V1_PREVIEW_TRANSACTION,
    V1Ledger = V1_LEDGER_TRANSACTION,
}

const V1_INTENT: u8 = 1;
const V1_SIGNED_INTENT: u8 = 2;
const V1_NOTARIZED_TRANSACTION: u8 = 3;
const V1_SYSTEM_TRANSACTION: u8 = 4;
const V1_CONSENSUS_TRANSACTION: u8 = 5;
const V1_PREVIEW_TRANSACTION: u8 = 6;
const V1_LEDGER_TRANSACTION: u8 = 7;

pub trait TransactionPayloadEncode {
    type EncodablePayload<'a>: ManifestEncode + ManifestCategorize
    where
        Self: 'a;

    fn as_payload<'a>(&'a self) -> Self::EncodablePayload<'a>;

    fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&self.as_payload())
    }
}

// TODO - change this to use #[flatten] when REP-84 is out
/// An enum of a variety of different transaction payload types
/// This might see use in (eg) the Node's transaction parse API.
/// These represent the different transaction types.
#[derive(Clone, Debug, Eq, PartialEq, ManifestSbor)]
pub enum VersionedTransactionPayload {
    #[sbor(discriminator(V1_INTENT))]
    IntentV1 {
        header: TransactionHeaderV1,
        instructions: InstructionsV1,
        blobs: BlobsV1,
        attachments: AttachmentsV1,
    },
    #[sbor(discriminator(V1_SIGNED_INTENT))]
    SignedIntentV1 {
        intent: IntentV1,
        intent_signatures: IntentSignaturesV1,
    },
    #[sbor(discriminator(V1_NOTARIZED_TRANSACTION))]
    NotarizedTransactionV1 {
        signed_intent: SignedIntentV1,
        notary_signature: NotarySignatureV1,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::{ecdsa_secp256k1::EcdsaSecp256k1PrivateKey, eddsa_ed25519::EddsaEd25519PrivateKey};

    fn hash_manifest_encoded_without_prefix_byte<T: ManifestEncode>(value: T) -> Hash {
        hash(&manifest_encode(&value).unwrap()[1..])
    }

    #[test]
    pub fn v1_hashes_and_payloads_agree() {
        let network = NetworkDefinition::simulator();

        // Create key pairs
        let sig_1_private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
        let sig_2_private_key = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();
        let notary_private_key = EddsaEd25519PrivateKey::from_u64(3).unwrap();

        //===================
        // INTENT
        //===================
        let header_v1 = TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: 1,
            end_epoch_exclusive: 5,
            nonce: 0,
            notary_public_key: notary_private_key.public_key().into(),
            notary_is_signatory: false,
            cost_unit_limit: 123,
            tip_percentage: 0,
        };
        let expected_header_hash = hash_manifest_encoded_without_prefix_byte(&header_v1);

        let instructions = vec![Instruction::ClearAuthZone];
        let expected_instructions_hash = hash_manifest_encoded_without_prefix_byte(&instructions);
        let instructions_v1 = InstructionsV1(instructions);

        let blob1: Vec<u8> = vec![0, 1, 2, 3];
        let blob2: Vec<u8> = vec![5, 6];
        let expected_blobs_hash =
            hash([hash(&blob1).0.as_slice(), hash(&blob2).0.as_slice()].concat());

        let blobs_v1 = BlobsV1 {
            blobs: vec![BlobV1(blob1), BlobV1(blob2)],
        };

        let prepared_blobs_v1 = PreparedBlobsV1::prepare_as_full_body_child_from_payload(
            &manifest_encode(&blobs_v1).unwrap(),
        )
        .unwrap();
        assert_eq!(prepared_blobs_v1.get_summary().hash, expected_blobs_hash);

        let attachments_v1 = AttachmentsV1 {};
        let expected_attachments_hash = hash_manifest_encoded_without_prefix_byte(&attachments_v1);

        let intent_v1 = IntentV1 {
            header: header_v1.clone(),
            instructions: instructions_v1.clone(),
            blobs: blobs_v1.clone(),
            attachments: attachments_v1.clone(),
        };
        let expected_intent_hash = IntentHash::from_hash(hash(
            [
                [TransactionDiscriminator::V1Intent as u8].as_slice(),
                expected_header_hash.0.as_slice(),
                expected_instructions_hash.0.as_slice(),
                expected_blobs_hash.0.as_slice(),
                expected_attachments_hash.0.as_slice(),
            ]
            .concat(),
        ));

        let intent_payload_bytes = intent_v1.to_payload_bytes().unwrap();
        let intent_as_versioned =
            manifest_decode::<VersionedTransactionPayload>(&intent_payload_bytes).unwrap();
        assert_eq!(
            intent_as_versioned,
            VersionedTransactionPayload::IntentV1 {
                header: header_v1.clone(),
                instructions: instructions_v1.clone(),
                blobs: blobs_v1.clone(),
                attachments: attachments_v1.clone(),
            }
        );

        let prepared_intent =
            PreparedIntentV1::prepare_from_payload(&intent_payload_bytes).unwrap();
        assert_eq!(expected_intent_hash, prepared_intent.intent_hash());

        let intent_hash = prepared_intent.intent_hash();

        //===================
        // SIGNED INTENT
        //===================
        let sig1 = sig_1_private_key.sign_with_public_key(&intent_hash);
        let sig2 = sig_2_private_key.sign_with_public_key(&intent_hash);

        let intent_signatures_v1 = IntentSignaturesV1 {
            signatures: vec![IntentSignatureV1(sig1), IntentSignatureV1(sig2)],
        };
        let expected_intent_signatures_hash =
            hash_manifest_encoded_without_prefix_byte(&intent_signatures_v1);

        let signed_intent_v1 = SignedIntentV1 {
            intent: intent_v1.clone(),
            intent_signatures: intent_signatures_v1.clone(),
        };
        let expected_signed_intent_hash = SignedIntentHash::from_hash(hash(
            [
                [TransactionDiscriminator::V1SignedIntent as u8].as_slice(),
                intent_hash.0.as_slice(),
                expected_intent_signatures_hash.0.as_slice(),
            ]
            .concat(),
        ));

        let signed_intent_payload_bytes = signed_intent_v1.to_payload_bytes().unwrap();
        let signed_intent_as_versioned =
            manifest_decode::<VersionedTransactionPayload>(&signed_intent_payload_bytes).unwrap();
        assert_eq!(
            signed_intent_as_versioned,
            VersionedTransactionPayload::SignedIntentV1 {
                intent: intent_v1,
                intent_signatures: intent_signatures_v1,
            }
        );

        let prepared_signed_intent =
            PreparedSignedIntentV1::prepare_from_payload(&signed_intent_payload_bytes).unwrap();
        assert_eq!(
            expected_signed_intent_hash,
            prepared_signed_intent.signed_intent_hash()
        );
        assert_eq!(intent_hash, prepared_signed_intent.intent_hash());

        let signed_intent_hash = expected_signed_intent_hash;

        //======================
        // NOTARIZED TRANSACTION
        //======================
        let notary_signature = notary_private_key.sign(&signed_intent_hash);

        let notary_signature_v1 = NotarySignatureV1(notary_signature.into());
        let expected_notary_signature_v1_hash =
            hash_manifest_encoded_without_prefix_byte(&notary_signature_v1);

        let notarized_transaction_v1 = NotarizedTransactionV1 {
            signed_intent: signed_intent_v1.clone(),
            notary_signature: notary_signature_v1.clone(),
        };
        let expected_notarized_intent_hash = NotarizedTransactionHash::from_hash(hash(
            [
                [TransactionDiscriminator::V1Notarized as u8].as_slice(),
                signed_intent_hash.0.as_slice(),
                expected_notary_signature_v1_hash.0.as_slice(),
            ]
            .concat(),
        ));

        let notarized_transaction_payload_bytes =
            notarized_transaction_v1.to_payload_bytes().unwrap();
        let notarized_transaction_as_versioned =
            manifest_decode::<VersionedTransactionPayload>(&notarized_transaction_payload_bytes)
                .unwrap();
        assert_eq!(
            notarized_transaction_as_versioned,
            VersionedTransactionPayload::NotarizedTransactionV1 {
                signed_intent: signed_intent_v1.clone(),
                notary_signature: notary_signature_v1.clone(),
            }
        );

        let prepared_notarized_transaction = PreparedNotarizedTransactionV1::prepare_from_payload(
            &notarized_transaction_payload_bytes,
        )
        .unwrap();
        assert_eq!(
            expected_notarized_intent_hash,
            prepared_notarized_transaction.notarized_transaction_hash()
        );
        assert_eq!(
            signed_intent_hash,
            prepared_notarized_transaction.signed_intent_hash()
        );
        assert_eq!(intent_hash, prepared_notarized_transaction.intent_hash());
    }
}

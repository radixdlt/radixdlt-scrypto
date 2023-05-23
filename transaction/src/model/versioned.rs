use super::v1::*;
use crate::internal_prelude::*;

//=============================================================================
// TRANSACTION PAYLOAD VERSIONING
//
// This file aligns with REP-82 - please see the REP for details on why the
// payloads are versioned this way.
//=============================================================================

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
    type Prepared: TransactionPayloadPreparable;

    fn as_payload<'a>(&'a self) -> Self::EncodablePayload<'a>;

    fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&self.as_payload())
    }

    fn prepare(&self) -> Result<Self::Prepared, ConvertToPreparedError> {
        Ok(Self::Prepared::prepare_from_payload(
            &self.to_payload_bytes()?,
        )?)
    }
}

pub trait TransactionPartialEncode: ManifestEncode {
    type Prepared: TransactionFullChildPreparable;

    fn prepare_partial(&self) -> Result<Self::Prepared, ConvertToPreparedError> {
        Ok(Self::Prepared::prepare_as_full_body_child_from_payload(
            &manifest_encode(self)?,
        )?)
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
    #[sbor(discriminator(V1_SYSTEM_TRANSACTION))]
    SystemTransactionV1 {
        instructions: InstructionsV1,
        blobs: BlobsV1,
        pre_allocated_ids: BTreeSet<NodeId>,
        hash_for_execution: Hash,
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

    /// This test demonstrates how the hashes and payloads are constructed in a valid user transaction.
    /// It also provides an example payload which can be used in other implementations.
    #[test]
    pub fn v1_user_transaction_structure() {
        let network = NetworkDefinition::simulator();

        // Create key pairs
        let sig_1_private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
        let sig_2_private_key = EddsaEd25519PrivateKey::from_u64(2).unwrap();
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
            tip_percentage: 0,
        };
        let expected_header_hash = hash_manifest_encoded_without_prefix_byte(&header_v1);

        let instructions = vec![InstructionV1::ClearAuthZone];
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
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1Intent as u8,
                ]
                .as_slice(),
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

        assert_eq!(
            intent_hash.to_string(),
            "6ec1aad83ad0796bf7ae2d01ca6981c166983e89dc5e7a4158abac892a191cc3"
        );
        assert_eq!(
            hex::encode(intent_payload_bytes),
            "4d220104210707f2090100000009050000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000800002022011200202002070400010203070205062100"
        );

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
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1SignedIntent as u8,
                ]
                .as_slice(),
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

        assert_eq!(
            signed_intent_hash.to_string(),
            "05840aa1b54235bf9fae522578497f554d202ab7019813ea97009d5f96f51291"
        );
        assert_eq!(
            hex::encode(signed_intent_payload_bytes),
            "4d2202022104210707f2090100000009050000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b010008000020220112002020020704000102030702050621002022020001210120074100821f54ad0e11d08e46debae1167d4181f6343ba5a431a45dd3b66a9e9c873ffa22870316f53cd1aed1b6ea4d29f8424d185b5d2d6202e73fdd8b183eec5ef9db01022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe267421012007402506c9295fa7210554d16bfdb4b0f22ab979c0148c505431d634a4aa9acc7758e25bcd8914773599cb70ffbc59d40a567b54715f63656f2d017ea8bf371bf500"
        );

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
        let expected_notarized_transaction_hash = NotarizedTransactionHash::from_hash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1Notarized as u8,
                ]
                .as_slice(),
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
            expected_notarized_transaction_hash,
            prepared_notarized_transaction.notarized_transaction_hash()
        );
        let notarized_transaction_hash = expected_notarized_transaction_hash;
        assert_eq!(
            signed_intent_hash,
            prepared_notarized_transaction.signed_intent_hash()
        );
        assert_eq!(intent_hash, prepared_notarized_transaction.intent_hash());

        assert_eq!(
            notarized_transaction_hash.to_string(),
            "2a384e0817104e4dd3e566339cccd445c400459e0779dba60a3dd9a9f8564082"
        );
        assert_eq!(
            hex::encode(notarized_transaction_payload_bytes),
            "4d22030221022104210707f2090100000009050000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b010008000020220112002020020704000102030702050621002022020001210120074100821f54ad0e11d08e46debae1167d4181f6343ba5a431a45dd3b66a9e9c873ffa22870316f53cd1aed1b6ea4d29f8424d185b5d2d6202e73fdd8b183eec5ef9db01022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe267421012007402506c9295fa7210554d16bfdb4b0f22ab979c0148c505431d634a4aa9acc7758e25bcd8914773599cb70ffbc59d40a567b54715f63656f2d017ea8bf371bf5002201012101200740eb907630079ab07da8598742cc3ed1c3831f02a3e8b5985473a441bdea17ac7e983584410f4c67ffe30ea840cc88f7948fa053346fe6cf0cf1af70373a848608"
        );
    }

    /// This test demonstrates how the hashes and payloads are constructed in a valid system transaction.
    /// A system transaction can be embedded into the node's LedgerTransaction structure, eg as part of Genesis
    #[test]
    pub fn v1_system_transaction_structure() {
        let instructions = vec![InstructionV1::ClearAuthZone];
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

        let pre_allocated_ids_v1 = btreeset![XRD.into_node_id()];
        let expected_preallocated_ids_hash =
            hash_manifest_encoded_without_prefix_byte(&pre_allocated_ids_v1);

        let hash_for_execution = hash(format!("Pretend genesis transaction"));

        let system_transaction_v1 = SystemTransactionV1 {
            instructions: instructions_v1.clone(),
            blobs: blobs_v1.clone(),
            pre_allocated_ids: pre_allocated_ids_v1.clone(),
            hash_for_execution: hash_for_execution.clone(),
        };
        let expected_system_transaction_hash = SystemTransactionHash::from_hash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1System as u8,
                ]
                .as_slice(),
                expected_instructions_hash.0.as_slice(),
                expected_blobs_hash.0.as_slice(),
                expected_preallocated_ids_hash.0.as_slice(),
                hash_for_execution.0.as_slice(),
            ]
            .concat(),
        ));

        let system_transaction_payload_bytes = system_transaction_v1.to_payload_bytes().unwrap();
        let system_transaction_as_versioned =
            manifest_decode::<VersionedTransactionPayload>(&system_transaction_payload_bytes)
                .unwrap();
        assert_eq!(
            system_transaction_as_versioned,
            VersionedTransactionPayload::SystemTransactionV1 {
                instructions: instructions_v1,
                blobs: blobs_v1,
                pre_allocated_ids: pre_allocated_ids_v1,
                hash_for_execution
            }
        );

        let prepared_system_transaction =
            PreparedSystemTransactionV1::prepare_from_payload(&system_transaction_payload_bytes)
                .unwrap();

        assert_eq!(
            expected_system_transaction_hash,
            prepared_system_transaction.system_transaction_hash()
        );
        assert_eq!(
            expected_system_transaction_hash.to_string(),
            "bab4dc066c63b9dfe3f88a295ce1b35873a98016f0a7148e3831580b322f62f7"
        );
        assert_eq!(
            hex::encode(system_transaction_payload_bytes),
            "4d220404202201120020200207040001020307020506202001071e5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c62007207646fcb3e6a2dbf0fd4830933c54928d3e8dafaf9f704afdae56336fc67aae0d"
        );
    }
}

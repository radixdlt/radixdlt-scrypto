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
    V1RoundUpdate = V1_ROUND_UPDATE_TRANSACTION,
    V1Preview = V1_PREVIEW_TRANSACTION,
    V1Ledger = V1_LEDGER_TRANSACTION,
}

const V1_INTENT: u8 = 1;
const V1_SIGNED_INTENT: u8 = 2;
const V1_NOTARIZED_TRANSACTION: u8 = 3;
const V1_SYSTEM_TRANSACTION: u8 = 4;
const V1_ROUND_UPDATE_TRANSACTION: u8 = 5;
const V1_PREVIEW_TRANSACTION: u8 = 6;
const V1_LEDGER_TRANSACTION: u8 = 7;

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
        message: MessageV1,
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
        pre_allocated_addresses: Vec<PreAllocatedAddress>,
        hash_for_execution: Hash,
    },
}

#[cfg(test)]
mod tests {
    use radix_engine_interface::blueprints::resource::FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT;

    use super::*;
    use crate::manifest::e2e::tests::print_blob;
    use crate::model::*;
    use crate::{signing::ed25519::Ed25519PrivateKey, signing::secp256k1::Secp256k1PrivateKey};

    fn hash_manifest_encoded_without_prefix_byte<T: ManifestEncode>(value: T) -> Hash {
        hash(&manifest_encode(&value).unwrap()[1..])
    }

    /// This test demonstrates how the hashes and payloads are constructed in a valid user transaction.
    /// It also provides an example payload which can be used in other implementations.
    #[test]
    pub fn v1_user_transaction_structure() {
        let network = NetworkDefinition::simulator();

        // Create key pairs
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let sig_2_private_key = Ed25519PrivateKey::from_u64(2).unwrap();
        let notary_private_key = Ed25519PrivateKey::from_u64(3).unwrap();

        //===================
        // INTENT
        //===================
        let header_v1 = TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(5),
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

        let message_v1 = MessageV1::default();
        let expected_attachments_hash = hash_manifest_encoded_without_prefix_byte(&message_v1);

        let intent_v1 = IntentV1 {
            header: header_v1.clone(),
            instructions: instructions_v1.clone(),
            blobs: blobs_v1.clone(),
            message: message_v1.clone(),
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

        println!();
        print_blob("HC_INTENT", intent_payload_bytes.clone());
        print_blob("HC_INTENT_HASH", expected_intent_hash.0.to_vec());

        IntentV1::from_payload_bytes(&intent_payload_bytes).expect("Intent can be decoded");
        let intent_as_versioned =
            manifest_decode::<VersionedTransactionPayload>(&intent_payload_bytes).unwrap();
        assert_eq!(
            intent_as_versioned,
            VersionedTransactionPayload::IntentV1 {
                header: header_v1.clone(),
                instructions: instructions_v1.clone(),
                blobs: blobs_v1.clone(),
                message: message_v1.clone(),
            }
        );

        let prepared_intent =
            PreparedIntentV1::prepare_from_payload(&intent_payload_bytes).unwrap();
        assert_eq!(expected_intent_hash, prepared_intent.intent_hash());

        let intent_hash = prepared_intent.intent_hash();

        assert_eq!(
            intent_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "txid_sim16hm8cq74dyusrgy8xg6eg5ss0d3cte9hdj0dhudtzp6vvszh3vjq3amttp"
        );
        assert_eq!(
            hex::encode(intent_payload_bytes),
            "4d220104210707f20a01000000000000000a05000000000000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b0100080000202201120020200207040001020307020506220000"
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
        SignedIntentV1::from_payload_bytes(&signed_intent_payload_bytes)
            .expect("SignedIntent can be decoded");
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
            signed_intent_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "signedintent_sim1dylyaqctdlpnr8768ve6gy6mhjryd5w46scepdx50nplyk64g28qcy3zxn"
        );
        assert_eq!(
            hex::encode(signed_intent_payload_bytes),
            "4d2202022104210707f20a01000000000000000a05000000000000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000800002022011200202002070400010203070205062200002022020001210120074100ffb4d3532977ad5f561d73ee8febbf4330812bb43063fd61a15e59ad233a13ea2f27b8eda06af0861b18108e4dae6301363b5b243ac1518f482e27f2f32f0bb701022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe26742101200740f0587aa712a637c84b0b2bc929c14cb2ccb3846c330434459205a11be5ff610cadfdbf33fa12b98d8e947f33a350a84068e710672753cdc33315c400db9c4e0f"
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
        NotarizedTransactionV1::from_payload_bytes(&notarized_transaction_payload_bytes)
            .expect("NotarizedTransaction can be decoded");
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
            notarized_transaction_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "notarizedtransaction_sim1lhfnzp027gt7ducszxmkl02qpp5lpx25npqwxkrk2qqyhs08raksacmd94"
        );
        assert_eq!(
            hex::encode(notarized_transaction_payload_bytes),
            "4d22030221022104210707f20a01000000000000000a05000000000000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000800002022011200202002070400010203070205062200002022020001210120074100ffb4d3532977ad5f561d73ee8febbf4330812bb43063fd61a15e59ad233a13ea2f27b8eda06af0861b18108e4dae6301363b5b243ac1518f482e27f2f32f0bb701022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe26742101200740f0587aa712a637c84b0b2bc929c14cb2ccb3846c330434459205a11be5ff610cadfdbf33fa12b98d8e947f33a350a84068e710672753cdc33315c400db9c4e0f2201012101200740321bfd17cac75d0b16fe6fd5aa9bb3e2beaf6521af4607f28815c8bd08718de8078a3fd75750354c400e1ea33cc8986853af6115bc43530cc0550ec9b2696a06"
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

        let pre_allocated_addresses_v1 = vec![PreAllocatedAddress {
            blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            address: XRD.into(),
        }];
        let expected_preallocated_addresses_hash =
            hash_manifest_encoded_without_prefix_byte(&pre_allocated_addresses_v1);

        let hash_for_execution = hash(format!("Pretend genesis transaction"));

        let system_transaction_v1 = SystemTransactionV1 {
            instructions: instructions_v1.clone(),
            blobs: blobs_v1.clone(),
            pre_allocated_addresses: pre_allocated_addresses_v1.clone(),
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
                expected_preallocated_addresses_hash.0.as_slice(),
                hash_for_execution.0.as_slice(),
            ]
            .concat(),
        ));

        let system_transaction_payload_bytes = system_transaction_v1.to_payload_bytes().unwrap();
        SystemTransactionV1::from_payload_bytes(&system_transaction_payload_bytes)
            .expect("SystemTransaction can be decoded");
        let system_transaction_as_versioned =
            manifest_decode::<VersionedTransactionPayload>(&system_transaction_payload_bytes)
                .unwrap();
        assert_eq!(
            system_transaction_as_versioned,
            VersionedTransactionPayload::SystemTransactionV1 {
                instructions: instructions_v1,
                blobs: blobs_v1,
                pre_allocated_addresses: pre_allocated_addresses_v1,
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
            expected_system_transaction_hash
                .to_string(&TransactionHashBech32Encoder::for_simulator()),
            "systemtransaction_sim14yf4hrcuqw9y8xrje8jr7h8y3fwnsg9y6nts2f5ru6r8s3yvgguq2da744"
        );
        assert_eq!(
            hex::encode(system_transaction_payload_bytes),
            "4d22040420220112002020020704000102030702050620210102210280000d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c60c1746756e6769626c655265736f757263654d616e6167657280005da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c62007207646fcb3e6a2dbf0fd4830933c54928d3e8dafaf9f704afdae56336fc67aae0d"
        );
    }
}

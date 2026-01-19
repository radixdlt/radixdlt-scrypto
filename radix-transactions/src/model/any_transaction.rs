use crate::internal_prelude::*;

//=============================================================================
// TRANSACTION PAYLOAD VERSIONING
//
// This file aligns with REP-82 - please see the REP for details on why the
// payloads are versioned this way.
//=============================================================================

/// Note - some of these are reserved for use in the node.
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromRepr)]
#[repr(u8)]
pub enum TransactionDiscriminator {
    V1Intent = V1_INTENT,
    V1SignedIntent = V1_SIGNED_INTENT,
    V1Notarized = V1_NOTARIZED_TRANSACTION,
    V1System = V1_SYSTEM_TRANSACTION,
    V1RoundUpdate = V1_ROUND_UPDATE_TRANSACTION,
    Ledger = LEDGER_TRANSACTION,
    V1Flash = V1_FLASH_TRANSACTION,
    V2TransactionIntent = V2_TRANSACTION_INTENT,
    V2SignedTransactionIntent = V2_SIGNED_TRANSACTION_INTENT,
    V2Subintent = V2_SUBINTENT,
    V2Notarized = V2_NOTARIZED_TRANSACTION,
    V2PartialTransaction = V2_PARTIAL_TRANSACTION,
    V2SignedPartialTransaction = V2_SIGNED_PARTIAL_TRANSACTION,
    V2PreviewTransaction = V2_PREVIEW_TRANSACTION,
}

const V1_INTENT: u8 = 1;
const V1_SIGNED_INTENT: u8 = 2;
const V1_NOTARIZED_TRANSACTION: u8 = 3;
const V1_SYSTEM_TRANSACTION: u8 = 4;
const V1_ROUND_UPDATE_TRANSACTION: u8 = 5;
// NOTE: 6 used to be reserved for serialized preview transactions,
//       but they have never been serialized, so 6 is free for re-use

// LEDGER TRANSACTION is not versioned, and can be extended with support
// for new versions
const LEDGER_TRANSACTION: u8 = 7;
const V1_FLASH_TRANSACTION: u8 = 8;
const V2_TRANSACTION_INTENT: u8 = 9;
const V2_SIGNED_TRANSACTION_INTENT: u8 = 10;
const V2_SUBINTENT: u8 = 11;
const V2_NOTARIZED_TRANSACTION: u8 = 12;
const V2_PARTIAL_TRANSACTION: u8 = 13;
const V2_SIGNED_PARTIAL_TRANSACTION: u8 = 14;
const V2_PREVIEW_TRANSACTION: u8 = 15;

/// An enum of a variety of different transaction payload types.
///
/// Running `to_payload_bytes()` on each transaction type gives the same
/// as Manifest SBOR encoding the variant of this enum.
///
/// For this reason, this type might see use in the Node's transaction
/// parse API, and in other places where we want to decode or handle an
/// arbitrary transaction payload.
///
/// All the transaction types also implement `ScryptoDescribe`, primarily
/// so that they can derive `ScryptoSborAssertion` to ensure we don't change
/// the types accidentally.
#[derive(Clone, Debug, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor(impl_variant_traits)]
#[sbor_assert(
    // This sum type of all payload-convertible transactions is extensible, so
    // we use `backwards_compatible` here. But most individual transaction models
    // should themselves be `fixed`, e.g. NotarizedTransactionV1
    backwards_compatible(
        bottlenose = "FILE:any_transaction_payload_schema_bottlenose.txt",
        cuttlefish = "FILE:any_transaction_payload_schema_cuttlefish.bin"
    ),
    settings(allow_name_changes)
)]
pub enum AnyTransaction {
    #[sbor(discriminator(V1_INTENT))]
    TransactionIntentV1(#[sbor(flatten)] IntentV1),
    #[sbor(discriminator(V1_SIGNED_INTENT))]
    SignedTransactionIntentV1(#[sbor(flatten)] SignedIntentV1),
    #[sbor(discriminator(V1_NOTARIZED_TRANSACTION))]
    NotarizedTransactionV1(#[sbor(flatten)] NotarizedTransactionV1),
    #[sbor(discriminator(V1_SYSTEM_TRANSACTION))]
    SystemTransactionV1(#[sbor(flatten)] SystemTransactionV1),
    #[sbor(discriminator(V1_ROUND_UPDATE_TRANSACTION))]
    RoundUpdateTransactionV1(#[sbor(flatten)] RoundUpdateTransactionV1),
    #[sbor(discriminator(LEDGER_TRANSACTION))] // Not flattened because it's an enum
    LedgerTransaction(LedgerTransaction),
    #[sbor(discriminator(V1_FLASH_TRANSACTION))]
    FlashTransactionV1(#[sbor(flatten)] FlashTransactionV1),
    #[sbor(discriminator(V2_TRANSACTION_INTENT))]
    TransactionIntentV2(#[sbor(flatten)] TransactionIntentV2),
    #[sbor(discriminator(V2_SIGNED_TRANSACTION_INTENT))]
    SignedTransactionIntentV2(#[sbor(flatten)] SignedTransactionIntentV2),
    #[sbor(discriminator(V2_SUBINTENT))]
    SubintentV2(#[sbor(flatten)] SubintentV2),
    #[sbor(discriminator(V2_NOTARIZED_TRANSACTION))]
    NotarizedTransactionV2(#[sbor(flatten)] NotarizedTransactionV2),
    #[sbor(discriminator(V2_PARTIAL_TRANSACTION))]
    PartialTransactionV2(#[sbor(flatten)] PartialTransactionV2),
    #[sbor(discriminator(V2_SIGNED_PARTIAL_TRANSACTION))]
    SignedPartialTransactionV2(#[sbor(flatten)] SignedPartialTransactionV2),
    #[sbor(discriminator(V2_PREVIEW_TRANSACTION))]
    PreviewTransactionV2(#[sbor(flatten)] PreviewTransactionV2),
}

#[cfg(test)]
mod tests {
    use radix_engine_interface::blueprints::resource::FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT;

    use super::*;
    use crate::manifest::e2e::tests::print_blob;
    use crate::model::*;

    #[deprecated = "Should only be used by transaction v1, because it's less flexible than hash_encoded_sbor_value_body"]
    fn hash_encoded_sbor_value<T: ManifestEncode>(value: T) -> Hash {
        // Ignore the version byte
        hash(&manifest_encode(&value).unwrap()[1..])
    }

    fn hash_encoded_sbor_value_body<T: ManifestEncode>(value: T) -> Hash {
        // Ignore the version byte AND the value kind
        hash(&manifest_encode(&value).unwrap()[2..])
    }

    fn hash_contatenated_hashes<H: Into<Hash>>(hashes: impl IntoIterator<Item = H>) -> Hash {
        let concatenated_hashes: Vec<u8> = hashes
            .into_iter()
            .flat_map(|h| Into::<Hash>::into(h).0)
            .collect();
        hash(concatenated_hashes)
    }

    fn hash_from_partial_prepare(value: &impl TransactionPartialPrepare) -> Hash {
        value
            .prepare_partial(PreparationSettings::latest_ref())
            .unwrap()
            .get_summary()
            .hash
    }

    /// This test demonstrates how the hashes and payloads are constructed in a valid user transaction.
    /// It also provides an example payload which can be used in other implementations.
    #[test]
    #[allow(deprecated)] // Transaction V1 is allowed to use deprecated hashing
    pub fn v1_user_transaction_structure() {
        let network = NetworkDefinition::simulator();
        let preparation_settings = PreparationSettings::babylon();

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
        let expected_header_hash = hash_encoded_sbor_value(&header_v1);

        let instructions = vec![InstructionV1::DropAuthZoneProofs(DropAuthZoneProofs)];
        let expected_instructions_hash = hash_encoded_sbor_value(&instructions);
        let instructions_v1 = InstructionsV1(instructions);

        let blob1: Vec<u8> = vec![0, 1, 2, 3];
        let blob2: Vec<u8> = vec![5, 6];
        let expected_blobs_hash =
            hash([hash(&blob1).0.as_slice(), hash(&blob2).0.as_slice()].concat());

        let blobs_v1 = BlobsV1 {
            blobs: vec![BlobV1(blob1), BlobV1(blob2)],
        };

        let prepared_blobs_v1 = blobs_v1.prepare_partial(&preparation_settings).unwrap();
        assert_eq!(prepared_blobs_v1.get_summary().hash, expected_blobs_hash);

        let message_v1 = MessageV1::default();
        let expected_attachments_hash = hash_encoded_sbor_value(&message_v1);

        let intent_v1 = IntentV1 {
            header: header_v1.clone(),
            instructions: instructions_v1.clone(),
            blobs: blobs_v1.clone(),
            message: message_v1.clone(),
        };
        let expected_intent_hash = TransactionIntentHash::from_hash(hash(
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

        let raw_intent_payload = intent_v1.to_raw().unwrap();

        println!();
        print_blob("HC_INTENT", raw_intent_payload.as_slice());
        print_blob("HC_INTENT_HASH", expected_intent_hash.0.as_slice());

        IntentV1::from_raw(&raw_intent_payload).expect("Intent can be decoded");
        let intent_as_versioned =
            manifest_decode::<AnyTransaction>(raw_intent_payload.as_slice()).unwrap();
        assert_eq!(
            intent_as_versioned,
            AnyTransaction::TransactionIntentV1(intent_v1.clone())
        );

        let prepared_intent =
            PreparedIntentV1::prepare(&raw_intent_payload, &preparation_settings).unwrap();
        assert_eq!(
            expected_intent_hash,
            prepared_intent.transaction_intent_hash()
        );

        let intent_hash = prepared_intent.transaction_intent_hash();

        assert_eq!(
            intent_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "txid_sim16hm8cq74dyusrgy8xg6eg5ss0d3cte9hdj0dhudtzp6vvszh3vjq3amttp"
        );
        assert_eq!(
            hex::encode(raw_intent_payload),
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
        let expected_intent_signatures_hash = hash_encoded_sbor_value(&intent_signatures_v1);

        let signed_intent_v1 = SignedIntentV1 {
            intent: intent_v1.clone(),
            intent_signatures: intent_signatures_v1.clone(),
        };
        let expected_signed_intent_hash = SignedTransactionIntentHash::from_hash(hash(
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

        let raw_signed_intent = signed_intent_v1.to_raw().unwrap();

        let signed_intent_as_versioned =
            manifest_decode::<AnyTransaction>(raw_signed_intent.as_slice()).unwrap();
        assert_eq!(
            signed_intent_as_versioned,
            AnyTransaction::SignedTransactionIntentV1(signed_intent_v1.clone())
        );

        let prepared_signed_intent =
            PreparedSignedIntentV1::prepare(&raw_signed_intent, &preparation_settings).unwrap();
        assert_eq!(
            expected_signed_intent_hash,
            prepared_signed_intent.signed_transaction_intent_hash()
        );
        assert_eq!(
            intent_hash,
            prepared_signed_intent.transaction_intent_hash()
        );

        let signed_intent_hash = expected_signed_intent_hash;

        assert_eq!(
            signed_intent_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "signedintent_sim1dylyaqctdlpnr8768ve6gy6mhjryd5w46scepdx50nplyk64g28qcy3zxn"
        );
        assert_eq!(
            hex::encode(raw_signed_intent),
            "4d2202022104210707f20a01000000000000000a05000000000000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000800002022011200202002070400010203070205062200002022020001210120074100ffb4d3532977ad5f561d73ee8febbf4330812bb43063fd61a15e59ad233a13ea2f27b8eda06af0861b18108e4dae6301363b5b243ac1518f482e27f2f32f0bb701022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe26742101200740f0587aa712a637c84b0b2bc929c14cb2ccb3846c330434459205a11be5ff610cadfdbf33fa12b98d8e947f33a350a84068e710672753cdc33315c400db9c4e0f"
        );

        //======================
        // NOTARIZED TRANSACTION
        //======================
        let notary_signature = notary_private_key.sign(signed_intent_hash);

        let notary_signature_v1 = NotarySignatureV1(notary_signature.into());
        let expected_notary_signature_v1_hash = hash_encoded_sbor_value(&notary_signature_v1);

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

        let raw_notarized_transaction = notarized_transaction_v1.to_raw().unwrap();
        NotarizedTransactionV1::from_raw(&raw_notarized_transaction)
            .expect("NotarizedTransaction can be decoded");
        let notarized_transaction_as_versioned =
            manifest_decode::<AnyTransaction>(raw_notarized_transaction.as_slice()).unwrap();
        assert_eq!(
            notarized_transaction_as_versioned,
            AnyTransaction::NotarizedTransactionV1(notarized_transaction_v1)
        );

        let prepared_notarized_transaction = PreparedNotarizedTransactionV1::prepare(
            &raw_notarized_transaction,
            &preparation_settings,
        )
        .unwrap();
        assert_eq!(
            expected_notarized_transaction_hash,
            prepared_notarized_transaction.notarized_transaction_hash()
        );
        let notarized_transaction_hash = expected_notarized_transaction_hash;
        assert_eq!(
            signed_intent_hash,
            prepared_notarized_transaction.signed_transaction_intent_hash()
        );
        assert_eq!(
            intent_hash,
            prepared_notarized_transaction.transaction_intent_hash()
        );

        assert_eq!(
            notarized_transaction_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "notarizedtransaction_sim1lhfnzp027gt7ducszxmkl02qpp5lpx25npqwxkrk2qqyhs08raksacmd94"
        );
        assert_eq!(
            hex::encode(raw_notarized_transaction),
            "4d22030221022104210707f20a01000000000000000a05000000000000000900000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000800002022011200202002070400010203070205062200002022020001210120074100ffb4d3532977ad5f561d73ee8febbf4330812bb43063fd61a15e59ad233a13ea2f27b8eda06af0861b18108e4dae6301363b5b243ac1518f482e27f2f32f0bb701022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe26742101200740f0587aa712a637c84b0b2bc929c14cb2ccb3846c330434459205a11be5ff610cadfdbf33fa12b98d8e947f33a350a84068e710672753cdc33315c400db9c4e0f2201012101200740321bfd17cac75d0b16fe6fd5aa9bb3e2beaf6521af4607f28815c8bd08718de8078a3fd75750354c400e1ea33cc8986853af6115bc43530cc0550ec9b2696a06"
        );
    }

    /// This test demonstrates how the hashes and payloads are constructed in a valid user transaction.
    /// It also provides an example payload which can be used in other implementations.
    #[test]
    pub fn v2_notarized_transaction_structure() {
        let network = NetworkDefinition::simulator();

        let (signed_transaction_intent, signed_transaction_intent_hash) =
            create_signed_transaction_intent_v2(&network);
        let (notary_signature, notary_signature_hash) =
            create_notary_signature_v2(signed_transaction_intent_hash);

        let notarized_transaction = NotarizedTransactionV2 {
            signed_transaction_intent,
            notary_signature,
        };
        let expected_hash = NotarizedTransactionHash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V2Notarized as u8,
                ]
                .as_slice(),
                signed_transaction_intent_hash.0.as_slice(),
                notary_signature_hash.0.as_slice(),
            ]
            .concat(),
        ));
        let raw = notarized_transaction.to_raw().unwrap();

        let prepared_transaction = notarized_transaction
            .prepare(&PreparationSettings::latest())
            .unwrap();
        let actual_transaction_intent_hash = prepared_transaction.transaction_intent_hash();
        let actual_signed_transaction_intent_hash =
            prepared_transaction.signed_transaction_intent_hash();
        let notarized_transaction_hash = prepared_transaction.notarized_transaction_hash();

        assert_eq!(expected_hash, notarized_transaction_hash);
        assert_eq!(
            notarized_transaction_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "notarizedtransaction_sim1qh37lkr547jgv5zfvlkq4njdhn62m2sg09k6njmkuma7u2hd4zasrmhyew",
        );
        assert_eq!(
            actual_signed_transaction_intent_hash
                .to_string(&TransactionHashBech32Encoder::for_simulator()),
            "signedintent_sim1z2at9wmfh7pcx7ad0c4npyv3xn3mecf2gyehwd6g6w99v56ntfsq4k92yx",
        );
        assert_eq!(
            actual_transaction_intent_hash
                .to_string(&TransactionHashBech32Encoder::for_simulator()),
            "txid_sim1v7xlgxkrk59qekpj53x8jul0lml0r4nzn3yfmv4jd5ysjewmkaust5l3t2",
        );
        assert_eq!(
            hex::encode(raw.as_slice()),
            "4d220c0221032103210322010120072009b3f25a3a1839f46ddb09b068271811f6f00a79246fb24e7a808a9e46d6075d010009000000002105210607f20a01000000000000000a0a000000000000002200002201010500000000000000000a00000000000000002020020704000102030702050622010121020c0a746578742f706c61696e2200010c0c48656c6c6f20776f726c64212020010720b37d9be9fe7362e9f01a828af77a3298758ac7d43be750575befdbd395c28918202201610209000000002100202101012105210607f20a01000000000000000a0a000000000000002200002201010500000000000000000a00000000000000002020020704000102030702050622010121020c0a746578742f706c61696e2200010c0c48656c6c6f20776f726c64212020002022016001210020220101022007204d956b5eb1147b3a80c40170e340e2918d2a9f33bdb529c54401e3ed80a4e70a2101200740e04f0e563d71ca150d900d75538d2253dff0f77d86c8ecfa4dcd25ac94de5a4ed27d76ac95c3ee8ebdcc1da52df6d1ca5f265bc1f973f631bc753e4146b3aa0c20200122010102200720c561fa9f643fe5c60113cce9db282fde2b9e5ca5fc6b6fc0d1679bb339c9f72f2101200740860417490e96c91addd5a390f5f1bcd2697535f23a947d2337b291a7b86611f56cc3aa0606ac8b8cba98381c35ef9a1f655362b18764eb90b1d8b814ec17f40e2201012101200740975a47326156a7818b4776e3e455a67c906c34eda7a9c9bb9688c77664ed9679c78aa9e33740aa1d3631b89119071a3feaf02b650799da64da7f659d107db905"
        );

        // Check that the transaction we created is actually valid...
        prepared_transaction
            .validate(&TransactionValidator::new_for_latest_simulator())
            .unwrap();
    }

    fn create_notary_signature_v2(
        hash_to_sign: SignedTransactionIntentHash,
    ) -> (NotarySignatureV2, Hash) {
        let notary_signature = NotarySignatureV2(
            TransactionV2Builder::testing_default_notary()
                .sign_without_public_key(hash_to_sign.as_hash()),
        );
        let expected_hash = hash_encoded_sbor_value_body(&notary_signature);
        let actual_hash = hash_from_partial_prepare(&notary_signature);
        assert_eq!(expected_hash, actual_hash);
        (notary_signature, actual_hash)
    }

    fn create_signed_transaction_intent_v2(
        network: &NetworkDefinition,
    ) -> (SignedTransactionIntentV2, SignedTransactionIntentHash) {
        let (transaction_intent, transaction_intent_hash, subintent_hash) =
            create_transaction_intent_v2(network);
        let (transaction_intent_signatures, transaction_intent_signatures_hash) =
            create_intent_signatures_v2(vec![2313], transaction_intent_hash);
        let (non_root_subintent_signatures, non_root_subintent_signatures_hash) =
            create_non_root_subintent_signatures(vec![subintent_hash]);

        let signed = SignedTransactionIntentV2 {
            transaction_intent,
            transaction_intent_signatures,
            non_root_subintent_signatures,
        };

        let expected_hash = SignedTransactionIntentHash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V2SignedTransactionIntent as u8,
                ]
                .as_slice(),
                transaction_intent_hash.0.as_slice(),
                transaction_intent_signatures_hash.0.as_slice(),
                non_root_subintent_signatures_hash.0.as_slice(),
            ]
            .concat(),
        ));

        let prepared = signed.prepare(&PreparationSettings::latest()).unwrap();
        let actual_hash = prepared.signed_transaction_intent_hash();
        assert_eq!(
            actual_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "signedintent_sim1z2at9wmfh7pcx7ad0c4npyv3xn3mecf2gyehwd6g6w99v56ntfsq4k92yx",
        );
        assert_eq!(expected_hash, actual_hash);

        (signed, actual_hash)
    }

    fn create_non_root_subintent_signatures(
        subintent_hashes: Vec<SubintentHash>,
    ) -> (NonRootSubintentSignaturesV2, Hash) {
        let (batches, batch_hashes): (Vec<_>, Vec<_>) = subintent_hashes
            .into_iter()
            .enumerate()
            .map(|(i, subintent_hash)| {
                create_intent_signatures_v2(vec![(i * 100 + 42) as u64], subintent_hash)
            })
            .unzip();
        let signature_batches = NonRootSubintentSignaturesV2 {
            by_subintent: batches,
        };
        let expected_hash = hash_contatenated_hashes(batch_hashes);
        let actual_hash = hash_from_partial_prepare(&signature_batches);
        assert_eq!(expected_hash, actual_hash);
        (signature_batches, expected_hash)
    }

    fn create_intent_signatures_v2(
        key_sources: Vec<u64>,
        intent_hash: impl Into<IntentHash>,
    ) -> (IntentSignaturesV2, Hash) {
        let hash_to_sign = intent_hash.into().into_hash();
        let signatures = IntentSignaturesV2 {
            signatures: key_sources
                .into_iter()
                .map(|key_source| {
                    create_intent_signature_v1(
                        Ed25519PrivateKey::from_u64(key_source).unwrap(),
                        &hash_to_sign,
                    )
                })
                .collect(),
        };
        let expected_hash = hash_encoded_sbor_value_body(&signatures);
        let actual_hash = hash_from_partial_prepare(&signatures);
        assert_eq!(expected_hash, actual_hash);
        (signatures, actual_hash)
    }

    fn create_intent_signature_v1(signer: impl Signer, hash_to_sign: &Hash) -> IntentSignatureV1 {
        let signature = signer.sign_with_public_key(hash_to_sign);
        IntentSignatureV1(signature)
    }

    fn create_transaction_intent_v2(
        network: &NetworkDefinition,
    ) -> (TransactionIntentV2, TransactionIntentHash, SubintentHash) {
        let (subintent_1, subintent_1_hash) = create_checked_childless_subintent_v2(network);
        let (non_root_subintents, non_root_subintents_hash) =
            create_non_root_subintents_v2(vec![subintent_1], vec![subintent_1_hash]);

        let (transaction_header, transaction_header_hash) = create_transaction_header_v2();
        let (root_intent_core, root_intent_core_hash) = create_intent_core_v2(
            &NetworkDefinition::simulator(),
            vec![InstructionV2::YieldToChild(YieldToChild::empty(0))],
            vec![subintent_1_hash],
        );

        let expected_transaction_intent_hash = TransactionIntentHash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V2TransactionIntent as u8,
                ]
                .as_slice(),
                transaction_header_hash.as_slice(),
                root_intent_core_hash.as_slice(),
                non_root_subintents_hash.as_slice(),
            ]
            .concat(),
        ));

        let transaction_intent = TransactionIntentV2 {
            transaction_header,
            root_intent_core,
            non_root_subintents,
        };

        let actual_hash = transaction_intent
            .prepare(PreparationSettings::latest_ref())
            .unwrap()
            .transaction_intent_hash();

        assert_eq!(expected_transaction_intent_hash, actual_hash);
        assert_eq!(
            expected_transaction_intent_hash
                .to_string(&TransactionHashBech32Encoder::for_simulator()),
            "txid_sim1v7xlgxkrk59qekpj53x8jul0lml0r4nzn3yfmv4jd5ysjewmkaust5l3t2",
        );

        (transaction_intent, actual_hash, subintent_1_hash)
    }

    fn create_transaction_header_v2() -> (TransactionHeaderV2, Hash) {
        let transaction_header = TransactionHeaderV2 {
            notary_public_key: TransactionV2Builder::testing_default_notary()
                .public_key()
                .into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        };
        let expected_hash = hash_encoded_sbor_value_body(&transaction_header);
        let actual_hash = hash_from_partial_prepare(&transaction_header);
        assert_eq!(expected_hash, actual_hash);
        (transaction_header, expected_hash)
    }

    fn create_non_root_subintents_v2(
        subintents: Vec<SubintentV2>,
        hashes: Vec<SubintentHash>,
    ) -> (NonRootSubintentsV2, Hash) {
        let non_root_subintents = NonRootSubintentsV2(subintents);

        let expected_hash = hash_contatenated_hashes(hashes);
        let actual_hash = hash_from_partial_prepare(&non_root_subintents);
        assert_eq!(expected_hash, actual_hash);

        (non_root_subintents, expected_hash)
    }

    fn create_checked_childless_subintent_v2(
        network: &NetworkDefinition,
    ) -> (SubintentV2, SubintentHash) {
        let (intent_core, intent_core_hash) = create_intent_core_v2(
            network,
            vec![InstructionV2::YieldToParent(YieldToParent::empty())],
            vec![],
        );

        let subintent = SubintentV2 { intent_core };

        let expected_subintent_hash = SubintentHash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V2Subintent as u8,
                ]
                .as_slice(),
                intent_core_hash.as_slice(),
            ]
            .concat(),
        ));

        let prepared = subintent
            .prepare(PreparationSettings::latest_ref())
            .unwrap();
        let actual_subintent_hash = prepared.subintent_hash();
        assert_eq!(expected_subintent_hash, actual_subintent_hash);
        assert_eq!(
            expected_subintent_hash.to_string(&TransactionHashBech32Encoder::for_simulator()),
            "subtxid_sim1kd7eh607wd3wnuq6s290w73jnp6c437580n4q46malda89wz3yvq3cph38",
        );

        (subintent, actual_subintent_hash)
    }

    fn create_intent_core_v2(
        network: &NetworkDefinition,
        instructions: Vec<InstructionV2>,
        children: Vec<SubintentHash>,
    ) -> (IntentCoreV2, Hash) {
        let (header, expected_header_hash) = create_intent_header_v2(network);
        let (blobs, expected_blobs_hash) = create_blobs_v1();
        let (instructions, expected_instructions_hash) =
            create_subintent_instructions_v2(instructions);
        let (message, expected_message_hash) = create_message_v2();
        let (child_intent_constraints, expected_constraints_hash) =
            create_child_subintent_specifiers_v2(children);

        let intent_core = IntentCoreV2 {
            header,
            instructions,
            blobs,
            message,
            children: child_intent_constraints,
        };

        let expected_hash = hash(
            [
                expected_header_hash.as_slice(),
                expected_blobs_hash.as_slice(),
                expected_message_hash.as_slice(),
                expected_constraints_hash.as_slice(),
                expected_instructions_hash.as_slice(),
            ]
            .concat(),
        );
        let actual_hash = hash_from_partial_prepare(&intent_core);
        assert_eq!(expected_hash, actual_hash);
        (intent_core, expected_hash)
    }

    fn create_intent_header_v2(network: &NetworkDefinition) -> (IntentHeaderV2, Hash) {
        let intent_header = IntentHeaderV2 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(10),
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: Some(Instant::new(0)),
            intent_discriminator: 0,
        };
        let expected_hash = hash_encoded_sbor_value_body(&intent_header);
        let actual_hash = hash_from_partial_prepare(&intent_header);
        assert_eq!(expected_hash, actual_hash);
        (intent_header, expected_hash)
    }

    fn create_blobs_v1() -> (BlobsV1, Hash) {
        let blob1: Vec<u8> = vec![0, 1, 2, 3];
        let blob2: Vec<u8> = vec![5, 6];
        let expected_hash = hash_contatenated_hashes([hash(&blob1), hash(&blob2)]);

        let blobs_v1 = BlobsV1 {
            blobs: vec![BlobV1(blob1), BlobV1(blob2)],
        };

        let actual_hash = hash_from_partial_prepare(&blobs_v1);
        assert_eq!(expected_hash, actual_hash);

        (blobs_v1, expected_hash)
    }

    fn create_subintent_instructions_v2(
        instructions: Vec<InstructionV2>,
    ) -> (InstructionsV2, Hash) {
        let instructions = InstructionsV2::from(instructions);
        let expected_hash = hash_encoded_sbor_value_body(&instructions);
        let actual_hash = hash_from_partial_prepare(&instructions);
        assert_eq!(expected_hash, actual_hash);

        (instructions, expected_hash)
    }

    fn create_message_v2() -> (MessageV2, Hash) {
        let message = MessageV2::Plaintext(PlaintextMessageV1::text("Hello world!"));
        let expected_hash = hash_encoded_sbor_value_body(&message);

        let actual_hash = hash_from_partial_prepare(&message);
        assert_eq!(expected_hash, actual_hash);

        (message, expected_hash)
    }

    fn create_child_subintent_specifiers_v2(
        children: Vec<SubintentHash>,
    ) -> (ChildSubintentSpecifiersV2, Hash) {
        let child_subintent_specifiers: ChildSubintentSpecifiersV2 = ChildSubintentSpecifiersV2 {
            children: children.clone().into_iter().map(|h| h.into()).collect(),
        };
        let expected_hash = hash_contatenated_hashes(children);
        let actual_hash = hash_from_partial_prepare(&child_subintent_specifiers);
        assert_eq!(expected_hash, actual_hash);

        (child_subintent_specifiers, expected_hash)
    }

    /// This test demonstrates how the hashes and payloads are constructed in a valid system transaction.
    /// A system transaction can be embedded into the node's LedgerTransaction structure, eg as part of Genesis
    #[test]
    #[allow(deprecated)] // Transaction V1 is allowed to use deprecated hashing
    pub fn v1_system_transaction_structure() {
        let instructions = vec![InstructionV1::DropAuthZoneProofs(DropAuthZoneProofs)];
        let expected_instructions_hash = hash_encoded_sbor_value(&instructions);
        let instructions_v1 = InstructionsV1(instructions);

        let blob1: Vec<u8> = vec![0, 1, 2, 3];
        let blob2: Vec<u8> = vec![5, 6];
        let expected_blobs_hash =
            hash([hash(&blob1).0.as_slice(), hash(&blob2).0.as_slice()].concat());

        let blobs_v1 = BlobsV1 {
            blobs: vec![BlobV1(blob1), BlobV1(blob2)],
        };

        let prepared_blobs_v1 = blobs_v1
            .prepare_partial(PreparationSettings::latest_ref())
            .unwrap();
        assert_eq!(prepared_blobs_v1.get_summary().hash, expected_blobs_hash);

        let pre_allocated_addresses_v1 = vec![PreAllocatedAddress {
            blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            address: XRD.into(),
        }];
        let expected_preallocated_addresses_hash =
            hash_encoded_sbor_value(&pre_allocated_addresses_v1);

        let hash_for_execution = hash("Pretend genesis transaction");

        let system_transaction_v1 = SystemTransactionV1 {
            instructions: instructions_v1.clone(),
            blobs: blobs_v1.clone(),
            pre_allocated_addresses: pre_allocated_addresses_v1.clone(),
            hash_for_execution,
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

        let raw_system_transaction = system_transaction_v1.to_raw().unwrap();
        SystemTransactionV1::from_raw(&raw_system_transaction)
            .expect("SystemTransaction can be decoded");
        let system_transaction_as_versioned =
            manifest_decode::<AnyTransaction>(raw_system_transaction.as_slice()).unwrap();
        assert_eq!(
            system_transaction_as_versioned,
            AnyTransaction::SystemTransactionV1(system_transaction_v1)
        );

        let prepared_system_transaction = PreparedSystemTransactionV1::prepare(
            &raw_system_transaction,
            PreparationSettings::latest_ref(),
        )
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
            hex::encode(raw_system_transaction),
            "4d22040420220112002020020704000102030702050620210102210280000d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c60c1746756e6769626c655265736f757263654d616e6167657280005da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c62007207646fcb3e6a2dbf0fd4830933c54928d3e8dafaf9f704afdae56336fc67aae0d"
        );
    }
}

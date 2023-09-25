mod concepts;
mod executable;
mod hash;
mod preparation;
mod v1;
mod versioned;

pub use concepts::*;
pub use executable::*;
pub use hash::*;
pub use preparation::*;
pub use v1::*;
pub use versioned::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal_prelude::{
            NotarizedTransactionValidator, TransactionValidationError, TransactionValidator,
            ValidationConfig,
        },
        prelude::*,
    };
    use radix_engine_common::prelude::*;
    use sbor::representations::*;

    fn reconcile_manifest_sbor(payload: &[u8], expected: &str) {
        let display_context = ValueDisplayParameters::Schemaless {
            display_mode: DisplayMode::NestedString,
            print_mode: PrintMode::MultiLine {
                indent_size: 4,
                base_indent: 0,
                first_line_indent: 0,
            },
            custom_context: Default::default(),
            depth_limit: 64,
        };
        let actual = ManifestRawPayload::new_from_valid_slice_with_checks(&payload)
            .unwrap()
            .to_string(display_context);
        let actual_clean: String = actual
            .trim()
            .split("\n")
            .map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join("\n");

        let expected_clean: String = expected
            .trim()
            .split("\n")
            .map(|line| line.split("//").next().unwrap().trim())
            .collect::<Vec<&str>>()
            .join("\n");

        println!("{}", actual);
        assert_eq!(actual_clean, expected_clean);
    }

    #[test]
    fn transaction_bytes_recon() {
        let network = NetworkDefinition::mainnet();
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let sig_2_private_key = Ed25519PrivateKey::from_u64(2).unwrap();
        let notary_private_key = Ed25519PrivateKey::from_u64(3).unwrap();

        let header_v1 = TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::of(55),
            end_epoch_exclusive: Epoch::of(66),
            nonce: 77,
            notary_public_key: notary_private_key.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 4,
        };
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .drop_all_proofs()
            .then(|mut builder| {
                builder.add_blob(vec![1, 2]);
                builder.set_message(MessageV1::Plaintext(PlaintextMessageV1 {
                    mime_type: "text/plain".to_owned(),
                    message: MessageContentsV1::String("hi".to_owned()),
                }));
                builder
            })
            .build();
        let transaction = TransactionBuilder::new()
            .header(header_v1)
            .manifest(manifest.clone())
            .sign(&sig_1_private_key)
            .sign(&sig_2_private_key)
            .notarize(&notary_private_key)
            .build();
        let mut payload = transaction.to_payload_bytes().unwrap();
        println!("{:?}", payload);

        reconcile_manifest_sbor(
            &payload,
            r###"
Enum<3u8>(
    Tuple(                  // signed intent
        Tuple(              // intent
            Tuple(          // header
                1u8,        // * network id
                55u64,      // * epoch start
                66u64,      // * epoch end
                77u32,      // * none
                Enum<1u8>(  // * notary public key
                    Array<U8>(Hex("f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b")),
                ),
                false,      // * notary is signatory
                4u16,       // * tip percentage 4%
            ),
            Array<Enum>(    // instructions
                Enum<65u8>(
                    Address("c0566318c6318c64f798cacc6318c6318cf7be8af78a78f8a6318c6318c6"),
                    "lock_fee",
                    Tuple(
                        Decimal("5000"),
                    ),
                ),
                Enum<80u8>(),
            ),
            Array<Array>(   // blobs
                Array<U8>(Hex("0102")),
            ),
            Enum<1u8>(      // message
                Tuple(
                    "text/plain",
                    Enum<0u8>(
                        "hi",
                    ),
                ),
            ),
        ),
        Array<Enum>(        // signature
            Enum<0u8>(
                Tuple(      // NOTE: unneeded struct
                    Array<U8>(Hex("00eb7980eb88500715d6d5bacf5d2bf8d0423450d54122ba7267162a1d241d0b854e1ca8b77ce283b4812b37bb54c523c5be6cfc9d4b6998af5815917220bc31c8")),
                ),
            ),
            Enum<1u8>(
                Array<U8>(Hex("7422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674")),
                Tuple(      // NOTE: unneeded struct
                    Array<U8>(Hex("39b5f82e2e5a40a1d6329d47736c4681fa5eec57866dd4f554a1a99fe4a7f9bf9fd72f7a8c7b7c2071fe4d6ded359c2dce61539cbfafbbfab33d3896c27e4205")),
                ),
            ),
        ),
    ),
    Enum<1u8>(              // notary signature
        Tuple(              // NOTE: unneeded struct
            Array<U8>(Hex("1a8334fd9af4622cd0f81b7e2d5f3033037f605c288c2d91e4b648fe0a2f153a60f739b1e7349dc4078e787e2db30bfb60405089c1fb7bc0c3bf8fed4a86df0e")),
        ),
    ),
)
"###,
        );

        let validated = NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
            .validate_from_payload_bytes(&payload)
            .unwrap();
        let executable = validated.get_executable();
        assert_eq!(
            executable,
            Executable {
                encoded_instructions: &manifest_encode(&manifest.instructions).unwrap(),
                references: indexset!(
                    Reference(
                        NodeId::try_from_hex(
                            "c0566318c6318c64f798cacc6318c6318cf7be8af78a78f8a6318c6318c6"
                        )
                        .unwrap()
                    ),
                    // NOTE: not needed
                    Reference(SECP256K1_SIGNATURE_VIRTUAL_BADGE.into_node_id()),
                    Reference(ED25519_SIGNATURE_VIRTUAL_BADGE.into_node_id())
                ),
                blobs: &indexmap!(
                    hash(&[1, 2]) => vec![1, 2]
                ),
                context: ExecutionContext {
                    intent_hash: TransactionIntentHash::ToCheck {
                        intent_hash: hash(
                            [
                                [
                                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                                    TransactionDiscriminator::V1Intent as u8,
                                ]
                                .as_slice(),
                                hash(
                                    &manifest_encode(&transaction.signed_intent.intent.header)
                                        .unwrap()[1..]
                                )
                                .as_slice(),
                                hash(
                                    &manifest_encode(
                                        &transaction.signed_intent.intent.instructions.0
                                    )
                                    .unwrap()[1..]
                                )
                                .as_slice(),
                                hash(
                                    &transaction.signed_intent.intent.blobs.blobs[0].0 // one blob only
                                )
                                .as_slice(),
                                hash(
                                    &manifest_encode(&transaction.signed_intent.intent.message)
                                        .unwrap()[1..]
                                )
                                .as_slice(),
                            ]
                            .concat(),
                        ),
                        expiry_epoch: Epoch::of(66)
                    },
                    epoch_range: Some(EpochRange {
                        start_epoch_inclusive: Epoch::of(55),
                        end_epoch_exclusive: Epoch::of(66)
                    }),
                    pre_allocated_addresses: vec![],
                    payload_size: payload.len(),
                    num_of_signature_validations: 3,
                    auth_zone_params: AuthZoneParams {
                        initial_proofs: btreeset!(
                            NonFungibleGlobalId::from_public_key(&sig_1_private_key.public_key()),
                            NonFungibleGlobalId::from_public_key(&sig_2_private_key.public_key())
                        ),
                        virtual_resources: btreeset!()
                    },
                    costing_parameters: TransactionCostingParameters {
                        tip_percentage: 4,
                        free_credit_in_xrd: dec!(0)
                    }
                }
            }
        );

        // Test unexpected transaction type
        payload[2] = 4;
        let executable = NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
            .validate_from_payload_bytes(&payload);
        assert_eq!(
            executable,
            Err(TransactionValidationError::PrepareError(
                PrepareError::UnexpectedDiscriminator {
                    expected: 3,
                    actual: 4
                }
            ))
        )
    }
}

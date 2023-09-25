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
        internal_prelude::{NotarizedTransactionValidator, TransactionValidator, ValidationConfig},
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
        let actual_clean: String = expected
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
        let transaction = TransactionBuilder::new()
            .header(header_v1)
            .manifest(
                ManifestBuilder::new()
                    .drop_all_proofs()
                    .drop_all_proofs()
                    .build(),
            )
            .sign(&sig_1_private_key)
            .sign(&sig_2_private_key)
            .notarize(&notary_private_key)
            .build();
        let payload = transaction.to_payload_bytes().unwrap();
        println!("{:?}", payload);

        reconcile_manifest_sbor(
            &payload,
            r###"Enum(
    Tuple(
        Tuple(
            Tuple(
                1u8,
                55u64,
                66u64,
                77u32,
                Enum(
                    Array<U8>(Hex("f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b")),
                ),
                false,
                4u16,
            ),
            Array<Enum>(
                Enum(80u8),
                Enum(80u8),
            ),
            Array<Array>(),
            Enum(0u8),
        ),
        Array<Enum>(
            Enum(
                Tuple(
                    Array<U8>(Hex("01d331fdc9898abfca2fccd3f578ae8bbe2615ff4ab2e2e0ad0b92cd1523c63a262b90b71d976d040c9891a6a90aa7e31372e02cbe2af962ea65270f9b868aaf22")),
                ),
            ),
            Enum(
                Array<U8>(Hex("7422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674")),
                Tuple(
                    Array<U8>(Hex("9f76cdb109f814fde2c020fb443134ae738288f2eedede8e897e9a3253b2ec8411d93f065088f87c562ce2950c1a4c163e1c5c6f5e3f312654a797b480a51c0f")),
                ),
            ),
        ),
    ),
    Enum(
        Tuple(
            Array<U8>(Hex("7483351fafd4cbbb87d76abe10f7a5b9257139ee91897f034ad4889b510673376461aa94469d3116314f6ae2c60916e960d6e8d9055c07087c0038c0468b4e08")),
        ),
    ),
)"###,
        );

        let executable = NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
            .validate_from_payload_bytes(&payload)
            .unwrap();
        println!("{:?}", executable);
    }
}

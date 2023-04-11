use super::*;
use crate::*;
use sbor::representations::*;
use sbor::rust::prelude::*;
use sbor::traversal::*;
use utils::ContextualDisplay;

impl<'a> CustomDisplayContext<'a> for ScryptoValueDisplayContext<'a> {
    type CustomTypeExtension = ScryptoCustomTypeExtension;
}

impl FormattableCustomTypeExtension for ScryptoCustomTypeExtension {
    type CustomDisplayContext<'a> = ScryptoValueDisplayContext<'a>;

    fn display_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error> {
        match &value.0 {
            ScryptoCustomValue::Address(value) => {
                write!(f, "\"{}\"", value.display(context.bech32_encoder))?;
            }
            ScryptoCustomValue::Own(value) => {
                write!(f, "\"{}\"", hex::encode(value.to_vec()))?;
            }
            ScryptoCustomValue::Decimal(value) => {
                write!(f, "\"{}\"", value)?;
            }
            ScryptoCustomValue::PreciseDecimal(value) => {
                write!(f, "\"{}\"", value)?;
            }
            ScryptoCustomValue::NonFungibleLocalId(value) => {
                write!(f, "\"{}\"", value)?;
            }
            ScryptoCustomValue::InternalRef(value) => {
                write!(f, "\"{}\"", hex::encode(value.to_vec()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Bech32Encoder;
    use crate::data::scrypto::model::*;

    #[test]
    fn test_rustlike_string_format_with_network() {
        use crate::math::{Decimal, PreciseDecimal};

        let encoder = Bech32Encoder::for_simulator();
        let value = ScryptoValue::Tuple {
            fields: vec![
                Value::Custom {
                    value: ScryptoCustomValue::Address(Address::Resource(
                        ResourceAddress::Fungible([0; ADDRESS_HASH_LENGTH]),
                    )),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own::Vault([0; OBJECT_ID_LENGTH])),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Decimal(Decimal::ONE),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Decimal(Decimal::ONE / 100),
                },
                Value::Custom {
                    value: ScryptoCustomValue::PreciseDecimal(PreciseDecimal::ZERO),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(
                        NonFungibleLocalId::string("hello").unwrap(),
                    ),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(NonFungibleLocalId::integer(123)),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(
                        NonFungibleLocalId::bytes(vec![0x23, 0x45]).unwrap(),
                    ),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(
                        NonFungibleLocalId::uuid(0x1f52cb1e_86c4_47ae_9847_9cdb14662ebd).unwrap(),
                    ),
                },
                Value::Custom {
                    value: ScryptoCustomValue::InternalRef(InternalRef([0; OBJECT_ID_LENGTH])),
                },
            ],
        };

        let expected = "Tuple(Address(\"resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k\"), Own(\"0200000000000000000000000000000000000000000000000000000000000000\"), Decimal(\"1\"), Decimal(\"0.01\"), PreciseDecimal(\"0\"), NonFungibleLocalId(\"<hello>\"), NonFungibleLocalId(\"#123#\"), NonFungibleLocalId(\"[2345]\"), NonFungibleLocalId(\"{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}\"), Reference(\"00000000000000000000000000000000000000000000000000000000000000\"))";

        let context = ScryptoValueDisplayContext::with_optional_bech32(Some(&encoder));

        let payload = ScryptoRawPayload::new_from_valid_owned(scrypto_encode(&value).unwrap());

        let actual_rustlike = payload.to_string(ValueDisplayParameters::Schemaless {
            display_mode: DisplayMode::RustLike,
            print_mode: PrintMode::SingleLine,
            custom_context: context,
        });
        let actual_nested = payload.to_string(ValueDisplayParameters::Schemaless {
            display_mode: DisplayMode::RustLike,
            print_mode: PrintMode::SingleLine,
            custom_context: context,
        });

        // They're both the same
        assert_eq!(&actual_rustlike, expected);
        assert_eq!(&actual_nested, expected);
    }
}

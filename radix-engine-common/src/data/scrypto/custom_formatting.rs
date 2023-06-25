use crate::internal_prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct ScryptoValueDisplayContext<'a> {
    pub address_bech32_encoder: Option<&'a AddressBech32Encoder>,
}

impl<'a> ScryptoValueDisplayContext<'a> {
    pub fn no_context() -> Self {
        Self {
            address_bech32_encoder: None,
        }
    }

    pub fn with_optional_bech32(address_bech32_encoder: Option<&'a AddressBech32Encoder>) -> Self {
        Self {
            address_bech32_encoder,
        }
    }
}

impl<'a> Into<ScryptoValueDisplayContext<'a>> for &'a AddressBech32Encoder {
    fn into(self) -> ScryptoValueDisplayContext<'a> {
        ScryptoValueDisplayContext::with_optional_bech32(Some(self))
    }
}

impl<'a> Into<ScryptoValueDisplayContext<'a>> for Option<&'a AddressBech32Encoder> {
    fn into(self) -> ScryptoValueDisplayContext<'a> {
        ScryptoValueDisplayContext::with_optional_bech32(self)
    }
}

impl<'a> CustomDisplayContext<'a> for ScryptoValueDisplayContext<'a> {
    type CustomExtension = ScryptoCustomExtension;
}

impl FormattableCustomExtension for ScryptoCustomExtension {
    type CustomDisplayContext<'a> = ScryptoValueDisplayContext<'a>;

    fn display_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error> {
        match &value.0 {
            ScryptoCustomValue::Reference(value) => {
                write!(f, "\"{}\"", value.0.display(context.address_bech32_encoder))?;
            }
            ScryptoCustomValue::Own(value) => {
                write!(f, "\"{}\"", value.0.display(context.address_bech32_encoder))?;
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
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::test_addresses::*;
    use crate::address::AddressBech32Encoder;
    use crate::data::scrypto::model::*;

    #[test]
    fn test_rustlike_string_format_with_network() {
        use crate::math::{Decimal, PreciseDecimal};
        let encoder = AddressBech32Encoder::for_simulator();
        let value = ScryptoValue::Tuple {
            fields: vec![
                Value::Custom {
                    value: ScryptoCustomValue::Reference(Reference(FUNGIBLE_RESOURCE_NODE_ID)),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(FUNGIBLE_RESOURCE_NODE_ID)),
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
                    value: ScryptoCustomValue::NonFungibleLocalId(NonFungibleLocalId::ruid(
                        [0x11; 32],
                    )),
                },
            ],
        };

        let expected = format!("Tuple(Reference(\"{FUNGIBLE_RESOURCE_SIM_ADDRESS}\"), Own(\"{FUNGIBLE_RESOURCE_SIM_ADDRESS}\"), Decimal(\"1\"), Decimal(\"0.01\"), PreciseDecimal(\"0\"), NonFungibleLocalId(\"<hello>\"), NonFungibleLocalId(\"#123#\"), NonFungibleLocalId(\"[2345]\"), NonFungibleLocalId(\"{{1111111111111111-1111111111111111-1111111111111111-1111111111111111}}\"))");

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
        assert_eq!(actual_rustlike, expected);
        assert_eq!(actual_nested, expected);
    }
}

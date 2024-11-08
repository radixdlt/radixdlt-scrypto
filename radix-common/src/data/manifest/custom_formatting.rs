use super::converter::*;
use super::model::*;
use crate::internal_prelude::*;

impl<'a> CustomDisplayContext<'a> for ManifestValueDisplayContext<'a> {
    type CustomExtension = ManifestCustomExtension;
}

impl FormattableCustomExtension for ManifestCustomExtension {
    type CustomDisplayContext<'a> = ManifestValueDisplayContext<'a>;

    fn display_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error> {
        match &value.0 {
            ManifestCustomValue::Address(value) => match value {
                ManifestAddress::Static(node_id) => {
                    if let Some(encoder) = context.address_bech32_encoder {
                        if let Ok(bech32) = encoder.encode(node_id.as_ref()) {
                            write!(f, "\"{}\"", bech32)?;
                        } else {
                            write!(f, "\"{}\"", hex::encode(node_id.as_bytes()))?;
                        }
                    } else {
                        write!(f, "\"{}\"", hex::encode(node_id.as_bytes()))?;
                    }
                }
                ManifestAddress::Named(value) => {
                    if let Some(name) = context.get_address_name(&value) {
                        write!(f, "\"{}\"", name)?;
                    } else {
                        write!(f, "\"{}\"", value.0)?;
                    }
                }
            },
            ManifestCustomValue::Bucket(value) => {
                if let Some(name) = context.get_bucket_name(&value) {
                    write!(f, "\"{}\"", name)?;
                } else {
                    write!(f, "\"{}\"", value.0)?;
                }
            }
            ManifestCustomValue::Proof(value) => {
                if let Some(name) = context.get_proof_name(&value) {
                    write!(f, "\"{}\"", name)?;
                } else {
                    write!(f, "\"{}\"", value.0)?;
                }
            }
            ManifestCustomValue::AddressReservation(value) => {
                write!(f, "\"{}\"", value.0)?;
            }
            ManifestCustomValue::Expression(value) => {
                match value {
                    ManifestExpression::EntireWorktop => write!(f, "\"ENTIRE_WORKTOP\"")?,
                    ManifestExpression::EntireAuthZone => write!(f, "\"ENTIRE_AUTH_ZONE\"")?,
                };
            }
            ManifestCustomValue::Blob(value) => {
                write!(f, "\"{}\"", hex::encode(&value.0))?;
            }
            ManifestCustomValue::Decimal(value) => {
                write!(f, "\"{}\"", to_decimal(value))?;
            }
            ManifestCustomValue::PreciseDecimal(value) => {
                write!(f, "\"{}\"", to_precise_decimal(value))?;
            }
            ManifestCustomValue::NonFungibleLocalId(value) => {
                write!(f, "\"{}\"", to_non_fungible_local_id(value.clone()))?;
            }
        }
        Ok(())
    }

    fn code_generation_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error> {
        let wrapper = match &value.0 {
            ManifestCustomValue::Address(ManifestAddress::Static(_)) => "Address",
            ManifestCustomValue::Address(ManifestAddress::Named(_)) => "NamedAddress",
            ManifestCustomValue::Bucket(_) => "Bucket",
            ManifestCustomValue::Proof(_) => "Proof",
            ManifestCustomValue::Expression(_) => "Expression",
            ManifestCustomValue::Blob(_) => "Blob",
            ManifestCustomValue::Decimal(_) => "Decimal",
            ManifestCustomValue::PreciseDecimal(_) => "PreciseDecimal",
            ManifestCustomValue::NonFungibleLocalId(_) => "NonFungibleLocalId",
            ManifestCustomValue::AddressReservation(_) => "AddressReservation",
        };
        write!(f, "{wrapper}(")?;
        Self::display_string_content(f, context, value)?;
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use radix_rust::ContextualDisplay;

    use super::*;
    use crate::address::test_addresses::*;
    use crate::address::AddressBech32Encoder;

    #[test]
    fn test_rustlike_string_format_with_network() {
        let encoder = AddressBech32Encoder::for_simulator();
        let payload = manifest_encode(&(
            ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    FUNGIBLE_RESOURCE.as_node_id().clone(),
                )),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Blob(ManifestBlobRef([0; 32])),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Bucket(ManifestBucket(0)),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Proof(ManifestProof(0)),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Decimal(ManifestDecimal([0; DECIMAL_SIZE])),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::PreciseDecimal(ManifestPreciseDecimal(
                    [0; PRECISE_DECIMAL_SIZE],
                )),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::NonFungibleLocalId(ManifestNonFungibleLocalId::String(
                    "hello".to_string(),
                )),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Expression(ManifestExpression::EntireAuthZone),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Expression(ManifestExpression::EntireWorktop),
            },
        ))
        .unwrap();

        let expected = format!("Tuple(Address(\"{FUNGIBLE_RESOURCE_SIM_ADDRESS}\"), Blob(\"0000000000000000000000000000000000000000000000000000000000000000\"), Bucket(\"0\"), Proof(\"0\"), Decimal(\"0\"), PreciseDecimal(\"0\"), NonFungibleLocalId(\"<hello>\"), Expression(\"ENTIRE_AUTH_ZONE\"), Expression(\"ENTIRE_WORKTOP\"))");

        let context = ManifestValueDisplayContext::with_optional_bech32(Some(&encoder));

        let payload = ManifestRawPayload::new_from_valid_owned(payload);

        let actual_rustlike = payload.to_string(ValueDisplayParameters::Schemaless {
            display_mode: DisplayMode::RustLike(RustLikeOptions::full()),
            print_mode: PrintMode::SingleLine,
            custom_context: context,
            depth_limit: MANIFEST_SBOR_V1_MAX_DEPTH,
        });
        let actual_nested = payload.to_string(ValueDisplayParameters::Schemaless {
            display_mode: DisplayMode::RustLike(RustLikeOptions::full()),
            print_mode: PrintMode::SingleLine,
            custom_context: context,
            depth_limit: MANIFEST_SBOR_V1_MAX_DEPTH,
        });

        // They're both the same
        assert_eq!(actual_rustlike, expected);
        assert_eq!(actual_nested, expected);
    }
}

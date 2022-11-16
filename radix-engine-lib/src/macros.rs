/// Creates a `Decimal` from literals.
///
#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        radix_engine_lib::math::Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = radix_engine_lib::math::Decimal::from($base);
            if $shift >= 0 {
                base * radix_engine_lib::math::Decimal::try_from(
                    radix_engine_lib::math::I256::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / radix_engine_lib::math::Decimal::try_from(
                    radix_engine_lib::math::I256::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}

/// Creates a safe integer from literals.
/// You must specify the type of the
/// integer you want to create.
///
#[macro_export]
macro_rules! i {
    ($x:expr) => {
        $x.try_into().expect("Parse Error")
    };
}

/// Creates a `PreciseDecimal` from literals.
///
#[macro_export]
macro_rules! pdec {
    ($x:literal) => {
        radix_engine_lib::math::PreciseDecimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a PreciseDecimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = radix_engine_lib::math::PreciseDecimal::from($base);
            if $shift >= 0 {
                base * radix_engine_lib::math::PreciseDecimal::try_from(
                    radix_engine_lib::math::I512::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / radix_engine_lib::math::PreciseDecimal::try_from(
                    radix_engine_lib::math::I512::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}

/// A macro for implementing sbor traits.
#[macro_export]
macro_rules! scrypto_type {
    // static size
    ($t:ty, $type_id:expr, $schema_type: expr, $size: expr) => {
        impl sbor::TypeId<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn type_id() -> sbor::SborTypeId<crate::data::ScryptoCustomTypeId> {
                sbor::SborTypeId::Custom($type_id)
            }
        }

        impl sbor::Encode<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn encode_type_id(encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_slice(&self.to_vec());
            }
        }

        impl sbor::Decode<crate::data::ScryptoCustomTypeId> for $t {
            fn check_type_id(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<(), sbor::DecodeError> {
                decoder.check_type_id(Self::type_id())
            }

            fn decode_value(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<Self, sbor::DecodeError> {
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }

        impl scrypto_abi::Describe for $t {
            fn describe() -> scrypto_abi::Type {
                $schema_type
            }
        }
    };

    // dynamic size
    ($t:ty, $type_id:expr, $schema_type: expr) => {
        impl sbor::TypeId<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn type_id() -> sbor::SborTypeId<crate::data::ScryptoCustomTypeId> {
                sbor::SborTypeId::Custom($type_id)
            }
        }

        impl sbor::Encode<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn encode_type_id(encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                let bytes = self.to_vec();
                encoder.write_size(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl sbor::Decode<crate::data::ScryptoCustomTypeId> for $t {
            fn check_type_id(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<(), sbor::DecodeError> {
                decoder.check_type_id(Self::type_id())
            }

            fn decode_value(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<Self, sbor::DecodeError> {
                let len = decoder.read_size()?;
                let slice = decoder.read_slice(len)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }

        impl scrypto_abi::Describe for $t {
            fn describe() -> scrypto_abi::Type {
                $schema_type
            }
        }
    };
}

/// Creates a `Decimal` from literals.
///
#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = Decimal::from($base);
            if $shift >= 0 {
                base * Decimal::try_from(
                    I256::from(10u8).pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / Decimal::try_from(
                    I256::from(10u8).pow(u32::try_from(-$shift).expect("Shift overflow")),
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
        PreciseDecimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a PreciseDecimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = PreciseDecimal::from($base);
            if $shift >= 0 {
                base * PreciseDecimal::try_from(
                    I512::from(10u8).pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / PreciseDecimal::try_from(
                    I512::from(10u8).pow(u32::try_from(-$shift).expect("Shift overflow")),
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
            fn encode_type_id(
                &self,
                encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>,
            ) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_body(&self, encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_slice(&self.to_vec());
            }
        }

        impl<D: sbor::Decoder<crate::data::ScryptoCustomTypeId>>
            sbor::Decode<crate::data::ScryptoCustomTypeId, D> for $t
        {
            fn decode_body_with_type_id(
                decoder: &mut D,
                type_id: sbor::SborTypeId<crate::data::ScryptoCustomTypeId>,
            ) -> Result<Self, sbor::DecodeError> {
                decoder.check_preloaded_type_id(type_id, Self::type_id())?;
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
            fn encode_type_id(
                &self,
                encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>,
            ) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_body(&self, encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                let bytes = self.to_vec();
                encoder.write_size(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl<D: sbor::Decoder<crate::data::ScryptoCustomTypeId>>
            sbor::Decode<crate::data::ScryptoCustomTypeId, D> for $t
        {
            fn decode_body_with_type_id(
                decoder: &mut D,
                type_id: sbor::SborTypeId<crate::data::ScryptoCustomTypeId>,
            ) -> Result<Self, sbor::DecodeError> {
                decoder.check_preloaded_type_id(type_id, Self::type_id())?;
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

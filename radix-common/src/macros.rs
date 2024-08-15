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

/// A macro for implementing sbor traits (for statically sized types).
#[macro_export]
macro_rules! well_known_scrypto_custom_type {
    // with describe
    ($t:ty, $value_kind:expr, $schema_type:expr, $size:expr, $well_known_type:ident, $well_known_type_data_method:ident$(,)?) => {
        impl sbor::Categorize<$crate::data::scrypto::ScryptoCustomValueKind> for $t {
            #[inline]
            fn value_kind() -> sbor::ValueKind<$crate::data::scrypto::ScryptoCustomValueKind> {
                sbor::ValueKind::Custom($value_kind)
            }
        }

        impl<E: sbor::Encoder<$crate::data::scrypto::ScryptoCustomValueKind>>
            sbor::Encode<$crate::data::scrypto::ScryptoCustomValueKind, E> for $t
        {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                encoder.write_value_kind(Self::value_kind())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                encoder.write_slice(&self.to_vec())
            }
        }

        impl<D: sbor::Decoder<$crate::data::scrypto::ScryptoCustomValueKind>>
            sbor::Decode<$crate::data::scrypto::ScryptoCustomValueKind, D> for $t
        {
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: sbor::ValueKind<$crate::data::scrypto::ScryptoCustomValueKind>,
            ) -> Result<Self, sbor::DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }

        impl sbor::Describe<$crate::data::scrypto::ScryptoCustomTypeKind> for $t {
            const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::WellKnown(
                $crate::data::scrypto::well_known_scrypto_custom_types::$well_known_type,
            );

            fn type_data() -> sbor::TypeData<$crate::data::scrypto::ScryptoCustomTypeKind, sbor::RustTypeId> {
                $crate::data::scrypto::well_known_scrypto_custom_types::$well_known_type_data_method()
            }
        }
    };
}

#[macro_export]
macro_rules! manifest_type {
    // Without describe - if you need describe, also use scrypto_describe_for_manifest_type!
    ($t:ty, $value_kind:expr, $size: expr$(,)?) => {
        impl sbor::Categorize<$crate::data::manifest::ManifestCustomValueKind> for $t {
            #[inline]
            fn value_kind() -> sbor::ValueKind<$crate::data::manifest::ManifestCustomValueKind> {
                sbor::ValueKind::Custom($value_kind)
            }
        }

        impl<E: sbor::Encoder<$crate::data::manifest::ManifestCustomValueKind>>
            sbor::Encode<$crate::data::manifest::ManifestCustomValueKind, E> for $t
        {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                encoder.write_value_kind(Self::value_kind())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                encoder.write_slice(&self.to_vec())
            }
        }

        impl<D: sbor::Decoder<$crate::data::manifest::ManifestCustomValueKind>>
            sbor::Decode<$crate::data::manifest::ManifestCustomValueKind, D> for $t
        {
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: sbor::ValueKind<$crate::data::manifest::ManifestCustomValueKind>,
            ) -> Result<Self, sbor::DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }
    };
}

#[macro_export]
macro_rules! scrypto_describe_for_manifest_type {
    ($t:ty, $well_known_type:ident, $well_known_type_data_method:ident$(,)?) => {
        impl sbor::Describe<$crate::data::scrypto::ScryptoCustomTypeKind> for $t {
            const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::WellKnown(
                $crate::data::scrypto::well_known_scrypto_custom_types::$well_known_type,
            );

            fn type_data() -> sbor::TypeData<$crate::data::scrypto::ScryptoCustomTypeKind, sbor::RustTypeId> {
                $crate::data::scrypto::well_known_scrypto_custom_types::$well_known_type_data_method()
            }
        }
    }
}

#[macro_export]
macro_rules! count {
    () => {0usize};
    ($a:expr) => {1usize};
    ($a:expr, $($rest:expr),*) => {1usize + $crate::count!($($rest),*)};
}

#[macro_export]
macro_rules! scrypto_args {
    ($($args: expr),*) => {{
        use sbor::Encoder;
        let mut buf = sbor::rust::vec::Vec::new();
        let mut encoder = $crate::data::scrypto::ScryptoEncoder::new(
            &mut buf,
            $crate::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH,
        );
        encoder
            .write_payload_prefix($crate::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder
            .write_value_kind($crate::data::scrypto::ScryptoValueKind::Tuple)
            .unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size($crate::count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
        )*
        buf
    }};
}

#[macro_export]
macro_rules! manifest_args {
    ($($args: expr),*$(,)?) => {{
        use sbor::Encoder;
        let mut buf = sbor::rust::vec::Vec::new();
        let mut encoder = $crate::data::manifest::ManifestEncoder::new(&mut buf, $crate::data::manifest::MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder.write_payload_prefix($crate::data::manifest::MANIFEST_SBOR_V1_PAYLOAD_PREFIX).unwrap();
        encoder.write_value_kind($crate::data::manifest::ManifestValueKind::Tuple).unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size($crate::count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
        )*
        let value = $crate::data::manifest::manifest_decode(&buf).unwrap();
        ManifestArgs::new_from_tuple_or_panic(value)
    }};
}

#[macro_export]
macro_rules! to_manifest_value_and_unwrap {
    ( $value:expr ) => {{
        $crate::data::manifest::to_manifest_value($value).unwrap()
    }};
}

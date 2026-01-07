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

/// This macro is used to create types that are supposed to be equivalent to existing types that
/// implement [`ScryptoSbor`]. These types are untyped which means that at the language-level, they
/// don't contain any type information (e.g., enums don't contain variants, structs don't contain
/// named fields, etc...).
///
/// It's a useful concepts for certain types that are too complex to create an equivalent manifest
/// types for. For example, `AccessRules` is too complex and has a lot of dependencies and therefore
/// creating a manifest type for it is very complex. Instead, we could opt to a create a manifest
/// type for it that has the same schema as it and that offers two way conversion from and into it
/// such that we can use it in any manifest invocation.
///
/// The `$scrypto_ty` provided to this macro must implement [`ScryptoSbor`] and [`ScryptoDescribe`]
/// and the generated type (named `$manifest_ty_ident`) will implement [`ManifestSbor`] and also
/// [`ScryptoDescribe`] with the same schema as the original Scrypto type.
///
/// # Note on Panics
///
/// The `From<$scrypto_ty> for $manifest_ty_ident` implementation will PANIC if it fails. Ensure
/// that this macro is not used anywhere in the engine itself and only used in the interface.
///
/// [`ScryptoSbor`]: crate::prelude::ScryptoSbor
/// [`ScryptoDescribe`]: crate::prelude::ScryptoDescribe
/// [`ManifestSbor`]: crate::prelude::ManifestSbor
#[macro_export]
macro_rules! define_untyped_manifest_type_wrapper {
    (
        $scrypto_ty: ty => $manifest_ty_ident: ident ($inner_ty: ty)
    ) => {
        #[cfg_attr(
            feature = "fuzzing",
            derive(::arbitrary::Arbitrary, ::serde::Serialize, ::serde::Deserialize)
        )]
        #[derive(Debug, Clone, PartialEq, Eq, $crate::prelude::ManifestSbor)]
        #[sbor(transparent)]
        pub struct $manifest_ty_ident($inner_ty);

        const _: () = {
            impl $manifest_ty_ident {
                pub fn new(
                    value: impl Into<$scrypto_ty>,
                ) -> Result<Self, $crate::prelude::ConversionError> {
                    let value = Into::<$scrypto_ty>::into(value);
                    let encoded_scrypto_bytes = $crate::prelude::scrypto_encode(&value)
                        .map_err($crate::prelude::ConversionError::EncodeError)?;
                    let scrypto_value = $crate::prelude::scrypto_decode::<
                        $crate::prelude::ScryptoValue,
                    >(&encoded_scrypto_bytes)
                    .map_err($crate::prelude::ConversionError::DecodeError)?;

                    let manifest_value =
                        $crate::prelude::scrypto_value_to_manifest_value(scrypto_value)?;
                    let encoded_manifest_bytes = $crate::prelude::manifest_encode(&manifest_value)
                        .map_err($crate::prelude::ConversionError::EncodeError)?;
                    $crate::prelude::manifest_decode(&encoded_manifest_bytes)
                        .map(Self)
                        .map_err($crate::prelude::ConversionError::DecodeError)
                }

                pub fn try_into_typed(
                    self,
                ) -> Result<$scrypto_ty, $crate::prelude::ConversionError> {
                    let value = self.0;
                    let encoded_manifest_bytes = $crate::prelude::manifest_encode(&value)
                        .map_err($crate::prelude::ConversionError::EncodeError)?;
                    let manifest_value = $crate::prelude::manifest_decode::<
                        $crate::prelude::ManifestValue,
                    >(&encoded_manifest_bytes)
                    .map_err($crate::prelude::ConversionError::DecodeError)?;

                    let scrypto_value =
                        $crate::prelude::manifest_value_to_scrypto_value(manifest_value)?;
                    let encoded_scrypto_bytes = $crate::prelude::scrypto_encode(&scrypto_value)
                        .map_err($crate::prelude::ConversionError::EncodeError)?;
                    $crate::prelude::scrypto_decode::<$scrypto_ty>(&encoded_scrypto_bytes)
                        .map_err($crate::prelude::ConversionError::DecodeError)
                }
            }

            // Note: this conversion WILL PANIC if it fails.
            impl<T> From<T> for $manifest_ty_ident
            where
                T: Into<$scrypto_ty>,
            {
                fn from(value: T) -> Self {
                    Self::new(value).expect(concat!(
                        "Conversion from ",
                        stringify!($scrypto_ty),
                        " into ",
                        stringify!($manifest_ty_ident),
                        " failed despite not being expected to fail"
                    ))
                }
            }

            impl TryFrom<$manifest_ty_ident> for $scrypto_ty {
                type Error = $crate::prelude::ConversionError;

                fn try_from(value: $manifest_ty_ident) -> Result<Self, Self::Error> {
                    value.try_into_typed()
                }
            }

            impl $crate::prelude::Describe<$crate::prelude::ScryptoCustomTypeKind>
                for $manifest_ty_ident
            {
                const TYPE_ID: $crate::prelude::RustTypeId =
                    <$scrypto_ty as $crate::prelude::Describe<
                        $crate::prelude::ScryptoCustomTypeKind,
                    >>::TYPE_ID;

                fn type_data() -> $crate::prelude::TypeData<
                    $crate::prelude::ScryptoCustomTypeKind,
                    $crate::prelude::RustTypeId,
                > {
                    <$scrypto_ty as Describe<$crate::prelude::ScryptoCustomTypeKind>>::type_data()
                }

                fn add_all_dependencies(
                    aggregator: &mut $crate::prelude::TypeAggregator<$crate::prelude::ScryptoCustomTypeKind>
                ) {
                    <$scrypto_ty as Describe<$crate::prelude::ScryptoCustomTypeKind>>::add_all_dependencies(aggregator)
                }
            }
        };
    };
}

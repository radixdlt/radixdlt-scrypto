use core::marker::PhantomData;

use radix_engine_common::prelude::*;

pub trait TypeInfoMarker {
    const PACKAGE_ADDRESS: Option<PackageAddress>;
    const BLUEPRINT_NAME: &'static str;
    const OWNED_TYPE_NAME: &'static str;
    const GLOBAL_TYPE_NAME: &'static str;
}

pub struct Global<T>(pub ComponentAddress, PhantomData<T>)
where
    T: TypeInfoMarker;

pub struct Owned<T>(pub InternalAddress, PhantomData<T>)
where
    T: TypeInfoMarker;

impl<T> Global<T>
where
    T: TypeInfoMarker,
{
    pub fn new(address: ComponentAddress) -> Self {
        Self(address, PhantomData)
    }
}

impl<T> Owned<T>
where
    T: TypeInfoMarker,
{
    pub fn new(address: InternalAddress) -> Self {
        Self(address, PhantomData)
    }
}

impl<O: TypeInfoMarker> Categorize<ScryptoCustomValueKind> for Global<O> {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Reference)
    }
}

impl<O: TypeInfoMarker, E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for Global<O>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<O: TypeInfoMarker, D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for Global<O>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        ComponentAddress::decode_body_with_value_kind(decoder, value_kind)
            .map(|address| Self(address, Default::default()))
    }
}

impl<T: TypeInfoMarker> Describe<ScryptoCustomTypeKind> for Global<T> {
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1(T::GLOBAL_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Reference),
            metadata: TypeMetadata::no_child_names(T::GLOBAL_TYPE_NAME),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalTyped(
                    T::PACKAGE_ADDRESS,
                    T::BLUEPRINT_NAME.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

impl<O: TypeInfoMarker> Categorize<ScryptoCustomValueKind> for Owned<O> {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<O: TypeInfoMarker, E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for Owned<O>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<O: TypeInfoMarker, D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for Owned<O>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        InternalAddress::decode_body_with_value_kind(decoder, value_kind)
            .map(|address| Self(address, Default::default()))
    }
}

impl<T: TypeInfoMarker> Describe<ScryptoCustomTypeKind> for Owned<T> {
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1(T::OWNED_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names(T::OWNED_TYPE_NAME),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(T::PACKAGE_ADDRESS, T::BLUEPRINT_NAME.to_string()),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

macro_rules! define_type_info_marker {
    ($package_address: expr, $blueprint_name: ident) => {
        paste::paste! {
            pub struct [< $blueprint_name ObjectTypeInfo >];

            impl crate::blueprints::component::TypeInfoMarker
                for [< $blueprint_name ObjectTypeInfo >]
            {
                const PACKAGE_ADDRESS: Option<PackageAddress> = $package_address;
                const BLUEPRINT_NAME: &'static str = stringify!($blueprint_name);
                const OWNED_TYPE_NAME: &'static str = stringify!([< Owned $blueprint_name >]);
                const GLOBAL_TYPE_NAME: &'static str = stringify!([< Global $blueprint_name >]);
            }
        }
    };
}
pub(crate) use define_type_info_marker;

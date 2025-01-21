use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use radix_common::prelude::*;

pub trait TypeInfoMarker {
    const PACKAGE_ADDRESS: Option<PackageAddress>;
    const BLUEPRINT_NAME: &'static str;
    const OWNED_TYPE_NAME: &'static str;
    const GLOBAL_TYPE_NAME: &'static str;
}

// This type is added for backwards compatibility so that this change is not apparent at all to
// Scrypto Developers
pub type Global<T> = GenericGlobal<ComponentAddress, T>;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ScryptoEncode,
    ScryptoDecode,
    ScryptoCategorize,
    ManifestEncode,
    ManifestDecode,
    ManifestCategorize,
)]
#[sbor(transparent, child_types = "A")]
pub struct GenericGlobal<A, M>(pub A, #[sbor(skip)] PhantomData<M>)
where
    M: TypeInfoMarker;

impl<A, M> GenericGlobal<A, M>
where
    M: TypeInfoMarker,
{
    pub fn new(address: A) -> Self {
        Self(address, PhantomData)
    }
}

impl<A, M> Hash for GenericGlobal<A, M>
where
    A: Hash,
    M: TypeInfoMarker,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<A, M> From<A> for GenericGlobal<A, M>
where
    M: TypeInfoMarker,
{
    fn from(value: A) -> Self {
        Self(value, PhantomData)
    }
}

impl<A, M> Describe<ScryptoCustomTypeKind> for GenericGlobal<A, M>
where
    M: TypeInfoMarker,
{
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1(M::GLOBAL_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Reference),
            metadata: TypeMetadata::no_child_names(M::GLOBAL_TYPE_NAME),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalTyped(
                    M::PACKAGE_ADDRESS,
                    M::BLUEPRINT_NAME.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

// This type is added for backwards compatibility so that this change is not apparent at all to
// Scrypto Developers
pub type Owned<T> = GenericOwned<InternalAddress, T>;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ScryptoEncode,
    ScryptoDecode,
    ScryptoCategorize,
    ManifestEncode,
    ManifestDecode,
    ManifestCategorize,
)]
#[sbor(transparent, child_types = "A")]
pub struct GenericOwned<A, M>(pub A, #[sbor(skip)] PhantomData<M>)
where
    M: TypeInfoMarker;

impl<A, M> GenericOwned<A, M>
where
    M: TypeInfoMarker,
{
    pub fn new(address: A) -> Self {
        Self(address, PhantomData)
    }
}

impl<A, M> Hash for GenericOwned<A, M>
where
    A: Hash,
    M: TypeInfoMarker,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<A, M> From<A> for GenericOwned<A, M>
where
    M: TypeInfoMarker,
{
    fn from(value: A) -> Self {
        Self(value, PhantomData)
    }
}

impl<A, M> Describe<ScryptoCustomTypeKind> for GenericOwned<A, M>
where
    M: TypeInfoMarker,
{
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1(M::OWNED_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names(M::OWNED_TYPE_NAME),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(M::PACKAGE_ADDRESS, M::BLUEPRINT_NAME.to_string()),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

macro_rules! define_type_marker {
    ($package_address: expr, $blueprint_name: ident) => {
        paste::paste! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub struct [< $blueprint_name Marker >];

            impl crate::blueprints::component::TypeInfoMarker
                for [< $blueprint_name Marker >]
            {
                const PACKAGE_ADDRESS: Option<PackageAddress> = $package_address;
                const BLUEPRINT_NAME: &'static str = stringify!($blueprint_name);
                const OWNED_TYPE_NAME: &'static str = stringify!([< Owned $blueprint_name >]);
                const GLOBAL_TYPE_NAME: &'static str = stringify!([< Global $blueprint_name >]);
            }
        }
    };
}
pub(crate) use define_type_marker;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use sbor::prelude::Vec;

    pub const SOME_ADDRESS: PackageAddress =
        PackageAddress::new_or_panic([EntityType::GlobalPackage as u8; NodeId::LENGTH]);

    define_type_marker!(Some(SOME_ADDRESS), SomeType);

    #[test]
    fn global_encode_decode() {
        let addr = ComponentAddress::new_or_panic(
            [EntityType::GlobalGenericComponent as u8; NodeId::LENGTH],
        );

        let object = Global::<SomeTypeMarker>::new(addr);
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buf, 1);
        assert!(object.encode_value_kind(&mut encoder).is_ok());
        assert!(object.encode_body(&mut encoder).is_ok());

        let buf_decode = buf.into_iter().skip(1).collect::<Vec<u8>>(); // skip Global value kind, not used in decode_body_with_value_kind() decoding function

        let mut decoder = VecDecoder::<ScryptoCustomValueKind>::new(&buf_decode, 1);
        let output = Global::<SomeTypeMarker>::decode_body_with_value_kind(
            &mut decoder,
            ComponentAddress::value_kind(),
        );
        assert!(output.is_ok());

        let describe = Global::<SomeTypeMarker>::type_data();
        assert_eq!(
            describe.kind,
            TypeKind::Custom(ScryptoCustomTypeKind::Reference)
        );
        assert_eq!(
            describe.metadata.type_name.unwrap().to_string(),
            "GlobalSomeType"
        );
    }

    #[test]
    fn owned_encode_decode() {
        let addr = InternalAddress::new_or_panic(
            [EntityType::InternalGenericComponent as u8; NodeId::LENGTH],
        );

        let object = Owned::<SomeTypeMarker>::new(addr);
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buf, 1);
        assert!(object.encode_value_kind(&mut encoder).is_ok());
        assert!(object.encode_body(&mut encoder).is_ok());

        let buf_decode = buf.into_iter().skip(1).collect::<Vec<u8>>(); // skip Owned value kind, not used in decode_body_with_value_kind() decoding function

        let mut decoder = VecDecoder::<ScryptoCustomValueKind>::new(&buf_decode, 1);
        let output = Owned::<SomeTypeMarker>::decode_body_with_value_kind(
            &mut decoder,
            InternalAddress::value_kind(),
        );
        assert_eq!(output.err(), None);

        let describe = Owned::<SomeTypeMarker>::type_data();
        assert_eq!(describe.kind, TypeKind::Custom(ScryptoCustomTypeKind::Own));
        assert_eq!(
            describe.metadata.type_name.unwrap().to_string(),
            "OwnedSomeType"
        );
    }
}

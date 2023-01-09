use crate::constants::*;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for Option<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Some(v) => {
                encoder.write_discriminator(OPTION_VARIANT_SOME)?;
                encoder.write_size(1)?;
                encoder.encode(v)?;
            }
            None => {
                encoder.write_discriminator(OPTION_VARIANT_NONE)?;
                encoder.write_size(0)?;
            }
        }
        Ok(())
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Option<T> {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let discriminator = decoder.read_discriminator()?;

        match discriminator.as_ref() {
            OPTION_VARIANT_SOME => {
                decoder.read_and_check_size(1)?;
                Ok(Some(decoder.decode()?))
            }
            OPTION_VARIANT_NONE => {
                decoder.read_and_check_size(0)?;
                Ok(None)
            }
            _ => Err(DecodeError::UnknownDiscriminator(discriminator)),
        }
    }
}

#[cfg(feature = "schema")]
impl<C: CustomTypeKind<GlobalTypeId>, T: NewDescribe<C>> NewDescribe<C> for Option<T> {
    const SCHEMA_TYPE_REF: GlobalTypeId = GlobalTypeId::complex("Option", &[T::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<TypeData<C, GlobalTypeId>> {
        #[allow(unused_imports)]
        use crate::rust::borrow::ToOwned;
        Some(TypeData::named_enum(
            "Option",
            crate::rust::collections::btree_map::btreemap![
                "Some".to_owned() => TypeData::named_tuple("Some", crate::rust::vec![T::SCHEMA_TYPE_REF]),
                "None".to_owned() => TypeData::named_unit("None"),
            ],
        ))
    }

    fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
        aggregator.add_child_type_and_descendents::<T>();
    }
}

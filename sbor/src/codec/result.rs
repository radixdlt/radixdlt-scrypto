use crate::constants::*;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, Enc: Encoder<X>, T: Encode<X, Enc>, E: Encode<X, Enc>> Encode<X, Enc>
    for Result<T, E>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut Enc) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut Enc) -> Result<(), EncodeError> {
        match self {
            Ok(o) => {
                encoder.write_discriminator(RESULT_VARIANT_OK)?;
                encoder.write_size(1)?;
                encoder.encode(o)?;
            }
            Err(e) => {
                encoder.write_discriminator(RESULT_VARIANT_ERR)?;
                encoder.write_size(1)?;
                encoder.encode(e)?;
            }
        }
        Ok(())
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>, E: Decode<X, D>> Decode<X, D>
    for Result<T, E>
{
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let discriminator = decoder.read_discriminator()?;
        match discriminator.as_ref() {
            RESULT_VARIANT_OK => {
                decoder.read_and_check_size(1)?;
                Ok(Ok(decoder.decode()?))
            }
            RESULT_VARIANT_ERR => {
                decoder.read_and_check_size(1)?;
                Ok(Err(decoder.decode()?))
            }
            _ => Err(DecodeError::UnknownDiscriminator(discriminator)),
        }
    }
}

#[cfg(feature = "schema")]
impl<C: CustomTypeSchema, T: Schema<C>, E: Schema<C>> Schema<C> for Result<T, E> {
    const SCHEMA_TYPE_REF: GlobalTypeRef =
        GlobalTypeRef::complex("Result", &[T::SCHEMA_TYPE_REF, E::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Enum {
                variants: crate::rust::collections::btree_map::btreemap![
                    "Ok".to_owned() => GlobalTypeRef::complex("Ok", &[T::SCHEMA_TYPE_REF]),
                    "Err".to_owned() => GlobalTypeRef::complex("Err", &[E::SCHEMA_TYPE_REF]),
                ],
            },
            naming: TypeNaming::named_no_child_names("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type(GlobalTypeRef::complex("Ok", &[T::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Ok", vec![T::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type(GlobalTypeRef::complex("Err", &[E::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Err", vec![E::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type_and_descendents::<T>();
        aggregator.add_child_type_and_descendents::<E>();
    }
}

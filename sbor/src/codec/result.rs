use crate::constants::*;
use crate::value_kind::*;
use crate::*;

categorize_generic!(Result<T, E>, <T, E>, ValueKind::Enum);

impl<X: CustomValueKind, Enc: Encoder<X>, T: Encode<X, Enc>, E: Encode<X, Enc>> Encode<X, Enc>
    for Result<T, E>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut Enc) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
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

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D>, E: Decode<X, D>> Decode<X, D>
    for Result<T, E>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let discriminator = decoder.read_discriminator()?;
        match discriminator {
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

impl<C: CustomTypeKind<GlobalTypeId>, T: Describe<C>, E: Describe<C>> Describe<C> for Result<T, E> {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::novel("Result", &[T::TYPE_ID, E::TYPE_ID]);

    fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
        #[allow(unused_imports)]
        use crate::rust::borrow::ToOwned;
        use crate::rust::collections::*;
        Some(TypeData::enum_variants(
            "Result",
            btreemap![
                RESULT_VARIANT_OK => TypeData::no_child_names(TypeKind::Tuple {field_types: crate::rust::vec![T::TYPE_ID]}, "Ok"),
                RESULT_VARIANT_ERR => TypeData::no_child_names(TypeKind::Tuple {field_types: crate::rust::vec![E::TYPE_ID]}, "Err"),
            ],
        ))
    }

    fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
        aggregator.add_child_type_and_descendents::<T>();
        aggregator.add_child_type_and_descendents::<E>();
    }
}

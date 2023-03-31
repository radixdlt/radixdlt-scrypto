use crate::rust::mem::MaybeUninit;
use crate::value_kind::*;
use crate::*;

impl<X: CustomValueKind, T> Categorize<X> for [T] {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, T, const N: usize> Categorize<X> for [T; N] {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E> + Categorize<X>> Encode<X, E> for [T] {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(T::value_kind())?;
        encoder.write_size(self.len())?;
        if T::value_kind() == ValueKind::U8 || T::value_kind() == ValueKind::I8 {
            let ptr = self.as_ptr().cast::<u8>();
            let slice = unsafe { sbor::rust::slice::from_raw_parts(ptr, self.len()) };
            encoder.write_slice(slice)?;
        } else {
            for v in self {
                encoder.encode_deeper_body(v)?;
            }
        }
        Ok(())
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E> + Categorize<X>, const N: usize>
    Encode<X, E> for [T; N]
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }
    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_slice().encode_body(encoder)
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D> + Categorize<X>, const N: usize>
    Decode<X, D> for [T; N]
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let element_value_kind = decoder.read_and_check_value_kind(T::value_kind())?;
        decoder.read_and_check_size(N)?;

        // Please read:
        // * https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        // * https://github.com/rust-lang/rust/issues/61956
        //
        // TODO: replace with `uninit_array` and `assume_array_init` once they're stable

        // Create an uninitialized array
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Decode element by element
        for elem in &mut data[..] {
            elem.write(decoder.decode_deeper_body_with_value_kind(element_value_kind)?);
        }

        // Use &mut as an assertion of unique "ownership"
        let ptr = &mut data as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(data);

        Ok(res)
    }
}

pub use schema::*;

mod schema {
    use super::*;

    impl<C: CustomTypeKind<GlobalTypeId>, T: Describe<C>> Describe<C> for [T] {
        const TYPE_ID: GlobalTypeId = match T::TYPE_ID {
            GlobalTypeId::WellKnown([basic_well_known_types::U8_ID]) => {
                GlobalTypeId::well_known(basic_well_known_types::BYTES_ID)
            }
            _ => GlobalTypeId::novel("Array", &[T::TYPE_ID]),
        };

        fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
            match T::TYPE_ID {
                GlobalTypeId::WellKnown([basic_well_known_types::U8_ID]) => None,
                _ => Some(TypeData::new(
                    TypeKind::Array {
                        element_type: T::TYPE_ID,
                    },
                    TypeMetadata::unnamed(),
                )),
            }
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            aggregator.add_child_type_and_descendents::<T>();
        }
    }

    impl<C: CustomTypeKind<GlobalTypeId>, T: Describe<C>, const N: usize> Describe<C> for [T; N] {
        const TYPE_ID: GlobalTypeId = GlobalTypeId::novel_validated(
            "Array",
            &[T::TYPE_ID],
            &[("min", &N.to_le_bytes()), ("max", &N.to_le_bytes())],
        );

        fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
            let size = N
                .try_into()
                .expect("The array length is too large for a u32 for the SBOR schema");
            let type_metadata = match T::TYPE_ID {
                GlobalTypeId::WellKnown([basic_well_known_types::U8_ID]) => {
                    TypeMetadata::no_child_names("Bytes")
                }
                _ => TypeMetadata::unnamed(),
            };
            Some(
                TypeData::new(
                    TypeKind::Array {
                        element_type: T::TYPE_ID,
                    },
                    type_metadata,
                )
                .with_validation(TypeValidation::Array(LengthValidation {
                    min: Some(size),
                    max: Some(size),
                })),
            )
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            aggregator.add_child_type_and_descendents::<T>();
        }
    }
}

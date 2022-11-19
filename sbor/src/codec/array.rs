use crate::rust::mem::MaybeUninit;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, T: Encode<X> + TypeId<X>> Encode<X> for [T] {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(T::type_id());
        encoder.write_size(self.len());
        if T::type_id() == SborTypeId::U8 || T::type_id() == SborTypeId::I8 {
            let ptr = self.as_ptr().cast::<u8>();
            let slice = unsafe { sbor::rust::slice::from_raw_parts(ptr, self.len()) };
            encoder.write_slice(slice);
        } else {
            for v in self {
                v.encode_body(encoder);
            }
        }
    }
}

impl<X: CustomTypeId, T: Encode<X> + TypeId<X>, const N: usize> Encode<X> for [T; N] {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        self.as_slice().encode_body(encoder);
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X>, const N: usize> Decode<X, D>
    for [T; N]
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let element_type_id = decoder.read_and_check_type_id(T::type_id())?;
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
            elem.write(decoder.decode_body_with_type_id(element_type_id)?);
        }

        // Use &mut as an assertion of unique "ownership"
        let ptr = &mut data as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(data);

        Ok(res)
    }
}

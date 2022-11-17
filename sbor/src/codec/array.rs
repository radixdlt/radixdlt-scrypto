use crate::rust::mem::MaybeUninit;
use crate::rust::ptr::copy;
use crate::rust::vec::Vec;
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
            let mut buf = Vec::<u8>::with_capacity(self.len());
            unsafe {
                copy(self.as_ptr() as *mut u8, buf.as_mut_ptr(), self.len());
                buf.set_len(self.len());
            }
            encoder.write_slice(&buf);
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

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>, const N: usize> Decode<X> for [T; N] {
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let element_type_id = decoder.check_type_id(T::type_id())?;
        decoder.check_size(N)?;

        // Please read:
        // * https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        // * https://github.com/rust-lang/rust/issues/61956
        //
        // TODO: replace with `uninit_array` and `assume_array_init` once they're stable

        // Create an uninitialized array
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Decode element by element
        for elem in &mut data[..] {
            elem.write(T::decode_with_type_id(decoder, element_type_id)?);
        }

        // Use &mut as an assertion of unique "ownership"
        let ptr = &mut data as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(data);

        Ok(res)
    }
}

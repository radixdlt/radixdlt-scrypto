use core::intrinsics::copy;
use super::super::*;
use core::mem::MaybeUninit;
use crate::rust::vec::Vec;
use crate::rust::hash::Hash;
use crate::rust::collections::{HashSet, BTreeSet};

impl<T, const N: usize> Interpretation for [T; N] {
    const INTERPRETATION: u8 = DefaultInterpretations::FIXED_LENGTH_ARRAY;
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_list(self.as_ref())
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let length = decoder.read_list_type_length()?;
        if length != N {
            return Err(DecodeError::InvalidLength { expected: N, actual: length });
        }

        // Please read:
        // * https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        // * https://github.com/rust-lang/rust/issues/61956
        //
        // TODO: replace with `uninit_array` and `assume_array_init` once they're stable

        // Create an uninitialized array
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Decode element by element
        for elem in &mut data[..] {
            elem.write(T::decode_value(decoder)?);
        }

        // Use &mut as an assertion of unique "ownership"
        let ptr = &mut data as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(data);

        Ok(res)
    }
}

impl<T: Interpretation> Interpretation for Vec<T> {
    const INTERPRETATION: u8 = if T::IS_BYTE {
        DefaultInterpretations::PLAIN_RAW_BYTES
    } else {
        DefaultInterpretations::NORMAL_LIST
    };
}

impl<T: Encode> Encode for Vec<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_slice().encode_value(encoder);
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        if T::IS_BYTE {
            // TODO - Improve when Rust finally implements specialisation
            let slice = decoder.read_raw_bytes()?;
            let mut result = Vec::<T>::with_capacity(slice.len());
            unsafe {
                copy(slice.as_ptr(), result.as_mut_ptr() as *mut u8, slice.len());
                result.set_len(slice.len());
            }
            Ok(result)
        } else {
            let length = decoder.read_list_type_length()?;
            let mut result = Vec::<T>::with_capacity(if length <= 1024 { length } else { 1024 });
            for _ in 0..length {
                result.push(T::decode_value(decoder)?);
            }
            Ok(result)
        }
    }
}

impl<T: Interpretation> Interpretation for [T] {
    const INTERPRETATION: u8 = if T::IS_BYTE {
        DefaultInterpretations::PLAIN_RAW_BYTES
    } else {
        DefaultInterpretations::NORMAL_LIST
    };
}

impl<T: Encode> Encode for [T] {
    fn encode_value(&self, encoder: &mut Encoder) {
        // TODO - Improve when Rust finally implements specialisation
        if T::IS_BYTE {
            // TODO - Can do this without buf if we add a specialised encoder method to read from a raw pointer
            let mut buf = Vec::<u8>::with_capacity(self.len());
            unsafe {
                copy(self.as_ptr() as *mut u8, buf.as_mut_ptr(), self.len());
                buf.set_len(self.len());
            }
            encoder.write_raw_bytes(buf.as_ref());
        } else {
            encoder.write_list(self.as_ref())
        }
    }
}

impl<T> Interpretation for HashSet<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::UNORDERED_SET;
}

impl<T: Encode + Ord + Hash> Encode for HashSet<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        // Encode elements based on the order defined on the key type to generate deterministic bytes.
        let values: BTreeSet<&T> = self.iter().collect();
        encoder.write_list_from_iterator(values.into_iter());
    }
}

impl<T: Decode + Hash + Eq> Decode for HashSet<T> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let length = decoder.read_list_type_length()?;
        let mut result = HashSet::<T>::with_capacity(if length <= 1024 { length } else { 1024 });
        for _ in 0..length {
            if !result.insert(T::decode_value(decoder)?) {
                return Err(DecodeError::DuplicateSetEntry)
            }
        }
        Ok(result)
    }
}

impl<T> Interpretation for BTreeSet<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::SORTED_SET;
}

impl<T: Encode> Encode for BTreeSet<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_list_from_iterator(self.into_iter());
    }
}

impl<T: Decode + Hash + Ord> Decode for BTreeSet<T> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let length = decoder.read_list_type_length()?;
        let mut result = BTreeSet::<T>::new();
        for _ in 0..length {
            if !result.insert(T::decode_value(decoder)?) {
                return Err(DecodeError::DuplicateSetEntry)
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;
    use super::*;

    #[test]
    fn vec_u8() {
        let payload: Vec<u8> = vec![0, 2, 3];

        let mut buf = vec![];
        let encoder = Encoder::new(&mut buf);
        encoder.encode_payload(&payload);

        assert_eq!(
            vec![
                SBOR_V1_PREFIX_BYTE,
                DefaultInterpretations::PLAIN_RAW_BYTES, TypeEncodingClass::RAW_BYTES_U8_LENGTH,
                3,
                0, 2, 3
            ],
            buf
        )
    }

    #[test]
    fn vec_u32() {
        let payload: Vec<u32> = vec![0, 2, 3];

        let mut buf = vec![];
        let encoder = Encoder::new(&mut buf);
        encoder.encode_payload(&payload);

        assert_eq!(
            vec![
                SBOR_V1_PREFIX_BYTE,
                DefaultInterpretations::NORMAL_LIST, TypeEncodingClass::LIST_U8_LENGTH,
                3,
                DefaultInterpretations::U32, TypeEncodingClass::RAW_BYTES_U8_LENGTH, 4, 0, 0, 0, 0,
                DefaultInterpretations::U32, TypeEncodingClass::RAW_BYTES_U8_LENGTH, 4, 2, 0, 0, 0,
                DefaultInterpretations::U32, TypeEncodingClass::RAW_BYTES_U8_LENGTH, 4, 3, 0, 0, 0,
            ],
            buf
        )
    }
}

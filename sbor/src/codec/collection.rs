use crate::rust::collections::*;
use crate::rust::hash::Hash;
use crate::rust::ptr::copy;
use crate::rust::vec::Vec;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, T: Encode<X> + TypeId<X>> Encode<X> for Vec<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        self.as_slice().encode_body(encoder);
    }
}

impl<X: CustomTypeId, T: Encode<X> + TypeId<X>> Encode<X> for BTreeSet<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(T::type_id());
        encoder.write_size(self.len());
        for v in self {
            v.encode_body(encoder);
        }
    }
}

impl<X: CustomTypeId, T: Encode<X> + TypeId<X> + Ord + Hash> Encode<X> for HashSet<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(T::type_id());
        encoder.write_size(self.len());
        for v in self {
            v.encode_body(encoder);
        }
    }
}

impl<X: CustomTypeId, K: Encode<X> + TypeId<X>, V: Encode<X> + TypeId<X>> Encode<X>
    for BTreeMap<K, V>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(<(K, V)>::type_id());
        encoder.write_size(self.len());
        for (k, v) in self {
            encoder.write_size(2);
            k.encode(encoder);
            v.encode(encoder);
        }
    }
}

impl<X: CustomTypeId, K: Encode<X> + TypeId<X> + Ord + Hash, V: Encode<X> + TypeId<X>> Encode<X>
    for HashMap<K, V>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(<(K, V)>::type_id());
        encoder.write_size(self.len());
        let keys: BTreeSet<&K> = self.keys().collect();
        for key in keys {
            encoder.write_size(2);
            key.encode(encoder);
            self.get(key).unwrap().encode(encoder);
        }
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X>> Decode<X, D> for Vec<T> {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let element_type_id = decoder.read_and_check_type_id(T::type_id())?;
        let len = decoder.read_size()?;

        if T::type_id() == SborTypeId::U8 || T::type_id() == SborTypeId::I8 {
            let slice = decoder.read_slice(len)?; // length is checked here
            let mut result = Vec::<T>::with_capacity(len);
            unsafe {
                copy(slice.as_ptr(), result.as_mut_ptr() as *mut u8, slice.len());
                result.set_len(slice.len());
            }
            Ok(result)
        } else {
            let mut result = Vec::<T>::with_capacity(if len <= 1024 { len } else { 1024 });
            for _ in 0..len {
                result.push(decoder.decode_body_with_type_id(element_type_id)?);
            }
            Ok(result)
        }
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X> + Ord> Decode<X, D>
    for BTreeSet<T>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let elements: Vec<T> = Vec::<T>::decode_body_with_type_id(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X> + Hash + Eq> Decode<X, D>
    for HashSet<T>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let elements: Vec<T> = Vec::<T>::decode_body_with_type_id(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

impl<
        X: CustomTypeId,
        D: Decoder<X>,
        K: Decode<X, D> + TypeId<X> + Ord,
        V: Decode<X, D> + TypeId<X>,
    > Decode<X, D> for BTreeMap<K, V>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let elements = Vec::<(K, V)>::decode_body_with_type_id(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

impl<
        X: CustomTypeId,
        D: Decoder<X>,
        K: Decode<X, D> + TypeId<X> + Hash + Eq,
        V: Decode<X, D> + TypeId<X>,
    > Decode<X, D> for HashMap<K, V>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let elements: Vec<(K, V)> = Vec::<(K, V)>::decode_body_with_type_id(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

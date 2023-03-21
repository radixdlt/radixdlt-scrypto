use crate::rust::collections::*;
use crate::rust::hash::Hash;
use crate::rust::ptr::copy;
use crate::rust::vec::Vec;
use crate::value_kind::*;
use crate::*;

categorize_generic!(Vec<T>, <T>, ValueKind::Array);
categorize_generic!(BTreeSet<T>, <T>, ValueKind::Array);
categorize_generic!(HashSet<T>, <T>, ValueKind::Array);
categorize_generic!(IndexSet<T>, <T>, ValueKind::Array);

categorize_generic!(BTreeMap<K, V>, <K, V>, ValueKind::Map);
categorize_generic!(HashMap<K, V>, <K, V>, ValueKind::Map);
categorize_generic!(IndexMap<K, V>, <K, V>, ValueKind::Map);

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E> + Categorize<X>> Encode<X, E> for Vec<T> {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_slice().encode_body(encoder)?;
        Ok(())
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E> + Categorize<X>> Encode<X, E>
    for BTreeSet<T>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(T::value_kind())?;
        encoder.write_size(self.len())?;
        for v in self {
            encoder.encode_deeper_body(v)?;
        }
        Ok(())
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E> + Categorize<X> + Ord + Hash> Encode<X, E>
    for HashSet<T>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(T::value_kind())?;
        encoder.write_size(self.len())?;
        let set: BTreeSet<&T> = self.iter().collect();
        for v in set {
            encoder.encode_deeper_body(v)?;
        }
        Ok(())
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E> + Categorize<X> + Hash> Encode<X, E>
    for IndexSet<T>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(T::value_kind())?;
        encoder.write_size(self.len())?;
        for v in self {
            encoder.encode_deeper_body(v)?;
        }
        Ok(())
    }
}

impl<
        X: CustomValueKind,
        E: Encoder<X>,
        K: Encode<X, E> + Categorize<X>,
        V: Encode<X, E> + Categorize<X>,
    > Encode<X, E> for BTreeMap<K, V>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(K::value_kind())?;
        encoder.write_value_kind(V::value_kind())?;
        encoder.write_size(self.len())?;
        for (k, v) in self {
            encoder.encode_deeper_body(k)?;
            encoder.encode_deeper_body(v)?;
        }
        Ok(())
    }
}

impl<
        X: CustomValueKind,
        E: Encoder<X>,
        K: Encode<X, E> + Categorize<X> + Ord + Hash,
        V: Encode<X, E> + Categorize<X>,
    > Encode<X, E> for HashMap<K, V>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(K::value_kind())?;
        encoder.write_value_kind(V::value_kind())?;
        encoder.write_size(self.len())?;
        let mut keys: Vec<&K> = self.keys().collect();
        keys.sort();
        for key in keys {
            encoder.encode_deeper_body(key)?;
            encoder.encode_deeper_body(self.get(key).unwrap())?;
        }
        Ok(())
    }
}

impl<
        X: CustomValueKind,
        E: Encoder<X>,
        K: Encode<X, E> + Categorize<X> + Hash + Eq + PartialEq,
        V: Encode<X, E> + Categorize<X>,
    > Encode<X, E> for IndexMap<K, V>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(K::value_kind())?;
        encoder.write_value_kind(V::value_kind())?;
        encoder.write_size(self.len())?;
        for (key, value) in self.iter() {
            encoder.encode_deeper_body(key)?;
            encoder.encode_deeper_body(value)?;
        }
        Ok(())
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D> + Categorize<X>> Decode<X, D> for Vec<T> {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let element_value_kind = decoder.read_and_check_value_kind(T::value_kind())?;
        let len = decoder.read_size()?;

        if T::value_kind() == ValueKind::U8 || T::value_kind() == ValueKind::I8 {
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
                result.push(decoder.decode_deeper_body_with_value_kind(element_value_kind)?);
            }
            Ok(result)
        }
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D> + Categorize<X> + Ord> Decode<X, D>
    for BTreeSet<T>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let elements: Vec<T> = Vec::<T>::decode_body_with_value_kind(decoder, value_kind)?;
        Ok(elements.into_iter().collect())
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D> + Categorize<X> + Hash + Eq> Decode<X, D>
    for HashSet<T>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let elements: Vec<T> = Vec::<T>::decode_body_with_value_kind(decoder, value_kind)?;
        Ok(elements.into_iter().collect())
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D> + Categorize<X> + Hash + Eq> Decode<X, D>
    for IndexSet<T>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let element_value_kind = decoder.read_and_check_value_kind(T::value_kind())?;
        let len = decoder.read_size()?;
        let mut result = index_set_with_capacity(if len <= 1024 { len } else { 1024 });
        for _ in 0..len {
            result.insert(decoder.decode_deeper_body_with_value_kind(element_value_kind)?);
        }
        Ok(result)
    }
}

impl<
        X: CustomValueKind,
        D: Decoder<X>,
        K: Decode<X, D> + Categorize<X> + Ord,
        V: Decode<X, D> + Categorize<X>,
    > Decode<X, D> for BTreeMap<K, V>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let key_value_kind = decoder.read_and_check_value_kind(K::value_kind())?;
        let value_value_kind = decoder.read_and_check_value_kind(V::value_kind())?;
        let size = decoder.read_size()?;
        let mut map = BTreeMap::new();
        for _ in 0..size {
            map.insert(
                decoder.decode_deeper_body_with_value_kind(key_value_kind)?,
                decoder.decode_deeper_body_with_value_kind(value_value_kind)?,
            );
        }
        Ok(map)
    }
}

impl<
        X: CustomValueKind,
        D: Decoder<X>,
        K: Decode<X, D> + Categorize<X> + Hash + Eq,
        V: Decode<X, D> + Categorize<X>,
    > Decode<X, D> for HashMap<K, V>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let key_value_kind = decoder.read_and_check_value_kind(K::value_kind())?;
        let value_value_kind = decoder.read_and_check_value_kind(V::value_kind())?;
        let size = decoder.read_size()?;
        let mut map = HashMap::with_capacity(if size <= 1024 { size } else { 1024 });
        for _ in 0..size {
            map.insert(
                decoder.decode_deeper_body_with_value_kind(key_value_kind)?,
                decoder.decode_deeper_body_with_value_kind(value_value_kind)?,
            );
        }
        Ok(map)
    }
}

impl<
        X: CustomValueKind,
        D: Decoder<X>,
        K: Decode<X, D> + Categorize<X> + Hash + Eq,
        V: Decode<X, D> + Categorize<X>,
    > Decode<X, D> for IndexMap<K, V>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let key_value_kind = decoder.read_and_check_value_kind(K::value_kind())?;
        let value_value_kind = decoder.read_and_check_value_kind(V::value_kind())?;
        let size = decoder.read_size()?;
        let mut map = index_map_with_capacity(if size <= 1024 { size } else { 1024 });
        for _ in 0..size {
            map.insert(
                decoder.decode_deeper_body_with_value_kind(key_value_kind)?,
                decoder.decode_deeper_body_with_value_kind(value_value_kind)?,
            );
        }
        Ok(map)
    }
}

pub use schema::*;

mod schema {
    use super::*;

    wrapped_generic_describe!(T, Vec<T>, [T]);

    impl<C: CustomTypeKind<GlobalTypeId>, T: Describe<C>> Describe<C> for BTreeSet<T> {
        const TYPE_ID: GlobalTypeId = GlobalTypeId::novel("Set", &[T::TYPE_ID]);

        fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
            Some(TypeData::new(
                TypeKind::Array {
                    element_type: T::TYPE_ID,
                },
                TypeMetadata::unnamed(),
            ))
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            aggregator.add_child_type_and_descendents::<T>();
        }
    }

    wrapped_generic_describe!(T, HashSet<T>, BTreeSet<T>);
    wrapped_generic_describe!(T, IndexSet<T>, BTreeSet<T>);

    impl<C: CustomTypeKind<GlobalTypeId>, K: Describe<C>, V: Describe<C>> Describe<C>
        for BTreeMap<K, V>
    {
        const TYPE_ID: GlobalTypeId = GlobalTypeId::novel("Map", &[K::TYPE_ID, V::TYPE_ID]);

        fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
            Some(TypeData::new(
                TypeKind::Map {
                    key_type: K::TYPE_ID,
                    value_type: V::TYPE_ID,
                },
                TypeMetadata::unnamed(),
            ))
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            aggregator.add_child_type_and_descendents::<K>();
            aggregator.add_child_type_and_descendents::<V>();
        }
    }

    wrapped_double_generic_describe!(K, V, HashMap<K, V>, BTreeMap<K, V>);
    wrapped_double_generic_describe!(K, V, IndexMap<K, V>, BTreeMap<K, V>);
}

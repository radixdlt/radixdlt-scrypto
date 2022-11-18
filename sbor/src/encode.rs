use crate::rust::marker::PhantomData;
use crate::rust::vec::Vec;
use crate::type_id::*;

/// A data structure that can be serialized into a byte array using SBOR.
pub trait Encode<X: CustomTypeId> {
    /// Encodes this object into the byte buffer encapsulated by the given encoder.
    fn encode(&self, encoder: &mut Encoder<X>) {
        self.encode_type_id(encoder);
        self.encode_body(encoder);
    }

    fn encode_type_id(&self, encoder: &mut Encoder<X>);

    fn encode_body(&self, encoder: &mut Encoder<X>);
}

/// An `Encoder` abstracts the logic for writing core types into a byte buffer.
pub struct Encoder<'a, X: CustomTypeId> {
    buf: &'a mut Vec<u8>,
    phantom: PhantomData<X>,
}

impl<'a, X: CustomTypeId> Encoder<'a, X> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self {
            buf,
            phantom: PhantomData,
        }
    }

    pub fn write_type_id(&mut self, ty: SborTypeId<X>) {
        self.buf.push(ty.as_u8());
    }

    pub fn write_discriminator(&mut self, discriminator: &str) {
        self.write_size(discriminator.len());
        self.write_slice(discriminator.as_bytes());
    }

    pub fn write_size(&mut self, len: usize) {
        self.buf.extend(&(len as u32).to_le_bytes());
    }

    pub fn write_byte(&mut self, n: u8) {
        self.buf.push(n);
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buf.extend(slice);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::borrow::ToOwned;
    use crate::rust::boxed::Box;
    use crate::rust::collections::*;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::NoCustomTypeId;

    fn do_encoding<X: CustomTypeId>(enc: &mut Encoder<X>) {
        ().encode(enc);
        true.encode(enc);
        1i8.encode(enc);
        1i16.encode(enc);
        1i32.encode(enc);
        1i64.encode(enc);
        1i128.encode(enc);
        1u8.encode(enc);
        1u16.encode(enc);
        1u32.encode(enc);
        1u64.encode(enc);
        1u128.encode(enc);
        "hello".encode(enc);

        [1u32, 2u32, 3u32].encode(enc);
        (1u32, 2u32).encode(enc);

        vec![1u32, 2u32, 3u32].encode(enc);
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        set.encode(enc);
        let mut map = BTreeMap::<u8, u8>::new();
        map.insert(1, 2);
        map.insert(3, 4);
        map.encode(enc);

        Some(1u32).encode(enc);
        Option::<u32>::None.encode(enc);
        Result::<u32, String>::Ok(1u32).encode(enc);
        Result::<u32, String>::Err("hello".to_owned()).encode(enc);
    }

    #[test]
    pub fn test_encoding() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::<NoCustomTypeId>::new(&mut bytes);
        do_encoding(&mut enc);

        assert_eq!(
            vec![
                0, 0, // unit
                1, 1, // bool
                2, 1, // i8
                3, 1, 0, // i16
                4, 1, 0, 0, 0, // i32
                5, 1, 0, 0, 0, 0, 0, 0, 0, // i64
                6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // i128
                7, 1, // u8
                8, 1, 0, // u16
                9, 1, 0, 0, 0, // u32
                10, 1, 0, 0, 0, 0, 0, 0, 0, // u64
                11, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // u128
                12, 5, 0, 0, 0, 104, 101, 108, 108, 111, // string
                32, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
                33, 2, 0, 0, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
                32, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vec
                32, 7, 2, 0, 0, 0, 1, 2, // set
                32, 33, 2, 0, 0, 0, 2, 0, 0, 0, 7, 1, 7, 2, 2, 0, 0, 0, 7, 3, 7, 4, // map
                17, 4, 0, 0, 0, 83, 111, 109, 101, 1, 0, 0, 0, 9, 1, 0, 0, 0, // Some<T>
                17, 4, 0, 0, 0, 78, 111, 110, 101, 0, 0, 0, 0, // None
                17, 2, 0, 0, 0, 79, 107, 1, 0, 0, 0, 9, 1, 0, 0, 0, // Ok<T>
                17, 3, 0, 0, 0, 69, 114, 114, 1, 0, 0, 0, 12, 5, 0, 0, 0, 104, 101, 108, 108,
                111, // Err<T>
            ],
            bytes
        );
    }

    #[test]
    pub fn test_encode_box() {
        let x = Box::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::<NoCustomTypeId>::new(&mut bytes);
        x.encode(&mut enc);
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_rc() {
        let x = crate::rust::rc::Rc::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::<NoCustomTypeId>::new(&mut bytes);
        x.encode(&mut enc);
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_ref_cell() {
        let x = crate::rust::cell::RefCell::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::<NoCustomTypeId>::new(&mut bytes);
        x.encode(&mut enc);
        assert_eq!(bytes, vec![7, 5])
    }
}

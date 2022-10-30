use crate::rust::vec::Vec;

use super::*;

// TODO - turn Encoder into a trait

/// An `Encoder` abstracts the logic for writing core types into a byte buffer.
pub struct Encoder<'a> {
    buf: &'a mut Vec<u8>,
    encoder_stack_depth: u8,
}

impl<'a> Encoder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self {
            buf,
            encoder_stack_depth: 0,
        }
    }

    /// For encoding a full payload
    pub fn encode_payload<T: Encodable>(mut self, value: &T) {
        self.write_u8(SBOR_V1_PREFIX_BYTE);
        self.encode(value);
    }

    /// For encoding the type to the encoder, as a partial encoding of a larger payload
    pub fn encode<T: Encodable>(&mut self, value: &T) {
        self.write_interpretation(value.interpretation());
        value.encode_value_to(self);
    }

    #[inline]
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) {
        let sized_length = SizedLength::from_size(bytes.len());
        self.write_u8(sized_length.list_encoding_id());
        self.write_length(sized_length);
        self.write_bytes(bytes);
    }

    #[inline]
    pub fn write_product_type_header_u8_length(&mut self, length: u8) {
        self.write_u8(ProductTypeLength::U8.encoding_id());
        self.write_u8(length);
    }

    #[inline]
    pub fn write_product_type_header_u16_length(&mut self, length: u16) {
        self.write_u8(ProductTypeLength::U16.encoding_id());
        self.write_u16(length);
    }

    #[inline]
    pub fn write_sum_type_u8_discriminator<T: Encodable>(
        &mut self,
        discriminator: u8,
        value: &T,
    ) {
        self.write_u8(SumTypeDiscriminator::U8.encoding_id());
        self.write_u8(discriminator);
        self.track_encode_depth_increase();
        self.encode(value);
        self.track_encode_depth_decrease();
    }

    #[inline]
    pub fn write_sum_type_u16_discriminator<T: Encodable>(
        &mut self,
        discriminator: u16,
        value: &T,
    ) {
        self.write_u8(SumTypeDiscriminator::U16.encoding_id());
        self.write_u16(discriminator);
        self.track_encode_depth_increase();
        self.encode(value);
        self.track_encode_depth_decrease();
    }

    #[inline]
    pub fn write_sum_type_u32_discriminator<T: Encodable>(
        &mut self,
        discriminator: u32,
        value: &T,
    ) {
        self.write_u8(SumTypeDiscriminator::U32.encoding_id());
        self.write_u32(discriminator);
        self.track_encode_depth_increase();
        self.encode(value);
        self.track_encode_depth_decrease();
    }

    #[inline]
    pub fn write_sum_type_u64_discriminator<T: Encodable>(
        &mut self,
        discriminator: u64,
        value: &T,
    ) {
        self.write_u8(SumTypeDiscriminator::U64.encoding_id());
        self.write_u64(discriminator);
        self.track_encode_depth_increase();
        self.encode(value);
        self.track_encode_depth_decrease();
    }

    #[inline]
    pub fn write_sum_type_any_discriminator<D: Encodable, V: Encodable>(
        &mut self,
        discriminator: &D,
        value: &V,
    ) {
        self.write_u8(SumTypeDiscriminator::Any.encoding_id());
        self.track_encode_depth_increase();
        self.encode(discriminator);
        self.encode(value);
        self.track_encode_depth_decrease();
    }

    #[inline]
    pub fn write_list<T: Encode>(&mut self, list: &[T]) {
        let sized_length = SizedLength::from_size(list.len());
        self.write_u8(sized_length.list_encoding_id());
        self.write_length(sized_length);
        self.track_encode_depth_increase();
        for value in list {
            self.encode(value);
        }
        self.track_encode_depth_decrease();
    }

    #[inline]
    pub fn write_map<TKey: Encodable, TValue: Encodable>(
        &mut self,
        map: &[(TKey, TValue)],
    ) {
        let sized_length = SizedLength::from_size(map.len());
        self.write_u8(sized_length.map_encoding_id());
        self.write_length(sized_length);
        self.track_encode_depth_increase();
        for (key, value) in map {
            self.encode(key);
            self.encode(value);
        }
        self.track_encode_depth_decrease();
    }

    #[inline]
    fn write_interpretation(&mut self, interpretation: u8) {
        self.write_u8(interpretation);
    }

    #[inline]
    fn write_length(&mut self, sized_length: SizedLength) {
        match sized_length {
            SizedLength::U8(len) => self.write_u8(len),
            SizedLength::U16(len) => self.write_u16(len),
            SizedLength::U32(len) => self.write_u32(len),
            SizedLength::U64(len) => self.write_u64(len),
        };
    }

    #[inline]
    fn write_u8(&mut self, val: u8) {
        self.buf.push(val);
    }

    #[inline]
    fn write_u16(&mut self, val: u16) {
        self.buf.extend(val.to_le_bytes());
    }

    #[inline]
    fn write_u32(&mut self, val: u32) {
        self.buf.extend(val.to_le_bytes());
    }

    #[inline]
    fn write_u64(&mut self, val: u64) {
        self.buf.extend(val.to_le_bytes());
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    fn track_encode_depth_increase(&mut self) {
        // TODO - add encode error to handle this!
        // TODO - add this to the decoder as well!
        self.encoder_stack_depth += 1;
        if self.encoder_stack_depth > DEFAULT_MAX_ENCODING_DEPTH {
            panic!("Max encoding depth reached encoding SBOR");
        }
    }

    #[inline]
    fn track_encode_depth_decrease(&mut self) {
        self.encoder_stack_depth -= 1;
    }
}

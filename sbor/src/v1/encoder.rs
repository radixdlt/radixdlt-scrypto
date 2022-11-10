use crate::rust::vec::Vec;

use super::*;

/// Represents an error occurred during encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    MaxDepthExceeded(u32),
    ProductTooLarge(usize),
}

/// An `Encoder` abstracts the logic for writing core types.
pub trait Encoder: Sized {
    /// For encoding a full payload
    fn encode_payload<T: Encode<Self> + ?Sized>(mut self, value: &T) -> Result<(), EncodeError> {
        self.append_u8(SBOR_V1_PREFIX_BYTE)?;
        self.encode(value)?;
        Ok(())
    }

    /// For encoding the type to the encoder, as a partial encoding of a larger payload
    ///
    /// In general, it is recommended that only this method is not marked as inlined -
    /// the other functions making up the encoding process can be. This will then be
    /// monomorphized for each concrete Encode type.
    fn encode<T: Encode<Self> + ?Sized>(&mut self, value: &T) -> Result<(), EncodeError> {
        self.track_encode_depth_increase()?;
        self.write_interpretation(value.get_interpretation())?;
        value.encode_value(self)?;
        self.track_encode_depth_decrease()?;
        Ok(())
    }

    #[inline]
    fn write_raw_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let sized_length = SizedLength::from_size(bytes.len());
        self.append_u8(sized_length.raw_bytes_encoding_id())?;
        self.write_length(sized_length)?;
        self.append_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn write_product_type_header_u8_length(&mut self, length: u8) -> Result<(), EncodeError> {
        self.append_u8(ProductTypeLength::U8.encoding_id())?;
        self.append_u8(length)?;
        Ok(())
    }

    #[inline]
    fn write_product_type_header_u16_length(&mut self, length: u16) -> Result<(), EncodeError> {
        self.append_u8(ProductTypeLength::U16.encoding_id())?;
        self.append_u16(length)?;
        Ok(())
    }

    #[inline]
    fn write_sum_type_u8_discriminator_header(
        &mut self,
        discriminator: u8,
    ) -> Result<(), EncodeError> {
        self.append_u8(SumTypeDiscriminator::U8.encoding_id())?;
        self.append_u8(discriminator)?;
        Ok(())
    }

    #[inline]
    fn write_sum_type_u16_discriminator_header(
        &mut self,
        discriminator: u16,
    ) -> Result<(), EncodeError> {
        self.append_u8(SumTypeDiscriminator::U16.encoding_id())?;
        self.append_u16(discriminator)?;
        Ok(())
    }

    #[inline]
    fn write_sum_type_u32_discriminator_header(
        &mut self,
        discriminator: u32,
    ) -> Result<(), EncodeError> {
        self.append_u8(SumTypeDiscriminator::U32.encoding_id())?;
        self.append_u32(discriminator)?;
        Ok(())
    }

    #[inline]
    fn write_sum_type_u64_discriminator_header(
        &mut self,
        discriminator: u64,
    ) -> Result<(), EncodeError> {
        self.append_u8(SumTypeDiscriminator::U64.encoding_id())?;
        self.append_u64(discriminator)?;
        Ok(())
    }

    #[inline]
    fn write_sum_type_any_discriminator_header<D: Encode<Self> + ?Sized>(
        &mut self,
        discriminator: &D,
    ) -> Result<(), EncodeError> {
        self.append_u8(SumTypeDiscriminator::Any.encoding_id())?;
        self.encode(discriminator)?;
        Ok(())
    }

    #[inline]
    fn write_list_from_slice<T: Encode<Self>>(&mut self, list: &[T]) -> Result<(), EncodeError> {
        let sized_length = SizedLength::from_size(list.len());
        self.append_u8(sized_length.list_encoding_id())?;
        self.write_length(sized_length)?;
        for value in list {
            self.encode(value)?;
        }
        Ok(())
    }

    #[inline]
    fn write_list_from_iterator<'t, 's, I: ExactSizeIterator<Item = &'s T>, T: Encode<Self> + ?Sized + 's>(
        &'t mut self, 
        sized_iterator: I,
    ) -> Result<(), EncodeError> {
        let sized_length = SizedLength::from_size(sized_iterator.len());
        self.append_u8(sized_length.list_encoding_id())?;
        self.write_length(sized_length)?;
        for value in sized_iterator {
            self.encode(value)?;
        }
        Ok(())
    }

    #[inline]
    fn write_map_from_slice<TKey: Encode<Self>, TValue: Encode<Self>>(
        &mut self,
        map: &[(TKey, TValue)],
    ) -> Result<(), EncodeError> {
        let sized_length = SizedLength::from_size(map.len());
        self.append_u8(sized_length.map_encoding_id())?;
        self.write_length(sized_length)?;
        for (key, value) in map {
            self.encode(key)?;
            self.encode(value)?;
        }
        Ok(())
    }

    #[inline]
    fn write_map_from_iterator<'t, 's, I: ExactSizeIterator<Item = (&'s TKey, &'s TValue)>, TKey: Encode<Self> + ?Sized + 's, TValue: Encode<Self> + ?Sized + 's>(
        &mut self,
        sized_iterator: I,
    ) -> Result<(), EncodeError> {
        let sized_length = SizedLength::from_size(sized_iterator.len());
        self.append_u8(sized_length.map_encoding_id())?;
        self.write_length(sized_length)?;
        for (key, value) in sized_iterator {
            self.encode(key)?;
            self.encode(value)?;
        }
        Ok(())
    }

    #[inline]
    fn write_interpretation(&mut self, interpretation: u8) -> Result<(), EncodeError> {
        self.append_u8(interpretation)
    }

    #[inline]
    fn write_length(&mut self, sized_length: SizedLength) -> Result<(), EncodeError> {
        match sized_length {
            SizedLength::U8(len) => self.append_u8(len),
            SizedLength::U16(len) => self.append_u16(len),
            SizedLength::U32(len) => self.append_u32(len),
            SizedLength::U64(len) => self.append_u64(len),
        }
    }

    #[inline]
    fn append_u8(&mut self, val: u8) -> Result<(), EncodeError> {
        self.append_byte(val)
    }

    #[inline]
    fn append_u16(&mut self, val: u16) -> Result<(), EncodeError> {
        self.append_bytes(&val.to_le_bytes())
    }

    #[inline]
    fn append_u32(&mut self, val: u32) -> Result<(), EncodeError> {
        self.append_bytes(&val.to_le_bytes())
    }

    #[inline]
    fn append_u64(&mut self, val: u64) -> Result<(), EncodeError> {
        self.append_bytes(&val.to_le_bytes())
    }

    fn append_byte(&mut self, val: u8) -> Result<(), EncodeError>;

    fn append_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;

    fn track_encode_depth_increase(&mut self) -> Result<(), EncodeError>;

    fn track_encode_depth_decrease(&mut self) -> Result<(), EncodeError>;
}

pub struct VecEncoder<'a> {
    buf: &'a mut Vec<u8>,
    encoder_stack_depth: u32,
}

impl<'a> VecEncoder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self {
            buf,
            encoder_stack_depth: 0,
        }
    }
}

impl<'a> Encoder for VecEncoder<'a> {
    #[inline]
    fn append_byte(&mut self, val: u8) -> Result<(), EncodeError> {
        self.buf.push(val);
        Ok(())
    }

    #[inline]
    fn append_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    #[inline]
    fn track_encode_depth_increase(&mut self) -> Result<(), EncodeError> {
        self.encoder_stack_depth += 1;
        if self.encoder_stack_depth > DEFAULT_MAX_ENCODING_DEPTH {
            return Err(EncodeError::MaxDepthExceeded(DEFAULT_MAX_ENCODING_DEPTH));
        }
        Ok(())
    }

    #[inline]
    fn track_encode_depth_decrease(&mut self) -> Result<(), EncodeError> {
        self.encoder_stack_depth -= 1;
        Ok(())
    }
}

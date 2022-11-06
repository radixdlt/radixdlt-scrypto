use super::*;


/// This type is intended to represent any SBOR value
pub struct Value {
    pub interpretation: u8,
    pub content: ValueContent,
}

impl Interpretation for Value {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    fn get_interpretation(&self) -> u8 {
        self.interpretation
    }

    fn check_interpretation(_actual: u8) -> Result<(), DecodeError> {
        Ok(())
    }
}

impl <E: Encoder> Encode<E> for Value {
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match &self.content {
            ValueContent::RawBytes { bytes } => {
                encoder.write_raw_bytes(bytes)?;
            }
            ValueContent::Product { values } => {
                let length = values.len();
                if length <= 255 {
                    encoder.write_product_type_header_u8_length(length as u8)?;
                } else if length <= u16::MAX as usize {
                    encoder.write_product_type_header_u16_length(length as u16)?;
                } else {
                    return Err(EncodeError::ProductTooLarge(length));
                }
                for value in values {
                    encoder.encode(value)?;
                }
            },
            ValueContent::List { items } => {
                encoder.write_list_from_slice(items)?;
            },
            ValueContent::Map { entries } => {
                encoder.write_map_from_slice(entries)?;
            },
            ValueContent::Sum { discriminator, value } => {
                match discriminator {
                    Discriminator::U8(d) => encoder.write_sum_type_u8_discriminator(*d, value)?,
                    Discriminator::U16(d) => encoder.write_sum_type_u16_discriminator(*d, value)?,
                    Discriminator::U32(d) => encoder.write_sum_type_u32_discriminator(*d, value)?,
                    Discriminator::U64(d) => encoder.write_sum_type_u64_discriminator(*d, value)?,
                    Discriminator::Any(d) => encoder.write_sum_type_any_discriminator(d, value)?,
                }
            },
        }
        Ok(())
    }
}

impl <D: Decoder> Decode<D> for Value {
    fn decode_value_with_interpretation(decoder: &mut D, read_interpretation: u8) -> Result<Self, DecodeError> {
        let encoding_class = decoder.read_type_encoding_class()?;
        let content = match encoding_class {
            TypeEncodingClass::RawBytes(length_type) => {
                let length = decoder.read_length(length_type)?;
                let bytes = decoder.consume_variable_bytes(length)?;
                ValueContent::RawBytes { bytes: bytes.to_vec() }
            },
            TypeEncodingClass::ProductType(length_type) => {
                let length = match length_type {
                    ProductTypeLength::U8 => decoder.read_u8()? as usize,
                    ProductTypeLength::U16 => decoder.read_u16()? as usize,
                };
                let mut values = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    values.push(decoder.decode()?);
                }
                ValueContent::Product { values }
            },
            TypeEncodingClass::SumType(discriminator_type) => {
                let discriminator = match discriminator_type {
                    SumTypeDiscriminator::U8 => Discriminator::U8(decoder.read_u8()?),
                    SumTypeDiscriminator::U16 => Discriminator::U16(decoder.read_u16()?),
                    SumTypeDiscriminator::U32 => Discriminator::U32(decoder.read_u32()?),
                    SumTypeDiscriminator::U64 => Discriminator::U64(decoder.read_u64()?),
                    SumTypeDiscriminator::Any => Discriminator::Any(Box::new(decoder.decode()?)),
                };
                let value = decoder.decode()?;
                ValueContent::Sum { discriminator, value }
            },
            TypeEncodingClass::List(length_type) => {
                let length = decoder.read_length(length_type)?;
                let mut items = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    items.push(decoder.decode()?);
                }
                ValueContent::List { items }
            },
            TypeEncodingClass::Map(length_type) => {
                let length = decoder.read_length(length_type)?;
                let mut entries = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    entries.push((decoder.decode()?, decoder.decode()?));
                }
                ValueContent::Map { entries }
            },
        };
        let value = Value {
            interpretation: read_interpretation,
            content
        };
        Ok(value)
    }

    fn decode_value(_decoder: &mut D) -> Result<Self, DecodeError> {
        panic!("Implemented decode_value_with_interpretation instead")
    }
}

pub enum ValueContent {
    RawBytes {
        bytes: Vec<u8>
    },
    Product {
        values: Vec<Value>
    },
    List {
        items: Vec<Value>
    },
    Map {
        entries: Vec<(Value, Value)>
    },
    Sum {
        discriminator: Discriminator,
        value: Box<Value>,
    },
}

pub enum Discriminator {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Any(Box<Value>),
}
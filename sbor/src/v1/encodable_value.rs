use super::*;

/// This type is intended to enable efficient dynamic serialization of an SBOR payload
/// from references to partial objects.
/// 
/// Unlike Value, it can store references and other items which only implement Encode.
/// It can also embed any encodable, not just other Values.
/// 
/// For solving a similar problem in typed SBOR, you can use the Cow smart pointer.
/// In typed SBOR, you need a single object to support both serialization and deserialization,
/// and the use of a Cow enables this.
pub struct EncodableValue<E: Encoder> {
    pub interpretation: u8,
    pub content: EncodableValueContent<E>,
}

impl <E: Encoder> Interpretation for EncodableValue<E> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    fn get_interpretation(&self) -> u8 {
        self.interpretation
    }

    fn check_interpretation(_actual: u8) -> Result<(), DecodeError> {
        Ok(())
    }
}

impl <E: Encoder> Encode<E> for EncodableValue<E> {
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match &self.content {
            // NOTE:
            // It would be normal to bound on Encode + Implementation - but a trait object
            // can't have associated types (which Implementation has), so we only just have the
            // Encode bound. This means that we can't use blanket impls such as on Vec<T> and
            // Box<T>, and have to hand-roll things a little more. 
            EncodableValueContent::RawBytes { bytes } => {
                encoder.write_raw_bytes(bytes)?;
            }
            EncodableValueContent::Product { values } => {
                let length = values.len();
                if length <= 255 {
                    encoder.write_product_type_header_u8_length(length as u8)?;
                } else if length <= u16::MAX as usize {
                    encoder.write_product_type_header_u16_length(length as u16)?;
                } else {
                    return Err(EncodeError::ProductTooLarge(length));
                }

                for value in values {
                    encoder.encode(value.as_ref())?;
                }
            },
            EncodableValueContent::List { items } => {
                encoder.write_list_from_iterator(
                    items
                        .iter()
                        .map(|x| x.as_ref())
                )?;
            },
            EncodableValueContent::Map { entries } => {
                encoder.write_map_from_iterator(
                    entries
                        .iter()
                        .map(|(k, v)| (k.as_ref(), v.as_ref())
                    )
                )?;
            },
            EncodableValueContent::Sum { discriminator, value } => {
                todo!()
            },
        }
        Ok(())
    }
}

pub enum EncodableValueContent<E: Encoder> {
    RawBytes {
        bytes: Vec<u8>
    },
    Product {
        values: Vec<Box<dyn Encode<E>>>
    },
    List {
        items: Vec<Box<dyn Encode<E>>>
    },
    Map {
        entries: Vec<(Box<dyn Encode<E>>, Box<dyn Encode<E>>)>
    },
    Sum {
        discriminator: EncodableValueDiscriminator<E>,
        value: Box<dyn Encode<E>>,
    },
}

pub enum EncodableValueDiscriminator<E: Encoder> {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Any(Box<dyn Encode<E>>),
}
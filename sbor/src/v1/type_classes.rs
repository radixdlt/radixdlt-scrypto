#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TypeEncodingClass {
    RawBytes(LengthType),
    ProductType(ProductTypeLength),
    SumType(SumTypeDiscriminator),
    List(LengthType),
    Map(LengthType),
}

impl TypeEncodingClass {
    // TODO separate out RAW_BYTE / RAW_BYTES_2 / RAW_BYTES_4 / RAW_BYTES_8
    pub const RAW_BYTES_U8_LENGTH: u8 = 0x00;
    pub const RAW_BYTES_U16_LENGTH: u8 = 0x01;
    pub const RAW_BYTES_U32_LENGTH: u8 = 0x02;
    pub const RAW_BYTES_U64_LENGTH: u8 = 0x03;

    // TODO separate out PRODUCT_TYPE_ZERO / PRODUCT_LEN_1 / PRODUCT_LEN_2 / PRODUCT_LEN_3 / .. / PRODUCT_LEN_8
    pub const PRODUCT_TYPE_U8_LENGTH: u8 = 0x10;
    pub const PRODUCT_TYPE_U16_LENGTH: u8 = 0x11;

    pub const SUM_TYPE_U8_DISCRIMINATOR: u8 = 0x20;
    pub const SUM_TYPE_U16_DISCRIMINATOR: u8 = 0x21;
    pub const SUM_TYPE_U32_DISCRIMINATOR: u8 = 0x22;
    pub const SUM_TYPE_U64_DISCRIMINATOR: u8 = 0x23;
    pub const SUM_TYPE_ANY_DISCRIMINATOR: u8 = 0x24;

    pub const LIST_U8_LENGTH: u8 = 0x30;
    pub const LIST_U16_LENGTH: u8 = 0x31;
    pub const LIST_U32_LENGTH: u8 = 0x32;
    pub const LIST_U64_LENGTH: u8 = 0x33;

    pub const MAP_U8_LENGTH: u8 = 0x40;
    pub const MAP_U16_LENGTH: u8 = 0x41;
    pub const MAP_U32_LENGTH: u8 = 0x42;
    pub const MAP_U64_LENGTH: u8 = 0x43;

    pub const fn encoding_id(&self) -> u8 {
        match self {
            TypeEncodingClass::RawBytes(len) => len.raw_bytes_encoding_id(),
            TypeEncodingClass::ProductType(type_length) => type_length.encoding_id(),
            TypeEncodingClass::SumType(discriminator) => discriminator.encoding_id(),
            TypeEncodingClass::List(len) => len.list_encoding_id(),
            TypeEncodingClass::Map(len) => len.map_encoding_id(),
        }
    }

    pub fn try_from_byte(byte: u8) -> Option<TypeEncodingClass> {
        Some(match byte {
            Self::RAW_BYTES_U8_LENGTH => TypeEncodingClass::RawBytes(LengthType::U8),
            Self::RAW_BYTES_U16_LENGTH => TypeEncodingClass::RawBytes(LengthType::U16),
            Self::RAW_BYTES_U32_LENGTH => TypeEncodingClass::RawBytes(LengthType::U32),
            Self::RAW_BYTES_U64_LENGTH => TypeEncodingClass::RawBytes(LengthType::U64),
            Self::PRODUCT_TYPE_U8_LENGTH => TypeEncodingClass::ProductType(ProductTypeLength::U8),
            Self::PRODUCT_TYPE_U16_LENGTH => TypeEncodingClass::ProductType(ProductTypeLength::U16),
            Self::SUM_TYPE_U8_DISCRIMINATOR => TypeEncodingClass::SumType(SumTypeDiscriminator::U8),
            Self::SUM_TYPE_U16_DISCRIMINATOR => TypeEncodingClass::SumType(SumTypeDiscriminator::U16),
            Self::SUM_TYPE_U32_DISCRIMINATOR => TypeEncodingClass::SumType(SumTypeDiscriminator::U32),
            Self::SUM_TYPE_U64_DISCRIMINATOR => TypeEncodingClass::SumType(SumTypeDiscriminator::U64),
            Self::SUM_TYPE_ANY_DISCRIMINATOR => TypeEncodingClass::SumType(SumTypeDiscriminator::Any),
            Self::LIST_U8_LENGTH => TypeEncodingClass::List(LengthType::U8),
            Self::LIST_U16_LENGTH => TypeEncodingClass::List(LengthType::U16),
            Self::LIST_U32_LENGTH => TypeEncodingClass::List(LengthType::U32),
            Self::LIST_U64_LENGTH => TypeEncodingClass::List(LengthType::U64),
            Self::MAP_U8_LENGTH => TypeEncodingClass::Map(LengthType::U8),
            Self::MAP_U16_LENGTH => TypeEncodingClass::Map(LengthType::U16),
            Self::MAP_U32_LENGTH => TypeEncodingClass::Map(LengthType::U32),
            Self::MAP_U64_LENGTH => TypeEncodingClass::Map(LengthType::U64),
            _ => return None,
        })
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SumTypeDiscriminator {
    U8,
    U16,
    U32,
    U64,
    Any
}

impl SumTypeDiscriminator {
    pub const fn encoding_id(&self) -> u8 {
        match self {
            SumTypeDiscriminator::U8 => TypeEncodingClass::SUM_TYPE_U8_DISCRIMINATOR,
            SumTypeDiscriminator::U16 => TypeEncodingClass::SUM_TYPE_U16_DISCRIMINATOR,
            SumTypeDiscriminator::U32 => TypeEncodingClass::SUM_TYPE_U32_DISCRIMINATOR,
            SumTypeDiscriminator::U64 => TypeEncodingClass::SUM_TYPE_U64_DISCRIMINATOR,
            SumTypeDiscriminator::Any => TypeEncodingClass::SUM_TYPE_ANY_DISCRIMINATOR,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProductTypeLength {
    U8,
    U16,
}

impl ProductTypeLength {
    pub const fn encoding_id(&self) -> u8 {
        match self {
            ProductTypeLength::U8 => TypeEncodingClass::PRODUCT_TYPE_U8_LENGTH,
            ProductTypeLength::U16 => TypeEncodingClass::PRODUCT_TYPE_U16_LENGTH,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LengthType {
    U8,
    U16,
    U32,
    U64,
}

impl LengthType {
    #[inline]
    pub const fn raw_bytes_encoding_id(&self) -> u8 {
        match self {
            LengthType::U8 => TypeEncodingClass::RAW_BYTES_U8_LENGTH,
            LengthType::U16 => TypeEncodingClass::RAW_BYTES_U16_LENGTH,
            LengthType::U32 => TypeEncodingClass::RAW_BYTES_U32_LENGTH,
            LengthType::U64 => TypeEncodingClass::RAW_BYTES_U64_LENGTH,
        }
    }

    #[inline]
    pub const fn list_encoding_id(&self) -> u8 {
        match self {
            LengthType::U8 => TypeEncodingClass::LIST_U8_LENGTH,
            LengthType::U16 => TypeEncodingClass::LIST_U16_LENGTH,
            LengthType::U32 => TypeEncodingClass::LIST_U32_LENGTH,
            LengthType::U64 => TypeEncodingClass::LIST_U64_LENGTH,
        }
    }

    #[inline]
    pub const fn map_encoding_id(&self) -> u8 {
        match self {
            LengthType::U8 => TypeEncodingClass::MAP_U8_LENGTH,
            LengthType::U16 => TypeEncodingClass::MAP_U16_LENGTH,
            LengthType::U32 => TypeEncodingClass::MAP_U32_LENGTH,
            LengthType::U64 => TypeEncodingClass::MAP_U64_LENGTH,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SizedLength {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl SizedLength {
    #[inline]
    pub fn from_size(length: usize) -> SizedLength {
        // There may be a faster way of implementing this by avoiding branches...
        // But I hope LLVM would take care of this
        if length <= u8::MAX as usize {
            SizedLength::U8(length as u8)
        } else if length <= u16::MAX as usize {
            SizedLength::U16(length as u16)
        } else if length <= u32::MAX as usize {
            SizedLength::U32(length as u32)
        } else if length <= u64::MAX as usize {
            SizedLength::U64(length as u64)
        } else {
            panic!("usize larger than 8 bytes not supported")
        }
    }

    #[inline]
    pub const fn length_type(&self) -> LengthType {
        match self {
            SizedLength::U8(_) => LengthType::U8,
            SizedLength::U16(_) => LengthType::U16,
            SizedLength::U32(_) => LengthType::U32,
            SizedLength::U64(_) => LengthType::U64,
        }
    }

    #[inline]
    pub const fn raw_bytes_encoding_id(&self) -> u8 {
        self.length_type().raw_bytes_encoding_id()
    }

    #[inline]
    pub const fn list_encoding_id(&self) -> u8 {
        self.length_type().list_encoding_id()
    }

    #[inline]
    pub const fn map_encoding_id(&self) -> u8 {
        self.length_type().map_encoding_id()
    }
}
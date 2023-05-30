use crate::internal_prelude::*;
use sbor::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueType {
    Blob,
    Attachment,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrepareError {
    DecodeError(DecodeError),
    EncodeError(EncodeError),
    TooManyValues {
        value_type: ValueType,
        actual: usize,
        max: usize,
    },
    LengthOverflow,
    UnexpectedDiscriminator {
        expected: u8,
        actual: u8,
    },
    Other(String),
}

impl From<DecodeError> for PrepareError {
    fn from(value: DecodeError) -> Self {
        Self::DecodeError(value)
    }
}

impl From<EncodeError> for PrepareError {
    fn from(value: EncodeError) -> Self {
        Self::EncodeError(value)
    }
}

pub struct TransactionDecoder<'a>(ManifestDecoder<'a>);

impl<'a> TransactionDecoder<'a> {
    pub fn new(manifest_decoder: ManifestDecoder<'a>) -> Self {
        Self(manifest_decoder)
    }

    /// Should be called before any manual call to read_X_header
    pub fn track_stack_depth_increase(&mut self) -> Result<(), PrepareError> {
        Ok(self.0.track_stack_depth_increase()?)
    }

    pub fn read_struct_header(&mut self, length: usize) -> Result<(), PrepareError> {
        self.0.read_and_check_value_kind(ValueKind::Tuple)?;
        self.0.read_and_check_size(length)?;
        Ok(())
    }

    pub fn read_enum_header(&mut self) -> Result<(u8, usize), PrepareError> {
        self.0.read_and_check_value_kind(ValueKind::Enum)?;
        let discriminator = self.0.read_discriminator()?;
        let length = self.0.read_size()?;
        Ok((discriminator, length))
    }

    pub fn read_expected_enum_variant_header(
        &mut self,
        expected_discriminator: u8,
        length: usize,
    ) -> Result<(), PrepareError> {
        self.0.read_and_check_value_kind(ValueKind::Enum)?;
        let discriminator = self.0.read_discriminator()?;
        if discriminator != expected_discriminator {
            return Err(PrepareError::UnexpectedDiscriminator {
                expected: expected_discriminator,
                actual: discriminator,
            });
        }
        self.0.read_and_check_size(length)?;
        Ok(())
    }

    pub fn read_array_header(
        &mut self,
        element_value_kind: ManifestValueKind,
    ) -> Result<usize, PrepareError> {
        self.0.read_and_check_value_kind(ValueKind::Array)?;
        self.0.read_and_check_value_kind(element_value_kind)?;
        Ok(self.0.read_size()?)
    }

    /// Should be called after reading all the children following a manual read_X_header call
    pub fn track_stack_depth_decrease(&mut self) -> Result<(), PrepareError> {
        Ok(self.0.track_stack_depth_decrease()?)
    }

    pub fn decode<T: ManifestDecode>(&mut self) -> Result<T, PrepareError> {
        Ok(self.0.decode()?)
    }

    pub fn decode_deeper_body_with_value_kind<T: ManifestDecode>(
        &mut self,
        value_kind: ManifestValueKind,
    ) -> Result<T, PrepareError> {
        Ok(self.0.decode_deeper_body_with_value_kind(value_kind)?)
    }

    pub fn get_offset(&self) -> usize {
        self.0.get_offset()
    }

    pub fn get_slice(&self, start_offset: usize, end_offset: usize) -> &[u8] {
        &self.0.get_input_slice()[start_offset..end_offset]
    }

    pub fn destructure(self) -> ManifestDecoder<'a> {
        self.0
    }
}

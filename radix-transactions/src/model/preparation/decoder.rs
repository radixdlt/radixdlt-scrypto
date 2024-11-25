use crate::internal_prelude::*;
use sbor::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueType {
    Blob,
    Subintent,
    ChildSubintentSpecifier,
    SubintentSignatureBatches,
    // Too many signatures is captured at validation time
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrepareError {
    TransactionTypeNotSupported,
    TransactionTooLarge,
    DecodeError(DecodeError),
    EncodeError(EncodeError),
    TooManyValues {
        value_type: ValueType,
        actual: usize,
        max: usize,
    },
    LengthOverflow,
    UnexpectedTransactionDiscriminator {
        actual: Option<u8>,
    },
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

pub type PreparationSettings = PreparationSettingsV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub struct PreparationSettingsV1 {
    pub v2_transactions_permitted: bool,
    pub max_user_payload_length: usize,
    pub max_ledger_payload_length: usize,
    pub max_child_subintents_per_intent: usize,
    pub max_subintents_per_transaction: usize,
    pub max_blobs: usize,
}

static LATEST_PREPARATION_SETTINGS: PreparationSettings = PreparationSettings::latest();

impl PreparationSettings {
    pub const fn latest() -> Self {
        Self::cuttlefish()
    }

    pub const fn babylon() -> Self {
        let max_user_payload_length = 1 * 1024 * 1024;
        Self {
            v2_transactions_permitted: false,
            max_user_payload_length,
            max_ledger_payload_length: max_user_payload_length + 10,
            max_child_subintents_per_intent: 0,
            max_subintents_per_transaction: 0,
            max_blobs: 64,
        }
    }

    pub const fn cuttlefish() -> Self {
        Self {
            v2_transactions_permitted: true,
            max_child_subintents_per_intent: 32,
            max_subintents_per_transaction: 32,
            ..Self::babylon()
        }
    }

    pub fn latest_ref() -> &'static Self {
        &LATEST_PREPARATION_SETTINGS
    }

    fn check_len(
        &self,
        kind: TransactionPayloadKind,
        payload_len: usize,
    ) -> Result<(), PrepareError> {
        match kind {
            TransactionPayloadKind::CompleteUserTransaction => {
                if payload_len > self.max_user_payload_length {
                    return Err(PrepareError::TransactionTooLarge);
                }
            }
            TransactionPayloadKind::LedgerTransaction => {
                if payload_len > self.max_ledger_payload_length {
                    return Err(PrepareError::TransactionTooLarge);
                }
            }
            TransactionPayloadKind::Other => {
                // No explicit payload length checks
            }
        }
        Ok(())
    }
}

pub struct TransactionDecoder<'a> {
    decoder: ManifestDecoder<'a>,
    settings: &'a PreparationSettings,
}

impl<'a> TransactionDecoder<'a> {
    pub fn new_transaction(
        payload: &'a [u8],
        kind: TransactionPayloadKind,
        settings: &'a PreparationSettings,
    ) -> Result<Self, PrepareError> {
        settings.check_len(kind, payload.len())?;
        let mut decoder = ManifestDecoder::new(&payload, MANIFEST_SBOR_V1_MAX_DEPTH);
        decoder.read_and_check_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
        Ok(Self { decoder, settings })
    }

    pub fn new_partial(
        payload: &'a [u8],
        settings: &'a PreparationSettings,
    ) -> Result<Self, PrepareError> {
        let mut decoder = ManifestDecoder::new(&payload, MANIFEST_SBOR_V1_MAX_DEPTH);
        decoder.read_and_check_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
        Ok(Self { decoder, settings })
    }

    pub fn settings(&self) -> &PreparationSettings {
        &self.settings
    }

    /// Should be called before any manual call to read_X_header
    pub fn track_stack_depth_increase(&mut self) -> Result<(), PrepareError> {
        Ok(self.decoder.track_stack_depth_increase()?)
    }

    pub fn read_header(
        &mut self,
        header: ExpectedTupleHeader,
        expected_length: usize,
    ) -> Result<(), PrepareError> {
        match header {
            ExpectedTupleHeader::EnumNoValueKind { discriminator } => {
                self.decoder.read_expected_discriminator(discriminator)?;
            }
            ExpectedTupleHeader::EnumWithValueKind { discriminator } => {
                self.read_and_check_value_kind(ValueKind::Enum)?;
                self.decoder.read_expected_discriminator(discriminator)?;
            }
            ExpectedTupleHeader::TupleWithValueKind => {
                self.read_and_check_value_kind(ValueKind::Tuple)?;
            }
            ExpectedTupleHeader::TupleNoValueKind => {}
        }
        self.decoder.read_and_check_size(expected_length)?;
        Ok(())
    }

    pub fn read_enum_header(&mut self) -> Result<(u8, usize), PrepareError> {
        self.read_and_check_value_kind(ValueKind::Enum)?;
        let discriminator = self.decoder.read_discriminator()?;
        let length = self.decoder.read_size()?;
        Ok((discriminator, length))
    }

    pub fn read_array_header(
        &mut self,
        element_value_kind: ManifestValueKind,
    ) -> Result<usize, PrepareError> {
        self.read_and_check_value_kind(ValueKind::Array)?;
        self.read_array_header_without_value_kind(element_value_kind)
    }

    pub fn read_array_header_without_value_kind(
        &mut self,
        element_value_kind: ManifestValueKind,
    ) -> Result<usize, PrepareError> {
        self.read_and_check_value_kind(element_value_kind)?;
        Ok(self.decoder.read_size()?)
    }

    pub fn read_and_check_value_kind(
        &mut self,
        value_kind: ManifestValueKind,
    ) -> Result<(), PrepareError> {
        self.decoder.read_and_check_value_kind(value_kind)?;
        Ok(())
    }

    /// Should be called after reading all the children following a manual read_X_header call
    pub fn track_stack_depth_decrease(&mut self) -> Result<(), PrepareError> {
        Ok(self.decoder.track_stack_depth_decrease()?)
    }

    pub fn decode<T: ManifestDecode>(&mut self) -> Result<T, PrepareError> {
        Ok(self.decoder.decode()?)
    }

    pub fn decode_deeper_body_with_value_kind<T: ManifestDecode>(
        &mut self,
        value_kind: ManifestValueKind,
    ) -> Result<T, PrepareError> {
        Ok(self
            .decoder
            .decode_deeper_body_with_value_kind(value_kind)?)
    }

    pub fn get_offset(&self) -> usize {
        self.decoder.get_offset()
    }

    pub fn get_slice_with_valid_bounds(&self, start_offset: usize, end_offset: usize) -> &[u8] {
        &self.decoder.get_input_slice()[start_offset..end_offset]
    }

    pub fn get_input_slice(&self) -> &[u8] {
        &self.decoder.get_input_slice()
    }

    pub fn check_complete(self) -> Result<(), PrepareError> {
        self.decoder.check_end()?;
        Ok(())
    }
}

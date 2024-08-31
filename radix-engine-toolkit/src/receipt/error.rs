use radix_common::prelude::*;

#[derive(Clone, Debug)]
pub enum ToolkitReceiptError {
    InvalidNodeId,
    InvalidGlobalAddress,
    InvalidResourceAddress,
    InvalidNonFungibleGlobalId,
    ReceiptLacksExecutionTrace,
    AddressBech32mEncodeError(AddressBech32EncodeError),
    AddressBech32mDecodeError(AddressBech32DecodeError),
}

impl From<AddressBech32EncodeError> for ToolkitReceiptError {
    fn from(value: AddressBech32EncodeError) -> Self {
        Self::AddressBech32mEncodeError(value)
    }
}

impl From<AddressBech32DecodeError> for ToolkitReceiptError {
    fn from(value: AddressBech32DecodeError) -> Self {
        Self::AddressBech32mDecodeError(value)
    }
}

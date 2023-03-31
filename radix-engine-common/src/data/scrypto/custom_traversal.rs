use sbor::decoder::*;
use sbor::traversal::*;
use sbor::value_kind::*;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScryptoCustomTerminalValueRef(pub ScryptoCustomValue);

impl CustomTerminalValueRef for ScryptoCustomTerminalValueRef {
    type CustomValueKind = ScryptoCustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind {
        self.0.get_custom_value_kind()
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTraversal {}

impl CustomTraversal for ScryptoCustomTraversal {
    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTerminalValueRef<'de> = ScryptoCustomTerminalValueRef;

    fn decode_custom_value_body<'de, R>(
        custom_value_kind: Self::CustomValueKind,
        reader: &mut R,
    ) -> Result<Self::CustomTerminalValueRef<'de>, DecodeError>
    where
        R: PayloadTraverser<'de, Self::CustomValueKind>,
    {
        // TODO: copy-free decoding for better performance
        ScryptoCustomValue::decode_body_with_value_kind(
            reader,
            ValueKind::Custom(custom_value_kind),
        )
        .map(|v| ScryptoCustomTerminalValueRef(v))
    }
}

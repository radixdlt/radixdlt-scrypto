use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestCustomTerminalValueRef(pub ManifestCustomValue);

impl CustomTerminalValueRef for ManifestCustomTerminalValueRef {
    type CustomValueKind = ManifestCustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind {
        self.0.get_custom_value_kind()
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ManifestCustomTraversal {}

impl CustomTraversal for ManifestCustomTraversal {
    type CustomValueKind = ManifestCustomValueKind;
    type CustomTerminalValueRef<'de> = ManifestCustomTerminalValueRef;

    fn read_custom_value_body<'de, R>(
        custom_value_kind: Self::CustomValueKind,
        reader: &mut R,
    ) -> Result<Self::CustomTerminalValueRef<'de>, DecodeError>
    where
        R: BorrowingDecoder<'de, Self::CustomValueKind>,
    {
        // TODO: copy-free decoding for better performance
        ManifestCustomValue::decode_body_with_value_kind(
            reader,
            ValueKind::Custom(custom_value_kind),
        )
        .map(|v| ManifestCustomTerminalValueRef(v))
    }
}

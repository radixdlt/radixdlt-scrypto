use sbor::decoder::*;
use sbor::traversal::*;
use sbor::value_kind::*;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScryptoCustomTerminalValueRef(ScryptoCustomValue);

impl CustomTerminalValueRef for ScryptoCustomTerminalValueRef {
    type CustomValueKind = ScryptoCustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind {
        self.0.get_custom_value_kind()
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTerminalValueBatchRef {}

impl CustomTerminalValueBatchRef for ScryptoCustomTerminalValueBatchRef {
    type CustomValueKind = ScryptoCustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind {
        unreachable!("ScryptoCustomTerminalValueBatchRef can't exist")
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomContainerHeader {}

impl CustomContainerHeader for ScryptoCustomContainerHeader {
    type CustomValueKind = ScryptoCustomValueKind;

    fn get_child_count(&self) -> u32 {
        unreachable!("ScryptoCustomContainerHeader can't exist")
    }

    fn get_implicit_child_value_kind(
        &self,
        _: u32,
    ) -> (ParentRelationship, Option<ValueKind<Self::CustomValueKind>>) {
        unreachable!("ScryptoCustomContainerHeader can't exist")
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTraversal {}

impl CustomTraversal for ScryptoCustomTraversal {
    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTerminalValueRef<'de> = ScryptoCustomTerminalValueRef;
    type CustomTerminalValueBatchRef<'de> = ScryptoCustomTerminalValueBatchRef;
    type CustomContainerHeader = ScryptoCustomContainerHeader;
    type CustomValueTraverser = ScryptoCustomTraverser;

    fn new_value_traversal(
        custom_value_kind: Self::CustomValueKind,
        parent_relationship: ParentRelationship,
        start_offset: usize,
        _: u8,
    ) -> Self::CustomValueTraverser {
        ScryptoCustomTraverser {
            custom_value_kind,
            parent_relationship,
            start_offset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScryptoCustomTraverser {
    custom_value_kind: ScryptoCustomValueKind,
    parent_relationship: ParentRelationship,
    start_offset: usize,
}

impl CustomValueTraverser for ScryptoCustomTraverser {
    type CustomTraversal = ScryptoCustomTraversal;

    fn next_event<
        't,
        'de,
        R: PayloadTraverser<'de, <Self::CustomTraversal as CustomTraversal>::CustomValueKind>,
    >(
        &mut self,
        container_stack: &'t mut Vec<ContainerChild<Self::CustomTraversal>>,
        decoder: &mut R,
    ) -> LocatedTraversalEvent<'t, 'de, Self::CustomTraversal> {
        let result = ScryptoCustomValue::decode_body_with_value_kind(
            decoder,
            ValueKind::Custom(self.custom_value_kind),
        );
        let location = Location {
            start_offset: self.start_offset,
            end_offset: decoder.get_offset(),
            parent_relationship: self.parent_relationship,
            ancestor_path: container_stack,
        };
        let event = match result {
            Ok(custom_value) => TraversalEvent::TerminalValue(TerminalValueRef::Custom(
                ScryptoCustomTerminalValueRef(custom_value),
            )),
            Err(decode_error) => TraversalEvent::DecodeError(decode_error),
        };
        LocatedTraversalEvent { location, event }
    }
}

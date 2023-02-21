use super::*;
use crate::*;

use super::CustomTraversal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalEvent<'de, C: CustomTraversal> {
    ContainerStart(LocatedDecoding<ContainerHeader<C::CustomContainerHeader>>),
    ContainerEnd(LocatedDecoding<ContainerHeader<C::CustomContainerHeader>>),
    TerminalValue(LocatedDecoding<TerminalValueRef<'de, C::CustomTerminalValueRef>>),
    TerminalValueBatch(LocatedDecoding<TerminalValueBatchRef<'de, C::CustomTerminalValueBatchRef>>),
    PayloadEnd(Location),
    DecodeError(LocatedError<DecodeError>),
}

impl<'de, C: CustomTraversal> TraversalEvent<'de, C> {
    pub fn get_next_sbor_depth(&self) -> u8 {
        match self {
            TraversalEvent::ContainerStart(le) => le.location.sbor_depth + 1,
            TraversalEvent::ContainerEnd(le) => le.location.sbor_depth,
            TraversalEvent::TerminalValue(le) => le.location.sbor_depth,
            TraversalEvent::TerminalValueBatch(le) => le.location.sbor_depth,
            TraversalEvent::PayloadEnd(location) => location.sbor_depth,
            TraversalEvent::DecodeError(le) => le.location.sbor_depth,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            TraversalEvent::DecodeError(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocatedError<E> {
    pub error: E,
    pub location: Location,
}

/// A wrapper for traversal event bodies, given the context inside the payload.
/// The `start_offset` and `end_offset` have meanings in the context of the event.
/// * For ContainerValueStart, they're the start/end of the header
/// * For ContainerValueEnd, they're the start/end of the whole value (including the header)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocatedDecoding<T> {
    pub inner: T,
    pub parent_relationship: ParentRelationship,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub start_offset: usize,
    pub end_offset: usize,
    pub sbor_depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerHeader<H: CustomContainerHeader> {
    Tuple(usize),
    EnumVariant(u8, usize),
    Array(ValueKind<H::CustomValueKind>, usize),
    Map(
        ValueKind<H::CustomValueKind>,
        ValueKind<H::CustomValueKind>,
        usize,
    ),
    Custom(H),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentRelationship {
    Root,
    Element { index: usize },
    ElementBatch { from_index: usize, to_index: usize },
    MapKey { index: usize },
    MapValue { index: usize },
}

impl<H: CustomContainerHeader> ContainerHeader<H> {
    pub fn get_child_count(&self) -> usize {
        match self {
            ContainerHeader::Tuple(size) => *size,
            ContainerHeader::EnumVariant(_, size) => *size,
            ContainerHeader::Array(_, size) => *size,
            ContainerHeader::Map(_, _, size) => *size * 2,
            ContainerHeader::Custom(custom_header) => custom_header.get_child_count(),
        }
    }

    pub fn get_implicit_child_value_kind(
        &self,
        index: usize,
    ) -> (ParentRelationship, Option<ValueKind<H::CustomValueKind>>) {
        match self {
            ContainerHeader::Tuple(_) => (ParentRelationship::Element { index }, None),
            ContainerHeader::EnumVariant(_, _) => (ParentRelationship::Element { index }, None),
            ContainerHeader::Array(element_value_kind, _) => (
                ParentRelationship::Element { index },
                Some(*element_value_kind),
            ),
            ContainerHeader::Map(key, value, _) => {
                if index % 2 == 0 {
                    (ParentRelationship::MapKey { index: index / 2 }, Some(*key))
                } else {
                    (
                        ParentRelationship::MapValue { index: index / 2 },
                        Some(*value),
                    )
                }
            }
            ContainerHeader::Custom(custom_header) => {
                custom_header.get_implicit_child_value_kind(index)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalValueRef<'a, V> {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    String(&'a str),
    Custom(V),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalValueBatchRef<'a, B> {
    U8(&'a [u8]),
    Custom(B),
}

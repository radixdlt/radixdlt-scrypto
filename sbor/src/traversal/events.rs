use super::*;
use crate::*;

use super::CustomTraversal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalEvent<'t, 'de, C: CustomTraversal> {
    PayloadPrefix(Location),
    ContainerStart(LocatedDecoding<'t, ContainerHeader<C>, C>),
    ContainerEnd(LocatedDecoding<'t, ContainerHeader<C>, C>),
    TerminalValue(LocatedDecoding<'t, TerminalValueRef<'de, C>, C>),
    TerminalValueBatch(LocatedDecoding<'t, TerminalValueBatchRef<'de, C>, C>),
    End(Location),
    DecodeError(LocatedError<DecodeError>),
}

impl<'t, 'de, C: CustomTraversal> TraversalEvent<'t, 'de, C> {
    pub fn get_next_sbor_depth(&self) -> u8 {
        match self {
            TraversalEvent::PayloadPrefix(location) => location.sbor_depth + 1,
            TraversalEvent::ContainerStart(le) => le.location.sbor_depth + 1,
            TraversalEvent::ContainerEnd(le) => le.location.sbor_depth,
            TraversalEvent::TerminalValue(le) => le.location.sbor_depth,
            TraversalEvent::TerminalValueBatch(le) => le.location.sbor_depth,
            TraversalEvent::End(location) => location.sbor_depth,
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
///
/// The `start_offset` and `end_offset` in `Location` have different meanings in the context of the event.
/// * For ContainerValueStart, they're the start/end of the header
/// * For ContainerValueEnd, they're the start/end of the whole value (including the header)
///
/// The `resultant_path` captures the path up to this point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocatedDecoding<'t, T, C: CustomTraversal> {
    pub inner: T,
    pub parent_relationship: ParentRelationship,
    pub location: Location,
    pub resultant_path: &'t [ContainerChild<C>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub start_offset: usize,
    pub end_offset: usize,
    pub sbor_depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerHeader<C: CustomTraversal> {
    Tuple(TupleHeader),
    EnumVariant(EnumVariantHeader),
    Array(ArrayHeader<C::CustomValueKind>),
    Map(MapHeader<C::CustomValueKind>),
    Custom(C::CustomContainerHeader),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TupleHeader {
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumVariantHeader {
    pub variant: u8,
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayHeader<X: CustomValueKind> {
    pub element_value_kind: ValueKind<X>,
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHeader<X: CustomValueKind> {
    pub key_value_kind: ValueKind<X>,
    pub value_value_kind: ValueKind<X>,
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentRelationship {
    Root,
    Element { index: u32 },
    ArrayElementBatch { from_index: u32, to_index: u32 },
    MapKey { index: u32 },
    MapValue { index: u32 },
}

impl<C: CustomTraversal> ContainerHeader<C> {
    pub fn get_child_count(&self) -> u32 {
        match self {
            ContainerHeader::Tuple(TupleHeader { length }) => *length,
            ContainerHeader::EnumVariant(EnumVariantHeader { length, .. }) => *length,
            ContainerHeader::Array(ArrayHeader { length, .. }) => *length,
            ContainerHeader::Map(MapHeader { length, .. }) => *length * 2,
            ContainerHeader::Custom(custom_header) => custom_header.get_child_count(),
        }
    }

    pub fn get_implicit_child_value_kind(
        &self,
        index: u32,
    ) -> (ParentRelationship, Option<ValueKind<C::CustomValueKind>>) {
        match self {
            ContainerHeader::Tuple(_) => (ParentRelationship::Element { index }, None),
            ContainerHeader::EnumVariant(_) => (ParentRelationship::Element { index }, None),
            ContainerHeader::Array(ArrayHeader {
                element_value_kind, ..
            }) => (
                ParentRelationship::Element { index },
                Some(*element_value_kind),
            ),
            ContainerHeader::Map(MapHeader {
                key_value_kind,
                value_value_kind,
                ..
            }) => {
                if index % 2 == 0 {
                    (
                        ParentRelationship::MapKey { index: index / 2 },
                        Some(*key_value_kind),
                    )
                } else {
                    (
                        ParentRelationship::MapValue { index: index / 2 },
                        Some(*value_value_kind),
                    )
                }
            }
            ContainerHeader::Custom(custom_header) => {
                custom_header.get_implicit_child_value_kind(index)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalValueRef<'de, T: CustomTraversal> {
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
    String(&'de str),
    Custom(T::CustomTerminalValueRef<'de>),
}

impl<'de, T: CustomTraversal> TerminalValueRef<'de, T> {
    pub fn value_kind(&self) -> ValueKind<T::CustomValueKind> {
        match self {
            TerminalValueRef::Bool(_) => ValueKind::Bool,
            TerminalValueRef::I8(_) => ValueKind::I8,
            TerminalValueRef::I16(_) => ValueKind::I16,
            TerminalValueRef::I32(_) => ValueKind::I32,
            TerminalValueRef::I64(_) => ValueKind::I64,
            TerminalValueRef::I128(_) => ValueKind::I128,
            TerminalValueRef::U8(_) => ValueKind::U8,
            TerminalValueRef::U16(_) => ValueKind::U16,
            TerminalValueRef::U32(_) => ValueKind::U32,
            TerminalValueRef::U64(_) => ValueKind::U64,
            TerminalValueRef::U128(_) => ValueKind::U128,
            TerminalValueRef::String(_) => ValueKind::String,
            TerminalValueRef::Custom(c) => ValueKind::Custom(c.custom_value_kind()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalValueBatchRef<'de, T: CustomTraversal> {
    U8(&'de [u8]),
    Custom(T::CustomTerminalValueBatchRef<'de>),
}

impl<'de, T: CustomTraversal> TerminalValueBatchRef<'de, T> {
    pub fn value_kind(&self) -> ValueKind<T::CustomValueKind> {
        match self {
            TerminalValueBatchRef::U8(_) => ValueKind::U8,
            TerminalValueBatchRef::Custom(c) => ValueKind::Custom(c.custom_value_kind()),
        }
    }
}

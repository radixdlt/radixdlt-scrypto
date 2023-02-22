use super::*;
use crate::*;

use super::CustomTraversal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalEvent<'de, C: CustomTraversal> {
    PayloadPrefix(Location),
    ContainerStart(LocatedDecoding<ContainerHeader<C::CustomContainerHeader>>),
    ContainerEnd(LocatedDecoding<ContainerHeader<C::CustomContainerHeader>>),
    TerminalValue(LocatedDecoding<TerminalValueRef<'de, C::CustomTerminalValueRef<'de>>>),
    TerminalValueBatch(
        LocatedDecoding<TerminalValueBatchRef<'de, C::CustomTerminalValueBatchRef<'de>>>,
    ),
    End(Location),
    DecodeError(LocatedError<DecodeError>),
}

impl<'de, C: CustomTraversal> TraversalEvent<'de, C> {
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
    Tuple(TupleHeader),
    EnumVariant(EnumVariantHeader),
    Array(ArrayHeader<H::CustomValueKind>),
    Map(MapHeader<H::CustomValueKind>),
    Custom(H),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TupleHeader {
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumVariantHeader {
    pub variant: u8,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayHeader<X: CustomValueKind> {
    pub element_value_kind: ValueKind<X>,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHeader<X: CustomValueKind> {
    pub key_value_kind: ValueKind<X>,
    pub value_value_kind: ValueKind<X>,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentRelationship {
    Root,
    Element { index: usize },
    ArrayElementBatch { from_index: usize, to_index: usize },
    MapKey { index: usize },
    MapValue { index: usize },
}

impl<H: CustomContainerHeader> ContainerHeader<H> {
    pub fn get_child_count(&self) -> usize {
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
        index: usize,
    ) -> (ParentRelationship, Option<ValueKind<H::CustomValueKind>>) {
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
pub enum TerminalValueRef<'a, V: CustomTerminalValueRef> {
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

impl<'a, V: CustomTerminalValueRef> TerminalValueRef<'a, V> {
    pub fn value_kind(&self) -> ValueKind<V::CustomValueKind> {
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
pub enum TerminalValueBatchRef<'a, B> {
    U8(&'a [u8]),
    Custom(B),
}

impl<'a, B: CustomTerminalValueBatchRef> TerminalValueBatchRef<'a, B> {
    pub fn value_kind(&self) -> ValueKind<B::CustomValueKind> {
        match self {
            TerminalValueBatchRef::U8(_) => ValueKind::U8,
            TerminalValueBatchRef::Custom(c) => ValueKind::Custom(c.custom_value_kind()),
        }
    }
}

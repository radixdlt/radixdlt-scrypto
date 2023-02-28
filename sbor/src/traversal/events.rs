use super::*;
use crate::*;

use super::CustomTraversal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedTraversalEvent<'t, 'de, C: CustomTraversal> {
    pub location: Location<'t, C>,
    pub event: TraversalEvent<'de, C>,
}

impl<'t, 'de, C: CustomTraversal> LocatedTraversalEvent<'t, 'de, C> {
    pub fn get_next_sbor_depth(&self) -> usize {
        match self.event {
            TraversalEvent::PayloadPrefix | TraversalEvent::End => 0,
            TraversalEvent::ContainerStart(_) => self.location.get_sbor_depth() + 1,
            _ => self.location.get_sbor_depth(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalEvent<'de, C: CustomTraversal> {
    PayloadPrefix,
    ContainerStart(ContainerHeader<C>),
    ContainerEnd(ContainerHeader<C>),
    TerminalValue(TerminalValueRef<'de, C>),
    TerminalValueBatch(TerminalValueBatchRef<'de, C>),
    End,
    DecodeError(DecodeError),
}

impl<'de, C: CustomTraversal> TraversalEvent<'de, C> {
    pub fn is_error(&self) -> bool {
        match self {
            TraversalEvent::DecodeError(_) => true,
            _ => false,
        }
    }
}

/// The Location of the encoding - capturing both the byte offset in the payload, and also
/// the container-path-based location in the SBOR value model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location<'t, C: CustomTraversal> {
    /// An offset in the payload, where this `Location` starts.
    /// The meaning of this offset depends on the context of the event, eg:
    /// * For ContainerStart, this is the start of the value
    /// * For ContainerEnd, this is the start of the value
    /// * For DecodeError, this is the location where the error occurred
    pub start_offset: usize,
    /// An offset in the payload, where this `Location` ends (could be the same as start_offset).
    /// The meaning of this offset depends on the context of the event, eg:
    /// * For ContainerStart, this is the end of the header
    /// * For ContainerEnd, this is the end of the whole container value
    /// * For DecodeError, this is the location where the error occurred
    pub end_offset: usize,
    /// The relationship of the value currently under consideration with its container parent
    pub parent_relationship: ParentRelationship,
    /// The path of containers from the root to the current value.
    /// If the event is ContainerStart/End, this does not include the newly started/ended container.
    pub ancestor_path: &'t [ContainerChild<C>],
}

impl<'t, C: CustomTraversal> Location<'t, C> {
    /// The current SBOR depth
    pub fn get_sbor_depth(&self) -> usize {
        match self.parent_relationship {
            ParentRelationship::NotInValueModel => 0,
            _ => self.ancestor_path.len() + 1,
        }
    }
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
    NotInValueModel,
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

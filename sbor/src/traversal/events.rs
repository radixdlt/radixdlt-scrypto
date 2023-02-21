use crate::*;

use super::CustomTraversal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalEvent<'de, C: CustomTraversal> {
    ContainerStart(LocatedEvent<ContainerHeader<C::CustomContainerHeader>>),
    ContainerEnd(LocatedEvent<ContainerHeader<C::CustomContainerHeader>>),
    TerminalValue(LocatedEvent<TerminalValueRef<'de, C::CustomTerminalValueRef>>),
    TerminalValueBatch(LocatedEvent<TerminalValueBatchRef<'de, C::CustomTerminalValueBatchRef>>),
    DecodeError(LocatedEvent<DecodeError>),
}

impl<'de, C: CustomTraversal> TraversalEvent<'de, C> {
    pub fn get_next_sbor_depth(&self) -> u8 {
        match self {
            TraversalEvent::ContainerStart(le) => le.sbor_depth + 1,
            TraversalEvent::ContainerEnd(le) => le.sbor_depth,
            TraversalEvent::TerminalValue(le) => le.sbor_depth,
            TraversalEvent::TerminalValueBatch(le) => le.sbor_depth,
            TraversalEvent::DecodeError(le) => le.sbor_depth,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            TraversalEvent::DecodeError(_) => true,
            _ => false,
        }
    }
}

/// A wrapper for traversal event bodies, given the context inside the payload.
/// The `start_offset` and `end_offset` have meanings in the context of the event.
/// * For ContainerValueStart, they're the start/end of the header
/// * For ContainerValueEnd, they're the start/end of the whole value (including the header)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocatedEvent<T> {
    pub event: T,
    pub start_offset: usize,
    pub end_offset: usize,
    pub sbor_depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerHeader<H: CustomContainerHeader> {
    Root,
    Tuple(usize),
    EnumVariant(u8, usize),
    Array(ValueKind<H::CustomValueKind>, usize),
    Map(ValueKind<H::CustomValueKind>, ValueKind<H::CustomValueKind>, usize),
    Custom(H),
}

impl<H: CustomContainerHeader> ContainerHeader<H> {
    pub fn get_child_count(&self) -> usize {
        match self {
            ContainerHeader::Root => 1,
            ContainerHeader::Tuple(size) => *size,
            ContainerHeader::EnumVariant(_, size) => *size,
            ContainerHeader::Array(_, size) => *size,
            ContainerHeader::Map(_, _, size) => *size * 2,
            ContainerHeader::Custom(custom_header) => custom_header.get_child_count(), 
        }
    }

    pub fn get_implicit_child_value_kind(&self, index: usize) -> Option<ValueKind<H::CustomValueKind>> {
        match self {
            ContainerHeader::Root => None,
            ContainerHeader::Tuple(_) => None,
            ContainerHeader::EnumVariant(_, _) => None,
            ContainerHeader::Array(element_value_kind, _) => Some(*element_value_kind),
            ContainerHeader::Map(key, value, _) => {
                Some(if index % 2 == 0 { *key } else { *value })
            }
            ContainerHeader::Custom(custom_header) => custom_header.get_implicit_child_value_kind(index),
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

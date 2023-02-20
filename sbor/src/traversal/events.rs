use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalEvent<'a, X: CustomValueKind, C> {
    StartOwnerValue(VisitOwnerValueHeader<X>),
    EndOwnerValue(VisitFullOwnerValue<X>),
    VisitTerminalValue(VisitTerminalValue<'a>),
    VisitTerminalValueSlice(VisitTerminalValueSlice<'a>),
    DecodeError(DecodeErrorEvent),
    Custom(C),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeErrorEvent {
    pub error: DecodeError,
    pub stack_depth: u8,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisitOwnerValueHeader<X: CustomValueKind> {
    pub header: OwnerValueHeader<X>,
    pub stack_depth: u8,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisitFullOwnerValue<X: CustomValueKind> {
    pub header: OwnerValueHeader<X>,
    pub stack_depth: u8,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerValueHeader<X: CustomValueKind> {
    Root,
    Tuple(usize),
    EnumVariant(u8, usize),
    Array(ValueKind<X>, usize),
    Map(ValueKind<X>, ValueKind<X>, usize),
}

impl<X: CustomValueKind> OwnerValueHeader<X> {
    pub fn get_child_count(&self) -> usize {
        match self {
            OwnerValueHeader::Root => 1,
            OwnerValueHeader::Tuple(size) => *size,
            OwnerValueHeader::EnumVariant(_, size) => *size,
            OwnerValueHeader::Array(_, size) => *size,
            OwnerValueHeader::Map(_, _, size) => *size * 2,
        }
    }

    pub fn get_implicit_child_value_kind(&self, index: usize) -> Option<ValueKind<X>> {
        match self {
            OwnerValueHeader::Root => None,
            OwnerValueHeader::Tuple(_) => None,
            OwnerValueHeader::EnumVariant(_, _) => None,
            OwnerValueHeader::Array(element_value_kind, _) => Some(*element_value_kind),
            OwnerValueHeader::Map(key, value, _) => {
                Some(if index % 2 == 0 { *key } else { *value })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisitTerminalValue<'a> {
    pub value: TerminalValue<'a>,
    pub stack_depth: u8,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalValue<'a> {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisitTerminalValueSlice<'a> {
    pub value_slice: TerminalValueSlice<'a>,
    pub stack_depth: u8,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalValueSlice<'a> {
    U8(&'a [u8]),
}

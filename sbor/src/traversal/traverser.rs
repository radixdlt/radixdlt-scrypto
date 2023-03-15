use super::*;
use crate::decoder::PayloadTraverser;
use crate::rust::prelude::*;
use crate::value_kind::*;
use crate::*;

pub trait CustomTraversal: Copy + Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;
    type CustomTerminalValueRef<'de>: CustomTerminalValueRef<
        CustomValueKind = Self::CustomValueKind,
    >;

    fn decode_custom_value_body<'de, R>(
        custom_value_kind: Self::CustomValueKind,
        reader: &mut R,
    ) -> Result<Self::CustomTerminalValueRef<'de>, DecodeError>
    where
        R: PayloadTraverser<'de, Self::CustomValueKind>;
}

pub trait CustomTerminalValueRef: Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerState<C: CustomTraversal> {
    pub container_header: ContainerHeader<C>,
    pub container_start_offset: usize,
    pub container_child_count: usize,
    pub next_child_index: usize,
}

impl<C: CustomTraversal> ContainerState<C> {
    pub fn current_child_index(&self) -> usize {
        self.next_child_index - 1
    }
}

/// The `VecTraverser` is for streamed decoding of a payload.
/// It turns payload decoding into a pull-based event stream.
///
/// The caller is responsible for stopping calling `next_event` after an Error or End event.
pub struct VecTraverser<'de, C: CustomTraversal> {
    max_depth: usize,
    check_exact_end: bool,
    decoder: VecDecoder<'de, C::CustomValueKind>,
    container_stack: Vec<ContainerState<C>>,
    next_event_override: NextEventOverride,
}

#[derive(Debug, Clone, Copy)]
pub enum NextEventOverride {
    ReadPrefix(u8),
    ReadRootValue,
    ReadBytes(usize),
    None,
}

#[macro_export]
macro_rules! terminal_value_from_body {
    ($self: expr, $value_type: ident, $type: ident, $start_offset: expr, $value_kind: expr) => {{
        terminal_value!(
            $self,
            $value_type,
            $start_offset,
            $type::decode_body_with_value_kind(&mut $self.decoder, $value_kind)
        )
    }};
}

#[macro_export]
macro_rules! terminal_value {
    ($self: expr, $value_type: ident, $start_offset: expr, $decoded: expr) => {{
        match $decoded {
            Ok(value) => LocatedTraversalEvent {
                event: TraversalEvent::TerminalValue(TerminalValueRef::$value_type(value)),
                location: Location {
                    start_offset: $start_offset,
                    end_offset: $self.get_offset(),
                    ancestor_path: &$self.container_stack,
                },
            },
            Err(error) => $self.map_error($start_offset, error),
        }
    }};
}

#[macro_export]
macro_rules! return_if_error {
    ($self: expr, $result: expr) => {{
        match $result {
            Ok(value) => value,
            Err(error) => return $self.map_error($self.get_offset(), error),
        }
    }};
}

impl<'de, T: CustomTraversal> VecTraverser<'de, T> {
    pub fn new(
        input: &'de [u8],
        max_depth: usize,
        payload_prefix: Option<u8>,
        check_exact_end: bool,
    ) -> Self {
        Self {
            decoder: VecDecoder::new(input, max_depth),
            container_stack: Vec::with_capacity(max_depth),
            max_depth,
            next_event_override: match payload_prefix {
                Some(prefix) => NextEventOverride::ReadPrefix(prefix),
                None => NextEventOverride::ReadRootValue,
            },
            check_exact_end,
        }
    }

    pub fn next_event<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        match self.next_event_override.clone() {
            NextEventOverride::ReadPrefix(expected_prefix) => {
                self.next_event_override = NextEventOverride::ReadRootValue;
                self.read_payload_prefix(expected_prefix)
            }
            NextEventOverride::ReadRootValue => {
                self.next_event_override = NextEventOverride::None;
                self.read_root_value()
            }
            NextEventOverride::ReadBytes(size) => {
                self.next_event_override = NextEventOverride::None;
                self.read_bytes_event_override(size)
            }
            NextEventOverride::None => {
                let parent = self.container_stack.last();
                match parent {
                    Some(parent) => {
                        if parent.next_child_index >= parent.container_child_count {
                            self.exit_container()
                        } else {
                            self.read_child_value()
                        }
                    }
                    None => self.read_end(),
                }
            }
        }
    }

    pub fn read_payload_prefix<'t>(
        &'t mut self,
        expected_prefix: u8,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let start_offset = self.get_offset();
        return_if_error!(
            self,
            self.decoder.read_and_check_payload_prefix(expected_prefix)
        );
        LocatedTraversalEvent {
            event: TraversalEvent::PayloadPrefix,
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                ancestor_path: &self.container_stack,
            },
        }
    }

    fn enter_container<'t>(
        &'t mut self,
        start_offset: usize,
        container_header: ContainerHeader<T>,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let child_count = container_header.get_child_count();

        self.container_stack.push(ContainerState {
            container_header,
            container_start_offset: start_offset,
            container_child_count: child_count,
            next_child_index: 0,
        });

        // Check depth: either container stack overflows or children of this container will overflow.
        if self.container_stack.len() > self.max_depth
            || self.container_stack.len() == self.max_depth && child_count > 0
        {
            return self.map_error(start_offset, DecodeError::MaxDepthExceeded(self.max_depth));
        }

        LocatedTraversalEvent {
            event: TraversalEvent::ContainerStart(container_header),
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                ancestor_path: &self.container_stack[0..self.container_stack.len() - 1],
            },
        }
    }

    fn exit_container<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let container = self.container_stack.pop().unwrap();
        LocatedTraversalEvent {
            event: TraversalEvent::ContainerEnd(container.container_header),
            location: Location {
                start_offset: container.container_start_offset,
                end_offset: self.get_offset(),
                ancestor_path: &self.container_stack,
            },
        }
    }

    fn read_root_value<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let start_offset = self.decoder.get_offset();
        let value_kind = return_if_error!(self, self.decoder.read_value_kind());
        self.next_value(start_offset, value_kind)
    }

    fn read_child_value<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let start_offset = self.decoder.get_offset();
        let parent = self.container_stack.last_mut().unwrap();
        let value_kind = parent
            .container_header
            .get_implicit_child_value_kind(parent.next_child_index);
        let value_kind = match value_kind {
            Some(value_kind) => value_kind,
            None => return_if_error!(self, self.decoder.read_value_kind()),
        };
        parent.next_child_index += 1;
        self.next_value(start_offset, value_kind)
    }

    fn next_value<'t>(
        &'t mut self,
        start_offset: usize,
        value_kind: ValueKind<T::CustomValueKind>,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        match value_kind {
            ValueKind::Bool => {
                terminal_value_from_body!(self, Bool, bool, start_offset, value_kind)
            }
            ValueKind::I8 => {
                terminal_value_from_body!(self, I8, i8, start_offset, value_kind)
            }
            ValueKind::I16 => {
                terminal_value_from_body!(self, I16, i16, start_offset, value_kind)
            }
            ValueKind::I32 => {
                terminal_value_from_body!(self, I32, i32, start_offset, value_kind)
            }
            ValueKind::I64 => {
                terminal_value_from_body!(self, I64, i64, start_offset, value_kind)
            }
            ValueKind::I128 => {
                terminal_value_from_body!(self, I128, i128, start_offset, value_kind)
            }
            ValueKind::U8 => {
                terminal_value_from_body!(self, U8, u8, start_offset, value_kind)
            }
            ValueKind::U16 => {
                terminal_value_from_body!(self, U16, u16, start_offset, value_kind)
            }
            ValueKind::U32 => {
                terminal_value_from_body!(self, U32, u32, start_offset, value_kind)
            }
            ValueKind::U64 => {
                terminal_value_from_body!(self, U64, u64, start_offset, value_kind)
            }
            ValueKind::U128 => {
                terminal_value_from_body!(self, U128, u128, start_offset, value_kind)
            }
            ValueKind::String => {
                terminal_value!(self, String, start_offset, self.decode_string_body())
            }
            ValueKind::Array => self.decode_array_header(start_offset),
            ValueKind::Map => self.decode_map_header(start_offset),
            ValueKind::Enum => self.decode_enum_variant_header(start_offset),
            ValueKind::Tuple => self.decode_tuple_header(start_offset),
            ValueKind::Custom(custom_value_kind) => {
                let result = T::decode_custom_value_body(custom_value_kind, &mut self.decoder);
                let location = Location {
                    start_offset: start_offset,
                    end_offset: self.get_offset(),
                    ancestor_path: &self.container_stack,
                };
                let event = match result {
                    Ok(custom_value) => {
                        TraversalEvent::TerminalValue(TerminalValueRef::Custom(custom_value))
                    }
                    Err(decode_error) => TraversalEvent::DecodeError(decode_error),
                };
                LocatedTraversalEvent { location, event }
            }
        }
    }

    fn map_error<'t>(
        &'t self,
        start_offset: usize,
        error: DecodeError,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        LocatedTraversalEvent {
            event: TraversalEvent::DecodeError(error),
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                ancestor_path: &self.container_stack,
            },
        }
    }

    #[inline]
    fn get_offset(&self) -> usize {
        self.decoder.get_offset()
    }

    fn decode_string_body(&mut self) -> Result<&'de str, DecodeError> {
        let size = self.decoder.read_size()?;
        let bytes_slices = self.decoder.read_slice_from_payload(size)?;
        sbor::rust::str::from_utf8(bytes_slices).map_err(|_| DecodeError::InvalidUtf8)
    }

    fn decode_enum_variant_header<'t>(
        &'t mut self,
        start_offset: usize,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let variant = return_if_error!(self, self.decoder.read_byte());
        let length = return_if_error!(self, self.decoder.read_size());
        self.enter_container(
            start_offset,
            ContainerHeader::EnumVariant(EnumVariantHeader { variant, length }),
        )
    }

    fn decode_tuple_header<'t>(
        &'t mut self,
        start_offset: usize,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let length = return_if_error!(self, self.decoder.read_size());
        self.enter_container(start_offset, ContainerHeader::Tuple(TupleHeader { length }))
    }

    fn decode_array_header<'t>(
        &'t mut self,
        start_offset: usize,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let element_value_kind = return_if_error!(self, self.decoder.read_value_kind());
        let length = return_if_error!(self, self.decoder.read_size());
        if element_value_kind == ValueKind::U8 && length > 0 {
            self.next_event_override = NextEventOverride::ReadBytes(length);
        }
        self.enter_container(
            start_offset,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind,
                length,
            }),
        )
    }

    fn decode_map_header<'t>(
        &'t mut self,
        start_offset: usize,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let key_value_kind = return_if_error!(self, self.decoder.read_value_kind());
        let value_value_kind = return_if_error!(self, self.decoder.read_value_kind());
        let length = return_if_error!(self, self.decoder.read_size());
        self.enter_container(
            start_offset,
            ContainerHeader::Map(MapHeader {
                key_value_kind,
                value_value_kind,
                length,
            }),
        )
    }

    fn read_end<'t>(&'t self) -> LocatedTraversalEvent<'t, 'de, T> {
        if self.check_exact_end {
            return_if_error!(self, self.decoder.check_end());
        }
        let offset = self.decoder.get_offset();

        LocatedTraversalEvent {
            event: TraversalEvent::End,
            location: Location {
                start_offset: offset,
                end_offset: offset,
                ancestor_path: &self.container_stack,
            },
        }
    }

    fn read_bytes_event_override<'t>(
        &'t mut self,
        size: usize,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let start_offset = self.get_offset();
        let bytes = return_if_error!(self, self.decoder.read_slice_from_payload(size));
        // Set it up so that we jump to the end of the child iteration
        self.container_stack.last_mut().unwrap().next_child_index = size;
        self.next_event_override = NextEventOverride::None;
        LocatedTraversalEvent {
            event: TraversalEvent::TerminalValueBatch(TerminalValueBatchRef::U8(bytes)),
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                ancestor_path: &self.container_stack,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::prelude::*;

    use super::*;

    #[derive(Categorize, Encode)]
    struct TestStruct {
        x: u32,
    }

    #[derive(Categorize, Encode)]
    #[allow(dead_code)]
    enum TestEnum {
        A { x: u32 },
        B(u32),
        C,
    }

    #[test]
    pub fn test_exact_events_returned() {
        let payload = basic_encode(&(
            2u8,
            vec![3u8, 7u8],
            (3u32, indexmap!(16u8 => 18u32)),
            TestEnum::B(4u32),
            Vec::<u8>::new(),
            Vec::<i32>::new(),
            vec![vec![(-2i64,)]],
        ))
        .unwrap();

        let mut traverser = basic_payload_traverser(&payload);

        // Start:
        next_event_is_payload_prefix(&mut traverser, 0, 1);
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Tuple(TupleHeader { length: 7 }),
            1,
            1,
            3,
        );
        // First line
        next_event_is_terminal_value(&mut traverser, TerminalValueRef::U8(2), 2, 3, 5);
        // Second line
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::U8,
                length: 2,
            }),
            2,
            5,
            8,
        );
        next_event_is_terminal_value_slice(
            &mut traverser,
            TerminalValueBatchRef::U8(&[3u8, 7u8]),
            3,
            8,
            10,
        );
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::U8,
                length: 2,
            }),
            2,
            5,
            10,
        );
        // Third line
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Tuple(TupleHeader { length: 2 }),
            2,
            10,
            12,
        );
        next_event_is_terminal_value(&mut traverser, TerminalValueRef::U32(3), 3, 12, 17);
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Map(MapHeader {
                key_value_kind: ValueKind::U8,
                value_value_kind: ValueKind::U32,
                length: 1,
            }),
            3,
            17,
            21,
        );
        next_event_is_terminal_value(&mut traverser, TerminalValueRef::U8(16), 4, 21, 22);
        next_event_is_terminal_value(&mut traverser, TerminalValueRef::U32(18), 4, 22, 26);
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Map(MapHeader {
                key_value_kind: ValueKind::U8,
                value_value_kind: ValueKind::U32,
                length: 1,
            }),
            3,
            17,
            26,
        );
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Tuple(TupleHeader { length: 2 }),
            2,
            10,
            26,
        );
        // Fourth line
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::EnumVariant(EnumVariantHeader {
                variant: 1,
                length: 1,
            }),
            2,
            26,
            29,
        );
        next_event_is_terminal_value(&mut traverser, TerminalValueRef::U32(4), 3, 29, 34);
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::EnumVariant(EnumVariantHeader {
                variant: 1,
                length: 1,
            }),
            2,
            26,
            34,
        );
        // Fifth line - empty Vec<u8> - no bytes event is output
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::U8,
                length: 0,
            }),
            2,
            34,
            37,
        );
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::U8,
                length: 0,
            }),
            2,
            34,
            37,
        );
        // Sixth line - empty Vec<i32>
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::I32,
                length: 0,
            }),
            2,
            37,
            40,
        );
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::I32,
                length: 0,
            }),
            2,
            37,
            40,
        );
        // Seventh line - Vec<Vec<(i64)>>
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::Array,
                length: 1,
            }),
            2,
            40,
            43,
        );
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::Tuple,
                length: 1,
            }),
            3,
            43,
            45,
        );
        next_event_is_container_start_header(
            &mut traverser,
            ContainerHeader::Tuple(TupleHeader { length: 1 }),
            4,
            45,
            46,
        );
        next_event_is_terminal_value(&mut traverser, TerminalValueRef::I64(-2), 5, 46, 55);
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Tuple(TupleHeader { length: 1 }),
            4,
            45,
            55,
        );
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::Tuple,
                length: 1,
            }),
            3,
            43,
            55,
        );
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind: ValueKind::Array,
                length: 1,
            }),
            2,
            40,
            55,
        );

        // End
        next_event_is_container_end(
            &mut traverser,
            ContainerHeader::Tuple(TupleHeader { length: 7 }),
            1,
            1,
            55,
        );
        next_event_is_end(&mut traverser, 55, 55);
    }

    pub fn next_event_is_payload_prefix(
        traverser: &mut BasicTraverser,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let LocatedTraversalEvent {
            event: TraversalEvent::PayloadPrefix,
            location: Location {
                start_offset,
                end_offset,
                ..
            },
        } = event else {
            panic!("Invalid event - expected PayloadPrefix, was {:?}", event);
        };
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
        assert!(event.location.ancestor_path.is_empty());
    }

    pub fn next_event_is_container_start_header(
        traverser: &mut BasicTraverser,
        expected_header: ContainerHeader<NoCustomTraversal>,
        expected_depth: usize,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.ancestor_path.len() + 1;
        let LocatedTraversalEvent {
            event: TraversalEvent::ContainerStart(header),
            location: Location {
                start_offset,
                end_offset,
                ..
            },
        } = event else {
            panic!("Invalid event - expected ContainerStart, was {:?}", event);
        };
        assert_eq!(header, expected_header);
        assert_eq!(sbor_depth, expected_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_container_end(
        traverser: &mut BasicTraverser,
        expected_header: ContainerHeader<NoCustomTraversal>,
        expected_depth: usize,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.ancestor_path.len() + 1;
        let LocatedTraversalEvent {
            event: TraversalEvent::ContainerEnd(header),
            location: Location {
                start_offset,
                end_offset,
                ..
            },
        } = event else {
            panic!("Invalid event - expected ContainerEnd, was {:?}", event);
        };
        assert_eq!(header, expected_header);
        assert_eq!(sbor_depth, expected_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_terminal_value<'de>(
        traverser: &mut BasicTraverser<'de>,
        expected_value: TerminalValueRef<'de, NoCustomTraversal>,
        expected_child_depth: usize,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.ancestor_path.len() + 1;
        let LocatedTraversalEvent {
            event: TraversalEvent::TerminalValue(value),
            location: Location {
                start_offset,
                end_offset,
                ..
            },
        } = event else {
            panic!("Invalid event - expected TerminalValue, was {:?}", event);
        };
        assert_eq!(value, expected_value);
        assert_eq!(sbor_depth, expected_child_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_terminal_value_slice<'de>(
        traverser: &mut BasicTraverser<'de>,
        expected_value_batch: TerminalValueBatchRef<'de>,
        expected_child_depth: usize,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.ancestor_path.len() + 1;
        let LocatedTraversalEvent {
            event: TraversalEvent::TerminalValueBatch(value_batch),
            location: Location {
                start_offset,
                end_offset,
                ..
            },
        } = event else {
            panic!("Invalid event - expected TerminalValueBatch, was {:?}", event);
        };
        assert_eq!(value_batch, expected_value_batch);
        assert_eq!(sbor_depth, expected_child_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_end(
        traverser: &mut BasicTraverser,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let LocatedTraversalEvent {
            event: TraversalEvent::End,
            location: Location {
                start_offset,
                end_offset,
                ..
            },
        } = event else {
            panic!("Invalid event - expected End, was {:?}", event);
        };
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
        assert!(event.location.ancestor_path.is_empty());
    }
}

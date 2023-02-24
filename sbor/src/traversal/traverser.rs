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
    type CustomTerminalValueBatchRef<'de>: CustomTerminalValueBatchRef<
        CustomValueKind = Self::CustomValueKind,
    >;
    type CustomContainerHeader: CustomContainerHeader<CustomValueKind = Self::CustomValueKind>;
    type CustomValueTraverser: CustomValueTraverser<CustomTraversal = Self>;

    fn new_value_traversal(
        custom_value_kind: Self::CustomValueKind,
        parent_relationship: ParentRelationship,
        start_offset: usize,
        current_depth: u8,
        max_depth: u8,
    ) -> Self::CustomValueTraverser;
}

pub trait CustomTerminalValueRef: Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind;
}

pub trait CustomTerminalValueBatchRef: Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind;
}

pub trait CustomContainerHeader: Copy + Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;
    fn get_child_count(&self) -> u32;
    fn get_implicit_child_value_kind(
        &self,
        index: u32,
    ) -> (ParentRelationship, Option<ValueKind<Self::CustomValueKind>>);
}

/// A `CustomValueTraverser` is responsible for emitting traversal events for a single custom value - and therefore is either:
/// - Emitting a single event for a terminal value at the current depth
/// - Emitting multiple events representing a single container value, which will return to the current depth
///
/// If traversing a container type, the `CustomValueTraverser` is responsible for keeping track of the depth itself,
/// and erroring if the max depth is exceeded.
pub trait CustomValueTraverser {
    type CustomTraversal: CustomTraversal;

    fn next_event<
        't,
        'de,
        R: PayloadTraverser<'de, <Self::CustomTraversal as CustomTraversal>::CustomValueKind>,
    >(
        &mut self,
        container_stack: &'t mut Vec<ContainerChild<Self::CustomTraversal>>,
        reader: &mut R,
    ) -> LocatedTraversalEvent<'t, 'de, Self::CustomTraversal>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerChild<C: CustomTraversal> {
    pub container_header: ContainerHeader<C>,
    pub container_parent_relationship: ParentRelationship,
    pub container_start_offset: usize,
    pub container_child_count: u32,
    pub current_child_index: u32,
}

/// The `VecTraverser` is for streamed decoding of a payload.
/// It turns payload decoding into a pull-based event stream.
///
/// The caller is responsible for stopping calling `next_event` after an Error or End event.
pub struct VecTraverser<'de, C: CustomTraversal> {
    decoder: VecDecoder<'de, C::CustomValueKind>,
    container_stack: Vec<ContainerChild<C>>,
    max_depth: u8,
    next_event_override: NextEventOverride<C::CustomValueTraverser>,
    check_exact_end: bool,
}

pub enum NextEventOverride<C> {
    Prefix(u8),
    Start,
    ReadBytes(u32),
    CustomValueTraversal(C, u8),
    None,
}

#[macro_export]
macro_rules! terminal_value_from_body {
    ($self: expr, $value_type: ident, $type: ident, $parent_relationship: expr, $start_offset: expr, $value_kind: expr) => {{
        terminal_value!(
            $self,
            $value_type,
            $parent_relationship,
            $start_offset,
            $type::decode_body_with_value_kind(&mut $self.decoder, $value_kind)
        )
    }};
}

#[macro_export]
macro_rules! terminal_value {
    ($self: expr, $value_type: ident, $parent_relationship: expr, $start_offset: expr, $decoded: expr) => {{
        match $decoded {
            Ok(value) => LocatedTraversalEvent {
                event: TraversalEvent::TerminalValue(TerminalValueRef::$value_type(value)),
                location: Location {
                    start_offset: $start_offset,
                    end_offset: $self.get_offset(),
                    parent_relationship: $parent_relationship,
                    ancestor_path: &$self.container_stack,
                },
            },
            Err(error) => $self.map_error($start_offset, $parent_relationship, error),
        }
    }};
}

#[macro_export]
macro_rules! return_if_error {
    ($self: expr, $parent_relationship: expr, $result: expr) => {{
        match $result {
            Ok(value) => value,
            Err(error) => return $self.map_error($self.get_offset(), $parent_relationship, error),
        }
    }};
}

impl<'de, T: CustomTraversal> VecTraverser<'de, T> {
    pub fn new(
        input: &'de [u8],
        max_depth: u8,
        payload_prefix: Option<u8>,
        check_exact_end: bool,
    ) -> Self {
        Self {
            decoder: VecDecoder::new(input, max_depth),
            container_stack: Vec::with_capacity(max_depth as usize),
            max_depth,
            next_event_override: match payload_prefix {
                Some(prefix) => NextEventOverride::Prefix(prefix),
                None => NextEventOverride::Start,
            },
            check_exact_end,
        }
    }

    pub fn read_and_check_payload_prefix<'t>(
        &'t mut self,
        expected_prefix: u8,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let parent_relationship = ParentRelationship::NotInValueModel;
        let start_offset = self.get_offset();
        return_if_error!(
            self,
            parent_relationship,
            self.decoder.read_and_check_payload_prefix(expected_prefix)
        );
        LocatedTraversalEvent {
            event: TraversalEvent::PayloadPrefix,
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                parent_relationship,
                ancestor_path: &self.container_stack,
            },
        }
    }

    pub fn next_event<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let event = match &mut self.next_event_override {
            NextEventOverride::Prefix(expected_prefix) => {
                let expected_prefix = *expected_prefix;
                self.next_event_override = NextEventOverride::Start;
                self.read_and_check_payload_prefix(expected_prefix)
            }
            NextEventOverride::Start => {
                self.next_event_override = NextEventOverride::None;
                self.root_value()
            }
            NextEventOverride::CustomValueTraversal(_, _) => self.custom_event_override(),
            NextEventOverride::ReadBytes(_) => self.read_bytes_event_override(),
            NextEventOverride::None => {
                let parent = self.container_stack.last();
                match parent {
                    Some(parent) => {
                        if parent.current_child_index >= parent.container_child_count {
                            self.exit_container()
                        } else {
                            self.child_value()
                        }
                    }
                    None => self.end_event(),
                }
            }
        };
        event
    }

    fn enter_container<'t>(
        &'t mut self,
        start_offset: usize,
        container_parent_relationship: ParentRelationship,
        container_header: ContainerHeader<T>,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let stack_depth_of_container = self.get_sbor_depth_for_next_value();
        if stack_depth_of_container >= self.max_depth {
            // We're already at the max depth, we can't add any more containers to the stack
            return self.map_error(
                start_offset,
                container_parent_relationship,
                DecodeError::MaxDepthExceeded(self.max_depth),
            );
        }
        self.container_stack.push(ContainerChild {
            container_header,
            container_parent_relationship,
            container_start_offset: start_offset,
            container_child_count: container_header.get_child_count(),
            current_child_index: 0,
        });
        LocatedTraversalEvent {
            event: TraversalEvent::ContainerStart(container_header),
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                parent_relationship: container_parent_relationship,
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
                parent_relationship: container.container_parent_relationship,
                ancestor_path: &self.container_stack,
            },
        }
    }

    fn root_value<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let start_offset = self.decoder.get_offset();
        let parent_relationship = ParentRelationship::Root;
        let value_kind =
            return_if_error!(self, parent_relationship, self.decoder.read_value_kind());
        self.next_value(start_offset, parent_relationship, value_kind)
    }

    fn child_value<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let start_offset = self.decoder.get_offset();
        let current_child = self.container_stack.last_mut().unwrap();
        let (relationship, value_kind) = current_child
            .container_header
            .get_implicit_child_value_kind(current_child.current_child_index);
        let value_kind = match value_kind {
            Some(value_kind) => value_kind,
            None => return_if_error!(self, relationship, self.decoder.read_value_kind()),
        };
        current_child.current_child_index += 1;
        self.next_value(start_offset, relationship, value_kind)
    }

    fn next_value<'t>(
        &'t mut self,
        start_offset: usize,
        relationship: ParentRelationship,
        value_kind: ValueKind<T::CustomValueKind>,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        match value_kind {
            ValueKind::Bool => {
                terminal_value_from_body!(self, Bool, bool, relationship, start_offset, value_kind)
            }
            ValueKind::I8 => {
                terminal_value_from_body!(self, I8, i8, relationship, start_offset, value_kind)
            }
            ValueKind::I16 => {
                terminal_value_from_body!(self, I16, i16, relationship, start_offset, value_kind)
            }
            ValueKind::I32 => {
                terminal_value_from_body!(self, I32, i32, relationship, start_offset, value_kind)
            }
            ValueKind::I64 => {
                terminal_value_from_body!(self, I64, i64, relationship, start_offset, value_kind)
            }
            ValueKind::I128 => {
                terminal_value_from_body!(self, I128, i128, relationship, start_offset, value_kind)
            }
            ValueKind::U8 => {
                terminal_value_from_body!(self, U8, u8, relationship, start_offset, value_kind)
            }
            ValueKind::U16 => {
                terminal_value_from_body!(self, U16, u16, relationship, start_offset, value_kind)
            }
            ValueKind::U32 => {
                terminal_value_from_body!(self, U32, u32, relationship, start_offset, value_kind)
            }
            ValueKind::U64 => {
                terminal_value_from_body!(self, U64, u64, relationship, start_offset, value_kind)
            }
            ValueKind::U128 => {
                terminal_value_from_body!(self, U128, u128, relationship, start_offset, value_kind)
            }
            ValueKind::String => {
                terminal_value!(
                    self,
                    String,
                    relationship,
                    start_offset,
                    self.decode_string_slice()
                )
            }
            ValueKind::Array => self.decode_array_header(start_offset, relationship),
            ValueKind::Map => self.decode_map_header(start_offset, relationship),
            ValueKind::Enum => self.decode_enum_variant_header(start_offset, relationship),
            ValueKind::Tuple => self.decode_tuple_header(start_offset, relationship),
            ValueKind::Custom(custom_value_kind) => {
                let depth = self.get_sbor_depth_for_next_value();
                self.next_event_override = NextEventOverride::CustomValueTraversal(
                    T::new_value_traversal(
                        custom_value_kind,
                        relationship,
                        start_offset,
                        depth,
                        self.max_depth,
                    ),
                    depth,
                );
                self.custom_event_override()
            }
        }
    }

    fn map_error<'t>(
        &'t self,
        start_offset: usize,
        parent_relationship: ParentRelationship,
        error: DecodeError,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        LocatedTraversalEvent {
            event: TraversalEvent::DecodeError(error),
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                parent_relationship,
                ancestor_path: &self.container_stack,
            },
        }
    }

    #[inline]
    fn get_sbor_depth_for_next_value(&self) -> u8 {
        // SAFE CASTING: The invariant self.container_stack.len() + 1 <= max_depth is maintained in `enter_container` before
        // we push to the stack. As `max_depth` is a u8, this can't overflow.
        (self.container_stack.len() as u8) + 1
    }

    #[inline]
    fn get_offset(&self) -> usize {
        self.decoder.get_offset()
    }

    fn decode_string_slice(&mut self) -> Result<&'de str, DecodeError> {
        self.decoder.read_and_check_value_kind(ValueKind::String)?;
        let size = self.decoder.read_size()?;
        let bytes_slices = self.decoder.read_slice_from_payload(size)?;
        sbor::rust::str::from_utf8(bytes_slices).map_err(|_| DecodeError::InvalidUtf8)
    }

    fn decode_enum_variant_header<'t>(
        &'t mut self,
        start_offset: usize,
        parent_relationship: ParentRelationship,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let variant = return_if_error!(self, parent_relationship, self.decoder.read_byte());
        let length = return_if_error!(self, parent_relationship, self.decoder.read_size_u32());
        self.enter_container(
            start_offset,
            parent_relationship,
            ContainerHeader::EnumVariant(EnumVariantHeader { variant, length }),
        )
    }

    fn decode_tuple_header<'t>(
        &'t mut self,
        start_offset: usize,
        parent_relationship: ParentRelationship,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let length = return_if_error!(self, parent_relationship, self.decoder.read_size_u32());
        self.enter_container(
            start_offset,
            parent_relationship,
            ContainerHeader::Tuple(TupleHeader { length }),
        )
    }

    fn decode_array_header<'t>(
        &'t mut self,
        start_offset: usize,
        parent_relationship: ParentRelationship,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let element_value_kind =
            return_if_error!(self, parent_relationship, self.decoder.read_value_kind());
        let length = return_if_error!(self, parent_relationship, self.decoder.read_size_u32());
        if element_value_kind == ValueKind::U8 && length > 0 {
            self.next_event_override = NextEventOverride::ReadBytes(length);
        }
        self.enter_container(
            start_offset,
            parent_relationship,
            ContainerHeader::Array(ArrayHeader {
                element_value_kind,
                length,
            }),
        )
    }

    fn decode_map_header<'t>(
        &'t mut self,
        start_offset: usize,
        parent_relationship: ParentRelationship,
    ) -> LocatedTraversalEvent<'t, 'de, T> {
        let key_value_kind =
            return_if_error!(self, parent_relationship, self.decoder.read_value_kind());
        let value_value_kind =
            return_if_error!(self, parent_relationship, self.decoder.read_value_kind());
        let length = return_if_error!(self, parent_relationship, self.decoder.read_size_u32());
        self.enter_container(
            start_offset,
            parent_relationship,
            ContainerHeader::Map(MapHeader {
                key_value_kind,
                value_value_kind,
                length,
            }),
        )
    }

    fn end_event<'t>(&'t self) -> LocatedTraversalEvent<'t, 'de, T> {
        let parent_relationship = ParentRelationship::NotInValueModel;
        if self.check_exact_end {
            return_if_error!(self, parent_relationship, self.decoder.check_end());
        }
        let offset = self.decoder.get_offset();

        LocatedTraversalEvent {
            event: TraversalEvent::End,
            location: Location {
                start_offset: offset,
                end_offset: offset,
                parent_relationship,
                ancestor_path: &self.container_stack,
            },
        }
    }

    fn custom_event_override<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let NextEventOverride::CustomValueTraversal(custom_traverser, entry_depth) = &mut self.next_event_override else {
            panic!("self.next_event_override expected to be NextEventOverride::Custom to hit this code")
        };
        let traversal_event =
            custom_traverser.next_event(&mut self.container_stack, &mut self.decoder);
        // We assume the custom traverser is for a single value - and therefore is either:
        // - Emitting a single event for a terminal value at the current depth
        // - Emitting multiple events representing a single container value, which will return to the current depth
        // Either way, when the traversal_event's next sbor depth matches the sbor depth when the custom traverser was entered,
        // this means that the custom traverser has returned
        if traversal_event.get_next_sbor_depth() == *entry_depth {
            self.next_event_override = NextEventOverride::None;
        }
        traversal_event
    }

    fn read_bytes_event_override<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let NextEventOverride::ReadBytes(size) = self.next_event_override else {
            panic!("self.next_event_override expected to be NextEventOverride::ReadBytes to hit this code")
        };
        let start_offset = self.get_offset();
        let parent_relationship = ParentRelationship::ArrayElementBatch {
            from_index: 0,
            to_index: size,
        };
        let bytes = return_if_error!(
            self,
            parent_relationship,
            self.decoder.read_slice_from_payload(size as usize)
        );
        // Set it up so that we jump to the end of the child iteration
        self.container_stack.last_mut().unwrap().current_child_index = size;
        self.next_event_override = NextEventOverride::None;
        LocatedTraversalEvent {
            event: TraversalEvent::TerminalValueBatch(TerminalValueBatchRef::U8(bytes)),
            location: Location {
                start_offset,
                end_offset: self.get_offset(),
                parent_relationship,
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
        next_event_is_payload_prefix(&mut traverser, 0, 0, 1);
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
        next_event_is_end(&mut traverser, 0, 55, 55);
    }

    pub fn next_event_is_payload_prefix(
        traverser: &mut BasicTraverser,
        expected_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.get_sbor_depth();
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
        assert_eq!(sbor_depth, expected_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_container_start_header(
        traverser: &mut BasicTraverser,
        expected_header: ContainerHeader<NoCustomTraversal>,
        expected_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.get_sbor_depth();
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
        expected_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.get_sbor_depth();
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
        expected_stack_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.get_sbor_depth();
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
        assert_eq!(sbor_depth, expected_stack_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_terminal_value_slice<'de>(
        traverser: &mut BasicTraverser<'de>,
        expected_value_batch: TerminalValueBatchRef<'de, NoCustomTraversal>,
        expected_stack_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.get_sbor_depth();
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
        assert_eq!(sbor_depth, expected_stack_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_end(
        traverser: &mut BasicTraverser,
        expected_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let sbor_depth = event.location.get_sbor_depth();
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
        assert_eq!(sbor_depth, expected_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }
}

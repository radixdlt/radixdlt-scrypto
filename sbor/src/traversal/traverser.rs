use super::*;
use crate::decoder::PayloadTraverser;
use crate::rust::marker::PhantomData;
use crate::rust::str;
use crate::value_kind::*;
use crate::*;

pub trait CustomTraverser<'de, R: PayloadTraverser<'de, Self::CustomValueKind>> {
    type CustomTraversalEvent;
    type CustomValueKind: CustomValueKind;

    fn new_traversal(custom_value_kind: Self::CustomValueKind) -> Self;
    fn next_event(
        &mut self,
        reader: &mut R,
    ) -> Result<
        (
            TraversalEvent<'de, Self::CustomValueKind, Self::CustomTraversalEvent>,
            bool,
        ),
        DecodeError,
    >;
}

pub struct CurrentChild<X: CustomValueKind> {
    pub owner_header: OwnerValueHeader<X>,
    pub start_offset: usize,
    pub total_child_count: usize,
    pub current_child_index: usize,
}

/// The `VecTraverser` is for streamed decoding of a payload.
/// It turns payload decoding into a pull-based event stream
pub struct VecTraverser<
    'de,
    X: CustomValueKind,
    C: CustomTraverser<'de, VecDecoder<'de, X, MAX_DEPTH>, CustomValueKind = X>,
    const MAX_DEPTH: u8,
> {
    decoder: VecDecoder<'de, C::CustomValueKind, MAX_DEPTH>,
    stack: Vec<CurrentChild<X>>,
    next_event_override: NextEventOverride<C>,
    phantom_custom_value_kind: PhantomData<X>,
}

pub enum NextEventOverride<C> {
    Start,
    Custom(C),
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
            Ok(value) => TraversalEvent::VisitTerminalValue(VisitTerminalValue {
                value: TerminalValue::$value_type(value),
                start_offset: $start_offset,
                stack_depth: $self.get_stack_depth(),
                end_offset: $self.get_offset(),
            }),
            Err(error) => $self.map_error(error),
        }
    }};
}

#[macro_export]
macro_rules! err_to_event {
    ($self: expr, $result: expr) => {{
        match $result {
            Ok(value) => value,
            Err(error) => return $self.map_error(error),
        }
    }};
}

impl<
        'de,
        X: CustomValueKind,
        C: CustomTraverser<'de, VecDecoder<'de, X, MAX_DEPTH>, CustomValueKind = X>,
        const MAX_DEPTH: u8,
    > VecTraverser<'de, X, C, MAX_DEPTH>
{
    pub fn new(input: &'de [u8]) -> Self {
        Self {
            decoder: VecDecoder::new(input),
            stack: Vec::with_capacity(MAX_DEPTH as usize),
            next_event_override: NextEventOverride::Start,
            phantom_custom_value_kind: PhantomData,
        }
    }

    pub fn read_and_check_payload_prefix(
        &mut self,
        expected_prefix: u8,
    ) -> Result<(), DecodeError> {
        self.decoder.read_and_check_payload_prefix(expected_prefix)
    }

    pub fn next_event(&mut self) -> TraversalEvent<'_, X, C::CustomTraversalEvent> {
        match &mut self.next_event_override {
            NextEventOverride::Start => self.start_event_override(),
            NextEventOverride::Custom(_) => self.custom_event_override(),
            NextEventOverride::ReadBytes(_) => self.read_bytes_event_override(),
            NextEventOverride::None => {
                if self.is_end_of_current_depth() {
                    self.exit_child()
                } else {
                    self.next_value_start()
                }
            }
        }
    }

    fn is_end_of_current_depth(&self) -> bool {
        let current_child = self.stack.last().unwrap();
        current_child.current_child_index >= current_child.total_child_count
    }

    fn enter_child(
        &mut self,
        start_offset: usize,
        owner_header: OwnerValueHeader<X>,
    ) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        if self.stack.len() >= MAX_DEPTH as usize {
            return self.map_error(DecodeError::MaxDepthExceeded(MAX_DEPTH));
        }
        let previous_stack_depth = self.get_stack_depth();
        self.stack.push(CurrentChild {
            owner_header,
            start_offset,
            total_child_count: owner_header.get_child_count(),
            current_child_index: 0,
        });
        TraversalEvent::StartOwnerValue(VisitOwnerValueHeader {
            header: owner_header,
            start_offset,
            end_offset: self.get_offset(),
            stack_depth: previous_stack_depth,
        })
    }

    fn exit_child(&mut self) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        let child = self.stack.pop().unwrap();
        let resultant_stack_depth = self.get_stack_depth();
        if resultant_stack_depth == 0 {
            err_to_event!(self, self.decoder.check_end());
        }
        TraversalEvent::EndOwnerValue(VisitFullOwnerValue {
            header: child.owner_header,
            start_offset: child.start_offset,
            end_offset: self.get_offset(),
            stack_depth: resultant_stack_depth,
        })
    }

    fn next_value_start(&mut self) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        let start_offset = self.decoder.get_offset();
        let current_child = self.stack.last_mut().unwrap();
        let value_kind = match current_child
            .owner_header
            .get_implicit_child_value_kind(current_child.current_child_index)
        {
            Some(value_kind) => value_kind,
            None => err_to_event!(self, self.decoder.read_value_kind()),
        };
        current_child.current_child_index += 1;
        let event = match value_kind {
            ValueKind::Bool => {
                terminal_value_from_body!(self, Bool, bool, start_offset, value_kind)
            }
            ValueKind::I8 => terminal_value_from_body!(self, I8, i8, start_offset, value_kind),
            ValueKind::I16 => terminal_value_from_body!(self, I16, i16, start_offset, value_kind),
            ValueKind::I32 => terminal_value_from_body!(self, I32, i32, start_offset, value_kind),
            ValueKind::I64 => terminal_value_from_body!(self, I64, i64, start_offset, value_kind),
            ValueKind::I128 => {
                terminal_value_from_body!(self, I128, i128, start_offset, value_kind)
            }
            ValueKind::U8 => terminal_value_from_body!(self, U8, u8, start_offset, value_kind),
            ValueKind::U16 => terminal_value_from_body!(self, U16, u16, start_offset, value_kind),
            ValueKind::U32 => terminal_value_from_body!(self, U32, u32, start_offset, value_kind),
            ValueKind::U64 => terminal_value_from_body!(self, U64, u64, start_offset, value_kind),
            ValueKind::U128 => {
                terminal_value_from_body!(self, U128, u128, start_offset, value_kind)
            }
            ValueKind::String => {
                terminal_value!(self, String, start_offset, self.decode_string_slice())
            }
            ValueKind::Array => err_to_event!(self, self.decode_array_header(start_offset)),
            ValueKind::Map => err_to_event!(self, self.decode_map_header(start_offset)),
            ValueKind::Enum => err_to_event!(self, self.decode_enum_variant_header(start_offset)),
            ValueKind::Tuple => err_to_event!(self, self.decode_tuple_header(start_offset)),
            ValueKind::Custom(custom_value_kind) => {
                self.next_event_override =
                    NextEventOverride::Custom(C::new_traversal(custom_value_kind));
                self.custom_event_override()
            }
        };
        event
    }

    fn map_error(&self, error: DecodeError) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        TraversalEvent::DecodeError(DecodeErrorEvent {
            error,
            stack_depth: self.get_stack_depth(),
            offset: self.get_offset(),
        })
    }

    #[inline]
    fn get_stack_depth(&self) -> u8 {
        // Safe because stack's size is limited by MAX_DEPTH: u8
        self.stack.len() as u8
    }

    #[inline]
    fn get_offset(&self) -> usize {
        self.decoder.get_offset()
    }

    fn decode_string_slice(&mut self) -> Result<&'de str, DecodeError> {
        self.decoder.read_and_check_value_kind(ValueKind::String)?;
        let size = self.decoder.read_size()?;
        let bytes_slices = self.decoder.read_slice_from_payload(size)?;
        str::from_utf8(bytes_slices).map_err(|_| DecodeError::InvalidUtf8)
    }

    fn decode_enum_variant_header(
        &mut self,
        start_offset: usize,
    ) -> Result<TraversalEvent<'de, X, C::CustomTraversalEvent>, DecodeError> {
        let variant = self.decoder.read_byte()?;
        let size = self.decoder.read_size()?;
        Ok(self.enter_child(start_offset, OwnerValueHeader::EnumVariant(variant, size)))
    }

    fn decode_tuple_header(
        &mut self,
        start_offset: usize,
    ) -> Result<TraversalEvent<'de, X, C::CustomTraversalEvent>, DecodeError> {
        let size = self.decoder.read_size()?;
        Ok(self.enter_child(start_offset, OwnerValueHeader::Tuple(size)))
    }

    fn decode_array_header(
        &mut self,
        start_offset: usize,
    ) -> Result<TraversalEvent<'de, X, C::CustomTraversalEvent>, DecodeError> {
        let element_value_kind = self.decoder.read_value_kind()?;
        let size = self.decoder.read_size()?;
        if element_value_kind == ValueKind::U8 && size > 0 {
            self.next_event_override = NextEventOverride::ReadBytes(size);
        }
        Ok(self.enter_child(
            start_offset,
            OwnerValueHeader::Array(element_value_kind, size),
        ))
    }

    fn decode_map_header(
        &mut self,
        start_offset: usize,
    ) -> Result<TraversalEvent<'de, X, C::CustomTraversalEvent>, DecodeError> {
        let key_value_kind = self.decoder.read_value_kind()?;
        let value_value_kind = self.decoder.read_value_kind()?;
        let size = self.decoder.read_size()?;
        Ok(self.enter_child(
            start_offset,
            OwnerValueHeader::Map(key_value_kind, value_value_kind, size),
        ))
    }

    fn start_event_override(&mut self) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        self.next_event_override = NextEventOverride::None;
        self.enter_child(self.decoder.get_offset(), OwnerValueHeader::Root)
    }

    fn custom_event_override(&mut self) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        let NextEventOverride::Custom(custom_traverser) = &mut self.next_event_override else {
            unreachable!()
        };
        let (traversal_event, is_finished) =
            err_to_event!(self, custom_traverser.next_event(&mut self.decoder));
        if is_finished {
            self.next_event_override = NextEventOverride::None;
        }
        traversal_event
    }

    fn read_bytes_event_override(&mut self) -> TraversalEvent<'de, X, C::CustomTraversalEvent> {
        let NextEventOverride::ReadBytes(size) = self.next_event_override else {
            unreachable!()
        };
        let start_offset = self.get_offset();
        let bytes = err_to_event!(self, self.decoder.read_slice_from_payload(size));
        // Set it up so that we jump to the end of the child iteration
        self.stack.last_mut().unwrap().current_child_index = size;
        self.next_event_override = NextEventOverride::None;
        TraversalEvent::VisitTerminalValueSlice(VisitTerminalValueSlice {
            value_slice: TerminalValueSlice::U8(bytes),
            start_offset,
            end_offset: self.get_offset(),
            stack_depth: self.get_stack_depth(),
        })
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

        let mut traverser = basic_traverser(&payload).unwrap();

        // Start:
        next_event_is_owner_header(&mut traverser, OwnerValueHeader::Root, 0, 1, 1);
        next_event_is_owner_header(&mut traverser, OwnerValueHeader::Tuple(7), 1, 1, 3);
        // First line
        next_event_is_terminal_value(&mut traverser, TerminalValue::U8(2), 2, 3, 5);
        // Second line
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::U8, 2),
            2,
            5,
            8,
        );
        next_event_is_terminal_value_slice(
            &mut traverser,
            TerminalValueSlice::U8(&[3u8, 7u8]),
            3,
            8,
            10,
        );
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::U8, 2),
            2,
            5,
            10,
        );
        // Third line
        next_event_is_owner_header(&mut traverser, OwnerValueHeader::Tuple(2), 2, 10, 12);
        next_event_is_terminal_value(&mut traverser, TerminalValue::U32(3), 3, 12, 17);
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::Map(ValueKind::U8, ValueKind::U32, 1),
            3,
            17,
            21,
        );
        next_event_is_terminal_value(&mut traverser, TerminalValue::U8(16), 4, 21, 22);
        next_event_is_terminal_value(&mut traverser, TerminalValue::U32(18), 4, 22, 26);
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::Map(ValueKind::U8, ValueKind::U32, 1),
            3,
            17,
            26,
        );
        next_event_is_full_owner(&mut traverser, OwnerValueHeader::Tuple(2), 2, 10, 26);
        // Fourth line
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::EnumVariant(1, 1),
            2,
            26,
            29,
        );
        next_event_is_terminal_value(&mut traverser, TerminalValue::U32(4), 3, 29, 34);
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::EnumVariant(1, 1),
            2,
            26,
            34,
        );
        // Fifth line - empty Vec<u8> - no bytes event is output
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::U8, 0),
            2,
            34,
            37,
        );
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::U8, 0),
            2,
            34,
            37,
        );
        // Sixth line - empty Vec<i32>
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::I32, 0),
            2,
            37,
            40,
        );
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::I32, 0),
            2,
            37,
            40,
        );
        // Seventh line - Vec<Vec<(i64)>>
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::Array, 1),
            2,
            40,
            43,
        );
        next_event_is_owner_header(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::Tuple, 1),
            3,
            43,
            45,
        );
        next_event_is_owner_header(&mut traverser, OwnerValueHeader::Tuple(1), 4, 45, 46);
        next_event_is_terminal_value(&mut traverser, TerminalValue::I64(-2), 5, 46, 55);
        next_event_is_full_owner(&mut traverser, OwnerValueHeader::Tuple(1), 4, 45, 55);
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::Tuple, 1),
            3,
            43,
            55,
        );
        next_event_is_full_owner(
            &mut traverser,
            OwnerValueHeader::Array(ValueKind::Array, 1),
            2,
            40,
            55,
        );

        // End
        next_event_is_full_owner(&mut traverser, OwnerValueHeader::Tuple(7), 1, 1, 55);
        next_event_is_full_owner(&mut traverser, OwnerValueHeader::Root, 0, 1, 55);
    }

    pub fn next_event_is_owner_header(
        traverser: &mut BasicTraverser,
        expected_header: OwnerValueHeader<NoCustomValueKind>,
        expected_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let TraversalEvent::StartOwnerValue(VisitOwnerValueHeader { header, stack_depth, start_offset, end_offset }) = event else {
            panic!("Invalid event - expected VisitOwnerValueHeader, was {:?}", event);
        };
        assert_eq!(header, expected_header);
        assert_eq!(stack_depth, expected_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_full_owner(
        traverser: &mut BasicTraverser,
        expected_header: OwnerValueHeader<NoCustomValueKind>,
        expected_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let TraversalEvent::EndOwnerValue(VisitFullOwnerValue { header, stack_depth, start_offset, end_offset }) = event else {
            panic!("Invalid event - expected VisitFullOwnerValue, was {:?}", event);
        };
        assert_eq!(header, expected_header);
        assert_eq!(stack_depth, expected_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_terminal_value(
        traverser: &mut BasicTraverser,
        expected_value: TerminalValue,
        expected_stack_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let TraversalEvent::VisitTerminalValue(VisitTerminalValue { value, stack_depth, start_offset, end_offset }) = event else {
            panic!("Invalid event - expected VisitTerminalValue, was {:?}", event);
        };
        assert_eq!(value, expected_value);
        assert_eq!(stack_depth, expected_stack_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }

    pub fn next_event_is_terminal_value_slice(
        traverser: &mut BasicTraverser,
        expected_value_slice: TerminalValueSlice,
        expected_stack_depth: u8,
        expected_start_offset: usize,
        expected_end_offset: usize,
    ) {
        let event = traverser.next_event();
        let TraversalEvent::VisitTerminalValueSlice(VisitTerminalValueSlice { value_slice, stack_depth, start_offset, end_offset }) = event else {
            panic!("Invalid event - expected VisitTerminalValueSlice, was {:?}", event);
        };
        assert_eq!(value_slice, expected_value_slice);
        assert_eq!(stack_depth, expected_stack_depth);
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
    }
}

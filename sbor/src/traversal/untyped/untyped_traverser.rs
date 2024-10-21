use core::ops::ControlFlow;

use super::*;
use crate::decoder::BorrowingDecoder;
use crate::rust::prelude::*;
use crate::rust::str;
use crate::value_kind::*;
use crate::*;

/// Designed for streamed decoding of a payload or single encoded value (tree).
pub struct UntypedTraverser<'de, T: CustomTraversal> {
    decoder: VecDecoder<'de, T::CustomValueKind>,
    ancestor_path: Vec<AncestorState<T>>,
    config: UntypedTraverserConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AncestorState<T: CustomTraversal> {
    pub container_header: ContainerHeader<T>,
    /// The byte offset of the start of the container in the input buffer
    pub container_start_offset: usize,
    /// Goes from 0,... container_header.child_count() - 1 as children in the container are considered.
    /// NOTE: For maps, container_header.child_count() = 2 * Map length
    ///
    /// The `current_child_index` does NOT necessarily point at a valid value which can be decoded,
    /// - the index is updated before the child is read, to record errors against it.
    pub current_child_index: usize,
}

impl<T: CustomTraversal> AncestorState<T> {
    #[inline]
    pub fn get_implicit_value_kind_of_current_child(
        &self,
    ) -> Option<ValueKind<T::CustomValueKind>> {
        self.container_header
            .get_implicit_child_value_kind(self.current_child_index)
    }
}

pub struct UntypedTraverserConfig {
    pub max_depth: usize,
    pub check_exact_end: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum NextAction<T: CustomTraversal> {
    ReadPrefix {
        expected_prefix: u8,
    },
    ReadRootValue,
    ReadRootValueBody {
        implicit_value_kind: ValueKind<T::CustomValueKind>,
    },
    ReadContainerContentStart {
        container_header: ContainerHeader<T>,
        container_start_offset: usize,
    },
    /// The state which is put into after entering parent, and
    /// the default state to return to from below
    ReadNextChildOrExitContainer,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedStart<X: CustomValueKind> {
    PayloadPrefix(u8),
    Value,
    ValueBody(ValueKind<X>),
}

impl<X: CustomValueKind> ExpectedStart<X> {
    pub fn into_starting_action<T: CustomTraversal<CustomValueKind = X>>(self) -> NextAction<T> {
        match self {
            ExpectedStart::PayloadPrefix(prefix) => NextAction::ReadPrefix {
                expected_prefix: prefix,
            },
            ExpectedStart::Value => NextAction::ReadRootValue,
            ExpectedStart::ValueBody(value_kind) => NextAction::ReadRootValueBody {
                implicit_value_kind: value_kind,
            },
        }
    }
}

impl<'de, T: CustomTraversal> UntypedTraverser<'de, T> {
    pub fn new(input: &'de [u8], config: UntypedTraverserConfig) -> Self {
        Self {
            // Note that the VecTraverserV2 needs to be very low level for performance,
            // so purposefully doesn't use the depth tracking in the decoder itself.
            // But we set a max depth anyway, for safety.
            decoder: VecDecoder::new(input, config.max_depth),
            ancestor_path: Vec::with_capacity(config.max_depth),
            config,
        }
    }

    pub fn run_from_start<'t, V: UntypedPayloadVisitor<'de, T>>(
        &'t mut self,
        expected_start: ExpectedStart<T::CustomValueKind>,
        visitor: &mut V,
    ) -> V::Output<'t> {
        self.continue_traversal_from(expected_start.into_starting_action(), visitor)
    }

    /// # Expected behaviour
    /// Start action should either be an action from ExpectedStart, or a `resume_action` returned
    /// in a previous event.
    pub fn continue_traversal_from<'t, V: UntypedPayloadVisitor<'de, T>>(
        &'t mut self,
        start_action: NextAction<T>,
        visitor: &mut V,
    ) -> V::Output<'t> {
        let mut action = start_action;
        loop {
            // SAFETY: Work around the current borrow checker, which is sound as per this thread:
            // https://users.rust-lang.org/t/mutable-borrow-in-loop-borrow-checker-query/118081/3
            // Unsafe syntax borrowed from here: https://docs.rs/polonius-the-crab/latest/polonius_the_crab/
            // Can remove this once the polonius borrow checker hits stable
            let ancester_path = unsafe { &mut *(&mut self.ancestor_path as *mut _) };
            action = match Self::step(
                action,
                &self.config,
                &mut self.decoder,
                ancester_path,
                visitor,
            ) {
                ControlFlow::Continue(action) => action,
                ControlFlow::Break(output) => return output,
            };
        }
    }

    #[inline]
    fn step<'t, V: UntypedPayloadVisitor<'de, T>>(
        action: NextAction<T>,
        config: &UntypedTraverserConfig,
        decoder: &mut VecDecoder<'de, T::CustomValueKind>,
        ancestor_path: &'t mut Vec<AncestorState<T>>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        match action {
            NextAction::ReadPrefix { expected_prefix } => {
                Locator::with(decoder.get_offset(), ancestor_path, decoder)
                    .read_and_check_payload_prefix::<V>(expected_prefix, visitor)
            }
            NextAction::ReadRootValue => {
                Locator::with(decoder.get_offset(), ancestor_path, decoder)
                    .read_value(None, visitor)
            }
            NextAction::ReadRootValueBody {
                implicit_value_kind,
            } => Locator::with(decoder.get_offset(), ancestor_path, decoder)
                .read_value(Some(implicit_value_kind), visitor),
            NextAction::ReadContainerContentStart {
                container_header,
                container_start_offset,
            } => {
                let container_child_size = container_header.get_child_count();
                if container_child_size == 0 {
                    // If the container has no children, we immediately container end without ever bothering
                    // adding it as an ancestor.
                    Locator::with(container_start_offset, ancestor_path, decoder)
                        .complete_container_end(container_header, visitor)
                } else {
                    // Add ancestor before checking for max depth so that the ancestor stack is
                    // correct if the depth check returns an error
                    ancestor_path.push(AncestorState {
                        container_header,
                        container_start_offset,
                        current_child_index: 0,
                    });

                    // We know we're about to read a child at depth ancestor_path.len() + 1 - so
                    // it's an error if ancestor_path.len() >= config.max_depth.
                    // (We avoid the +1 so that we don't need to worry about overflow).
                    if ancestor_path.len() >= config.max_depth {
                        let error_output =
                            Locator::with(decoder.get_offset(), ancestor_path, decoder)
                                .handle_error(
                                    DecodeError::MaxDepthExceeded(config.max_depth),
                                    visitor,
                                );
                        return ControlFlow::Break(error_output);
                    }

                    let parent = ancestor_path.last_mut().unwrap();
                    let parent_container = &parent.container_header;
                    let is_byte_array = matches!(
                        parent_container,
                        ContainerHeader::Array(ArrayHeader {
                            element_value_kind: ValueKind::U8,
                            ..
                        })
                    );
                    // If it's a byte array, we do a batch-read optimisation
                    if is_byte_array {
                        // We know this is >= 1 from the above check
                        let array_length = container_child_size;
                        let max_index_which_would_be_read = array_length - 1;
                        // Set current child index before we read so that if we get an error on read
                        // then it comes through at the max child index we attempted to read.
                        parent.current_child_index = max_index_which_would_be_read;
                        Locator::with(decoder.get_offset(), ancestor_path, decoder)
                            .read_byte_array(array_length, visitor)
                    } else {
                        // NOTE: parent.current_child_index is already 0, so no need to change it
                        let implicit_value_kind = parent.get_implicit_value_kind_of_current_child();
                        Locator::with(decoder.get_offset(), ancestor_path, decoder)
                            .read_value(implicit_value_kind, visitor)
                    }
                }
            }
            NextAction::ReadNextChildOrExitContainer => {
                let parent = ancestor_path.last_mut();
                match parent {
                    Some(parent) => {
                        let next_child_index = parent.current_child_index + 1;
                        let is_complete =
                            next_child_index >= parent.container_header.get_child_count();
                        if is_complete {
                            // We pop the completed parent from the ancestor list
                            let AncestorState {
                                container_header,
                                container_start_offset,
                                ..
                            } = ancestor_path.pop().expect("Parent has just been read");

                            Locator::with(container_start_offset, ancestor_path, decoder)
                                .complete_container_end(container_header, visitor)
                        } else {
                            parent.current_child_index = next_child_index;
                            let implicit_value_kind =
                                parent.get_implicit_value_kind_of_current_child();
                            Locator::with(decoder.get_offset(), ancestor_path, decoder)
                                .read_value(implicit_value_kind, visitor)
                        }
                    }
                    None => {
                        // We are due to read another element and exit but have no parent
                        // This is because we have finished reading the `root` value.
                        let output = Locator::with(decoder.get_offset(), ancestor_path, decoder)
                            .handle_traversal_end(config.check_exact_end, visitor);
                        return ControlFlow::Break(output);
                    }
                }
            }
        }
    }
}

macro_rules! handle_result {
    ($self: expr, $visitor: expr, $result: expr$(,)?) => {{
        let result = $result;
        $self.handle_result(result, $visitor)?
    }};
}

/// This is just an encapsulation to improve code quality by:
/// * Removing code duplication by capturing the ancestor_path/decoder/start_offset in one place
/// * Ensuring code correctness by fixing the ancestor path
struct Locator<'t, 'd, 'de, T: CustomTraversal> {
    ancestor_path: &'t [AncestorState<T>],
    decoder: &'d mut VecDecoder<'de, T::CustomValueKind>,
    start_offset: usize,
}

impl<'t, 'd, 'de, T: CustomTraversal> Locator<'t, 'd, 'de, T> {
    #[inline]
    fn with(
        start_offset: usize,
        ancestor_path: &'t [AncestorState<T>],
        decoder: &'d mut VecDecoder<'de, T::CustomValueKind>,
    ) -> Self {
        Self {
            ancestor_path,
            decoder,
            start_offset,
        }
    }

    #[must_use]
    #[inline]
    fn read_and_check_payload_prefix<V: UntypedPayloadVisitor<'de, T>>(
        self,
        expected_prefix: u8,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        handle_result!(
            self,
            visitor,
            self.decoder.read_and_check_payload_prefix(expected_prefix)
        );
        ControlFlow::Continue(NextAction::ReadRootValue)
    }

    #[inline]
    #[must_use]
    fn read_value<V: UntypedPayloadVisitor<'de, T>>(
        self,
        implicit_value_kind: Option<ValueKind<T::CustomValueKind>>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let value_kind = match implicit_value_kind {
            Some(value_kind) => value_kind,
            None => handle_result!(self, visitor, self.decoder.read_value_kind()),
        };
        self.read_value_body(value_kind, visitor)
    }

    #[inline]
    #[must_use]
    fn read_byte_array<V: UntypedPayloadVisitor<'de, T>>(
        self,
        array_length: usize,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let bytes = handle_result!(
            self,
            visitor,
            self.decoder.read_slice_from_payload(array_length)
        );
        self.complete_terminal_value_batch(TerminalValueBatchRef::U8(bytes), visitor)
    }

    #[inline]
    #[must_use]
    fn read_value_body<V: UntypedPayloadVisitor<'de, T>>(
        self,
        value_kind: ValueKind<T::CustomValueKind>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        match value_kind {
            ValueKind::Bool => self.read_basic_value(value_kind, TerminalValueRef::Bool, visitor),
            ValueKind::I8 => self.read_basic_value(value_kind, TerminalValueRef::I8, visitor),
            ValueKind::I16 => self.read_basic_value(value_kind, TerminalValueRef::I16, visitor),
            ValueKind::I32 => self.read_basic_value(value_kind, TerminalValueRef::I32, visitor),
            ValueKind::I64 => self.read_basic_value(value_kind, TerminalValueRef::I64, visitor),
            ValueKind::I128 => self.read_basic_value(value_kind, TerminalValueRef::I128, visitor),
            ValueKind::U8 => self.read_basic_value(value_kind, TerminalValueRef::U8, visitor),
            ValueKind::U16 => self.read_basic_value(value_kind, TerminalValueRef::U16, visitor),
            ValueKind::U32 => self.read_basic_value(value_kind, TerminalValueRef::U32, visitor),
            ValueKind::U64 => self.read_basic_value(value_kind, TerminalValueRef::U64, visitor),
            ValueKind::U128 => self.read_basic_value(value_kind, TerminalValueRef::U128, visitor),
            ValueKind::String => {
                let length = handle_result!(self, visitor, self.decoder.read_size());
                let bytes =
                    handle_result!(self, visitor, self.decoder.read_slice_from_payload(length));
                let string_decode_result =
                    str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8);
                let string_body = handle_result!(self, visitor, string_decode_result);
                self.complete_terminal_value(TerminalValueRef::String(string_body), visitor)
            }
            ValueKind::Array => {
                let element_value_kind =
                    handle_result!(self, visitor, self.decoder.read_value_kind());
                let length = handle_result!(self, visitor, self.decoder.read_size());
                self.complete_container_start(
                    ContainerHeader::Array(ArrayHeader {
                        element_value_kind,
                        length,
                    }),
                    visitor,
                )
            }
            ValueKind::Map => {
                let key_value_kind = handle_result!(self, visitor, self.decoder.read_value_kind());
                let value_value_kind =
                    handle_result!(self, visitor, self.decoder.read_value_kind());
                let length = handle_result!(self, visitor, self.decoder.read_size());
                self.complete_container_start(
                    ContainerHeader::Map(MapHeader {
                        key_value_kind,
                        value_value_kind,
                        length,
                    }),
                    visitor,
                )
            }
            ValueKind::Enum => {
                let variant = handle_result!(self, visitor, self.decoder.read_byte());
                let length = handle_result!(self, visitor, self.decoder.read_size());
                self.complete_container_start(
                    ContainerHeader::EnumVariant(EnumVariantHeader { variant, length }),
                    visitor,
                )
            }
            ValueKind::Tuple => {
                let length = handle_result!(self, visitor, self.decoder.read_size());
                self.complete_container_start(
                    ContainerHeader::Tuple(TupleHeader { length }),
                    visitor,
                )
            }
            ValueKind::Custom(custom_value_kind) => {
                let custom_value_ref = handle_result!(
                    self,
                    visitor,
                    T::read_custom_value_body(custom_value_kind, self.decoder)
                );
                self.complete_terminal_value(TerminalValueRef::Custom(custom_value_ref), visitor)
            }
        }
    }

    #[inline]
    #[must_use]
    fn read_basic_value<
        X: Decode<T::CustomValueKind, VecDecoder<'de, T::CustomValueKind>>,
        V: UntypedPayloadVisitor<'de, T>,
    >(
        self,
        value_kind: ValueKind<T::CustomValueKind>,
        value_ref_constructor: impl Fn(X) -> TerminalValueRef<'de, T>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let value = handle_result!(
            self,
            visitor,
            X::decode_body_with_value_kind(self.decoder, value_kind)
        );
        self.complete_terminal_value(value_ref_constructor(value), visitor)
    }

    #[inline]
    #[must_use]
    fn complete_terminal_value<V: UntypedPayloadVisitor<'de, T>>(
        self,
        value_ref: TerminalValueRef<'de, T>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let next_action = NextAction::ReadNextChildOrExitContainer;
        visitor.on_terminal_value(OnTerminalValue {
            location: self.location(),
            value: value_ref,
            resume_action: next_action,
        })?;
        ControlFlow::Continue(next_action)
    }

    #[inline]
    #[must_use]
    fn complete_terminal_value_batch<V: UntypedPayloadVisitor<'de, T>>(
        self,
        value_batch_ref: TerminalValueBatchRef<'de>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let next_action = NextAction::ReadNextChildOrExitContainer;
        visitor.on_terminal_value_batch(OnTerminalValueBatch {
            location: self.location(),
            value_batch: value_batch_ref,
            resume_action: next_action,
        })?;
        ControlFlow::Continue(next_action)
    }

    #[inline]
    #[must_use]
    fn complete_container_start<V: UntypedPayloadVisitor<'de, T>>(
        self,
        container_header: ContainerHeader<T>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let next_action = NextAction::ReadContainerContentStart {
            container_header: container_header.clone(),
            container_start_offset: self.start_offset,
        };
        visitor.on_container_start(OnContainerStart {
            location: self.location(),
            header: container_header,
            resume_action: next_action,
        })?;
        ControlFlow::Continue(next_action)
    }

    #[inline]
    #[must_use]
    fn complete_container_end<V: UntypedPayloadVisitor<'de, T>>(
        self,
        container_header: ContainerHeader<T>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, NextAction<T>> {
        let next_action = NextAction::ReadNextChildOrExitContainer;
        visitor.on_container_end(OnContainerEnd {
            location: self.location(),
            header: container_header,
            resume_action: next_action,
        })?;
        ControlFlow::Continue(next_action)
    }

    #[inline]
    #[must_use]
    fn handle_result<V: UntypedPayloadVisitor<'de, T>, X>(
        &self,
        result: Result<X, DecodeError>,
        visitor: &mut V,
    ) -> ControlFlow<V::Output<'t>, X> {
        match result {
            Ok(value) => ControlFlow::Continue(value),
            Err(error) => ControlFlow::Break(self.handle_error(error, visitor)),
        }
    }

    #[inline]
    #[must_use]
    fn handle_traversal_end<V: UntypedPayloadVisitor<'de, T>>(
        self,
        check_end: bool,
        visitor: &mut V,
    ) -> V::Output<'t> {
        if check_end {
            if let Err(error) = self.decoder.check_end() {
                return self.handle_error(error, visitor);
            }
        }
        visitor.on_traversal_end(OnTraversalEnd {
            location: self.location(),
        })
    }

    #[inline]
    #[must_use]
    fn handle_error<V: UntypedPayloadVisitor<'de, T>>(
        &self,
        error: DecodeError,
        visitor: &mut V,
    ) -> V::Output<'t> {
        visitor.on_error(OnError {
            error,
            location: self.location(),
        })
    }

    #[inline]
    fn location(&self) -> Location<'t, T> {
        Location {
            start_offset: self.start_offset,
            end_offset: self.decoder.get_offset(),
            ancestor_path: self.ancestor_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::prelude::*;

    use super::*;

    #[derive(Categorize, Encode)]
    #[allow(dead_code)]
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
    pub fn test_calculate_value_tree_body_byte_array() {
        let payload = basic_encode(&BasicValue::Array {
            element_value_kind: BasicValueKind::Array,
            elements: vec![BasicValue::Array {
                element_value_kind: BasicValueKind::U8,
                elements: vec![BasicValue::U8 { value: 44 }, BasicValue::U8 { value: 55 }],
            }],
        })
        .unwrap();
        /*
            91  - prefix
            32  - value kind: array
            32  - element value kind: array
            1   - number of elements: 1
            7   - element value kind: u8
            2   - number of elements: u8
            44  - u8
            55  - u8
        */
        let length = calculate_value_tree_body_byte_length::<NoCustomExtension>(
            &payload[2..],
            BasicValueKind::Array,
            0,
            100,
        )
        .unwrap();
        assert_eq!(length, 6);
        let length = calculate_value_tree_body_byte_length::<NoCustomExtension>(
            &payload[6..],
            BasicValueKind::U8,
            0,
            100,
        )
        .unwrap();
        assert_eq!(length, 1);
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
            location:
                Location {
                    start_offset,
                    end_offset,
                    ..
                },
        } = event
        else {
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
            location:
                Location {
                    start_offset,
                    end_offset,
                    ..
                },
        } = event
        else {
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
            location:
                Location {
                    start_offset,
                    end_offset,
                    ..
                },
        } = event
        else {
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
            location:
                Location {
                    start_offset,
                    end_offset,
                    ..
                },
        } = event
        else {
            panic!(
                "Invalid event - expected TerminalValueBatch, was {:?}",
                event
            );
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
            location:
                Location {
                    start_offset,
                    end_offset,
                    ..
                },
        } = event
        else {
            panic!("Invalid event - expected End, was {:?}", event);
        };
        assert_eq!(start_offset, expected_start_offset);
        assert_eq!(end_offset, expected_end_offset);
        assert!(event.location.ancestor_path.is_empty());
    }
}

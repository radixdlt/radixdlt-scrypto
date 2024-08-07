use super::*;
use crate::decoder::BorrowingDecoder;
use crate::rust::prelude::*;
use crate::rust::str;
use crate::value_kind::*;
use crate::*;

/// Returns the length of the value at the start of the partial payload.
pub fn calculate_value_tree_body_byte_length<'de, 's, E: CustomExtension>(
    partial_payload: &'de [u8],
    value_kind: ValueKind<E::CustomValueKind>,
    current_depth: usize,
    depth_limit: usize,
) -> Result<usize, DecodeError> {
    let mut traverser = VecTraverser::<E::CustomTraversal>::new(
        partial_payload,
        ExpectedStart::ValueBody(value_kind),
        VecTraverserConfig {
            max_depth: depth_limit - current_depth,
            check_exact_end: false,
        },
    );
    loop {
        let next_event = traverser.next_event();
        match next_event.event {
            TraversalEvent::End => return Ok(next_event.location.end_offset),
            TraversalEvent::DecodeError(decode_error) => return Err(decode_error),
            _ => {}
        }
    }
}

pub trait CustomTraversal: Copy + Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;
    type CustomTerminalValueRef<'de>: CustomTerminalValueRef<
        CustomValueKind = Self::CustomValueKind,
    >;

    fn read_custom_value_body<'de, R>(
        custom_value_kind: Self::CustomValueKind,
        reader: &mut R,
    ) -> Result<Self::CustomTerminalValueRef<'de>, DecodeError>
    where
        R: BorrowingDecoder<'de, Self::CustomValueKind>;
}

pub trait CustomTerminalValueRef: Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind;
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
    fn get_implicit_value_kind_of_current_child(&self) -> Option<ValueKind<T::CustomValueKind>> {
        self.container_header
            .get_implicit_child_value_kind(self.current_child_index)
    }
}

/// The `VecTraverser` is for streamed decoding of a payload or single encoded value (tree).
/// It turns payload decoding into a pull-based event stream.
///
/// The caller is responsible for stopping calling `next_event` after an Error or End event.
pub struct VecTraverser<'de, T: CustomTraversal> {
    decoder: VecDecoder<'de, T::CustomValueKind>,
    ancestor_path: Vec<AncestorState<T>>,
    next_action: NextAction<T>,
    config: VecTraverserConfig,
}

pub struct VecTraverserConfig {
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
    Errored,
    Ended,
    /// Impossible to observe this value
    InProgressPlaceholder,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedStart<X: CustomValueKind> {
    PayloadPrefix(u8),
    Value,
    ValueBody(ValueKind<X>),
}

impl<'de, T: CustomTraversal> VecTraverser<'de, T> {
    pub fn new(
        input: &'de [u8],
        expected_start: ExpectedStart<T::CustomValueKind>,
        config: VecTraverserConfig,
    ) -> Self {
        Self {
            // Note that the VecTraverser needs to be very low level for performance,
            // so purposefully doesn't use the depth tracking in the decoder itself.
            // But we set a max depth anyway, for safety.
            decoder: VecDecoder::new(input, config.max_depth),
            ancestor_path: Vec::with_capacity(config.max_depth),
            next_action: match expected_start {
                ExpectedStart::PayloadPrefix(prefix) => NextAction::ReadPrefix {
                    expected_prefix: prefix,
                },
                ExpectedStart::Value => NextAction::ReadRootValue,
                ExpectedStart::ValueBody(value_kind) => NextAction::ReadRootValueBody {
                    implicit_value_kind: value_kind,
                },
            },
            config,
        }
    }

    pub fn next_event<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        let (event, next_action) = Self::step(
            core::mem::replace(&mut self.next_action, NextAction::InProgressPlaceholder),
            &self.config,
            &mut self.decoder,
            &mut self.ancestor_path,
        );
        self.next_action = next_action;
        event
    }

    #[inline]
    fn step<'t, 'd>(
        action: NextAction<T>,
        config: &VecTraverserConfig,
        decoder: &'d mut VecDecoder<'de, T::CustomValueKind>,
        ancestor_path: &'t mut Vec<AncestorState<T>>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        match action {
            NextAction::ReadPrefix { expected_prefix } => {
                // The reading of the prefix has no associated event, so we perform the prefix check first,
                // and then proceed to read the root value if it succeeds.
                let start_offset = decoder.get_offset();
                match decoder.read_and_check_payload_prefix(expected_prefix) {
                    Ok(()) => {
                        // Prefix read successfully. Now read root value.
                        ActionHandler::new_from_current_offset(ancestor_path, decoder)
                            .read_value(None)
                    }
                    Err(error) => {
                        ActionHandler::new_with_fixed_offset(ancestor_path, decoder, start_offset)
                            .complete_with_error(error)
                    }
                }
            }
            NextAction::ReadRootValue => {
                ActionHandler::new_from_current_offset(ancestor_path, decoder).read_value(None)
            }
            NextAction::ReadRootValueBody {
                implicit_value_kind,
            } => ActionHandler::new_from_current_offset(ancestor_path, decoder)
                .read_value(Some(implicit_value_kind)),
            NextAction::ReadContainerContentStart {
                container_header,
                container_start_offset,
            } => {
                let container_child_size = container_header.get_child_count();
                if container_child_size == 0 {
                    // If the container has no children, we immediately container end without ever bothering
                    // adding it as an ancestor.
                    return ActionHandler::new_with_fixed_offset(
                        ancestor_path,
                        decoder,
                        container_start_offset,
                    )
                    .complete_container_end(container_header);
                }

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
                    return ActionHandler::new_from_current_offset(ancestor_path, decoder)
                        .complete_with_error(DecodeError::MaxDepthExceeded(config.max_depth));
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
                    ActionHandler::new_from_current_offset(ancestor_path, decoder)
                        .read_byte_array(array_length)
                } else {
                    // NOTE: parent.current_child_index is already 0, so no need to change it
                    let implicit_value_kind = parent.get_implicit_value_kind_of_current_child();
                    ActionHandler::new_from_current_offset(ancestor_path, decoder)
                        .read_value(implicit_value_kind)
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

                            ActionHandler::new_with_fixed_offset(
                                ancestor_path,
                                decoder,
                                container_start_offset,
                            )
                            .complete_container_end(container_header)
                        } else {
                            parent.current_child_index = next_child_index;
                            let implicit_value_kind =
                                parent.get_implicit_value_kind_of_current_child();
                            ActionHandler::new_from_current_offset(ancestor_path, decoder)
                                .read_value(implicit_value_kind)
                        }
                    }
                    None => {
                        // We are due to read another element and exit but have no parent
                        // This is because we have finished reading the `root` value.
                        // Therefore we call `end`.
                        ActionHandler::new_from_current_offset(ancestor_path, decoder).end(config)
                    }
                }
            }
            NextAction::Errored => {
                panic!("It is unsupported to call `next_event` on a traverser which has returned an error.")
            }
            NextAction::Ended => {
                panic!("It is unsupported to call `next_event` on a traverser which has already emitted an end event.")
            }
            NextAction::InProgressPlaceholder => {
                unreachable!("It is not possible to observe this value - it is a placeholder for rust memory safety.")
            }
        }
    }
}

macro_rules! handle_error {
    ($action_handler: expr, $result: expr$(,)?) => {{
        match $result {
            Ok(value) => value,
            Err(error) => {
                return $action_handler.complete_with_error(error);
            }
        }
    }};
}

/// This is just an encapsulation to improve code quality by:
/// * Removing code duplication by capturing the ancestor_path/decoder/start_offset in one place
/// * Ensuring code correctness by fixing the ancestor path
struct ActionHandler<'t, 'd, 'de, T: CustomTraversal> {
    ancestor_path: &'t [AncestorState<T>],
    decoder: &'d mut VecDecoder<'de, T::CustomValueKind>,
    start_offset: usize,
}

impl<'t, 'd, 'de, T: CustomTraversal> ActionHandler<'t, 'd, 'de, T> {
    #[inline]
    fn new_from_current_offset(
        ancestor_path: &'t [AncestorState<T>],
        decoder: &'d mut VecDecoder<'de, T::CustomValueKind>,
    ) -> Self {
        let start_offset = decoder.get_offset();
        Self {
            ancestor_path,
            decoder,
            start_offset,
        }
    }

    #[inline]
    fn new_with_fixed_offset(
        ancestor_path: &'t [AncestorState<T>],
        decoder: &'d mut VecDecoder<'de, T::CustomValueKind>,
        start_offset: usize,
    ) -> Self {
        Self {
            ancestor_path,
            decoder,
            start_offset,
        }
    }

    #[inline]
    fn read_value(
        self,
        implicit_value_kind: Option<ValueKind<T::CustomValueKind>>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        let value_kind = match implicit_value_kind {
            Some(value_kind) => value_kind,
            None => handle_error!(self, self.decoder.read_value_kind()),
        };
        self.read_value_body(value_kind)
    }

    #[inline]
    fn read_byte_array(
        self,
        array_length: usize,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        let bytes = handle_error!(self, self.decoder.read_slice_from_payload(array_length));
        self.complete(
            TraversalEvent::TerminalValueBatch(TerminalValueBatchRef::U8(bytes)),
            // This is the correct action to ensure we exit the container on the next step
            NextAction::ReadNextChildOrExitContainer,
        )
    }

    #[inline]
    fn end(
        self,
        config: &VecTraverserConfig,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        if config.check_exact_end {
            handle_error!(self, self.decoder.check_end());
        }
        self.complete(TraversalEvent::End, NextAction::Ended)
    }

    #[inline]
    fn read_value_body(
        self,
        value_kind: ValueKind<T::CustomValueKind>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        match value_kind {
            ValueKind::Bool => self.read_terminal_value(value_kind, TerminalValueRef::Bool),
            ValueKind::I8 => self.read_terminal_value(value_kind, TerminalValueRef::I8),
            ValueKind::I16 => self.read_terminal_value(value_kind, TerminalValueRef::I16),
            ValueKind::I32 => self.read_terminal_value(value_kind, TerminalValueRef::I32),
            ValueKind::I64 => self.read_terminal_value(value_kind, TerminalValueRef::I64),
            ValueKind::I128 => self.read_terminal_value(value_kind, TerminalValueRef::I128),
            ValueKind::U8 => self.read_terminal_value(value_kind, TerminalValueRef::U8),
            ValueKind::U16 => self.read_terminal_value(value_kind, TerminalValueRef::U16),
            ValueKind::U32 => self.read_terminal_value(value_kind, TerminalValueRef::U32),
            ValueKind::U64 => self.read_terminal_value(value_kind, TerminalValueRef::U64),
            ValueKind::U128 => self.read_terminal_value(value_kind, TerminalValueRef::U128),
            ValueKind::String => {
                let length = handle_error!(self, self.decoder.read_size());
                let bytes = handle_error!(self, self.decoder.read_slice_from_payload(length));
                let string_body = handle_error!(
                    self,
                    str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
                );
                self.complete(
                    TraversalEvent::TerminalValue(TerminalValueRef::String(string_body)),
                    NextAction::ReadNextChildOrExitContainer,
                )
            }
            ValueKind::Array => {
                let element_value_kind = handle_error!(self, self.decoder.read_value_kind());
                let length = handle_error!(self, self.decoder.read_size());
                self.complete_container_start(ContainerHeader::Array(ArrayHeader {
                    element_value_kind,
                    length,
                }))
            }
            ValueKind::Map => {
                let key_value_kind = handle_error!(self, self.decoder.read_value_kind());
                let value_value_kind = handle_error!(self, self.decoder.read_value_kind());
                let length = handle_error!(self, self.decoder.read_size());
                self.complete_container_start(ContainerHeader::Map(MapHeader {
                    key_value_kind,
                    value_value_kind,
                    length,
                }))
            }
            ValueKind::Enum => {
                let variant = handle_error!(self, self.decoder.read_byte());
                let length = handle_error!(self, self.decoder.read_size());
                self.complete_container_start(ContainerHeader::EnumVariant(EnumVariantHeader {
                    variant,
                    length,
                }))
            }
            ValueKind::Tuple => {
                let length = handle_error!(self, self.decoder.read_size());
                self.complete_container_start(ContainerHeader::Tuple(TupleHeader { length }))
            }
            ValueKind::Custom(custom_value_kind) => {
                let custom_value_ref = handle_error!(
                    self,
                    T::read_custom_value_body(custom_value_kind, self.decoder)
                );
                self.complete(
                    TraversalEvent::TerminalValue(TerminalValueRef::Custom(custom_value_ref)),
                    NextAction::ReadNextChildOrExitContainer,
                )
            }
        }
    }

    #[inline]
    fn read_terminal_value<V: Decode<T::CustomValueKind, VecDecoder<'de, T::CustomValueKind>>>(
        self,
        value_kind: ValueKind<T::CustomValueKind>,
        value_ref_constructor: impl Fn(V) -> TerminalValueRef<'de, T>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        match V::decode_body_with_value_kind(self.decoder, value_kind) {
            Ok(value) => self.complete(
                TraversalEvent::TerminalValue(value_ref_constructor(value)),
                NextAction::ReadNextChildOrExitContainer,
            ),
            Err(error) => self.complete_with_error(error),
        }
    }

    #[inline]
    fn complete_container_start(
        self,
        container_header: ContainerHeader<T>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        let next_action = NextAction::ReadContainerContentStart {
            container_header: container_header.clone(),
            container_start_offset: self.start_offset,
        };
        self.complete(
            TraversalEvent::ContainerStart(container_header),
            next_action,
        )
    }

    #[inline]
    fn complete_container_end(
        self,
        container_header: ContainerHeader<T>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        self.complete(
            TraversalEvent::ContainerEnd(container_header),
            // Continue interating the parent
            NextAction::ReadNextChildOrExitContainer,
        )
    }

    #[inline]
    fn complete_with_error(
        self,
        error: DecodeError,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        self.complete(TraversalEvent::DecodeError(error), NextAction::Errored)
    }

    #[inline]
    fn complete(
        self,
        traversal_event: TraversalEvent<'de, T>,
        next_action: NextAction<T>,
    ) -> (LocatedTraversalEvent<'t, 'de, T>, NextAction<T>) {
        let located_event = LocatedTraversalEvent {
            event: traversal_event,
            location: Location {
                start_offset: self.start_offset,
                end_offset: self.decoder.get_offset(),
                ancestor_path: self.ancestor_path,
            },
        };
        (located_event, next_action)
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

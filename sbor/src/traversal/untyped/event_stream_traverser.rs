use crate::internal_prelude::*;

// =================
// DEPRECATION NOTES
// =================
// Once we no longer need this (because we've moved to the visitor model), this opens up
// a world of further optimisations:
// * We can change it so that issuing a ControlFlow::Break aborts the process
// * Can get rid of `NextAction` and Step completely
//   * And instead, just have standard top-down traversal logic
//   * We can avoid storing a `resume_action` on the events

/// The `VecTraverser` is for streamed decoding of a payload or single encoded value (tree).
/// It turns payload decoding into a pull-based event stream.
///
/// The caller is responsible for stopping calling `next_event` after an Error or End event.
#[deprecated = "Use UntypedTraverser which uses the visitor pattern and is more efficient"]
pub struct VecTraverser<'de, T: CustomTraversal> {
    untyped_traverser: UntypedTraverser<'de, T>,
    visitor: EventStreamVisitor<'de, T>,
}

pub struct VecTraverserConfig {
    pub max_depth: usize,
    pub check_exact_end: bool,
}

#[allow(deprecated)]
impl<'de, T: CustomTraversal> VecTraverser<'de, T> {
    pub fn new(
        input: &'de [u8],
        expected_start: ExpectedStart<T::CustomValueKind>,
        config: VecTraverserConfig,
    ) -> Self {
        let config = UntypedTraverserConfig {
            max_depth: config.max_depth,
            check_exact_end: config.check_exact_end,
        };
        let untyped_traverser = UntypedTraverser::<T>::new(input, config);
        Self {
            untyped_traverser,
            visitor: EventStreamVisitor {
                next_action: SuspendableNextAction::Action(expected_start.into_starting_action()),
                next_event: None,
            },
        }
    }

    pub fn next_event<'t>(&'t mut self) -> LocatedTraversalEvent<'t, 'de, T> {
        match self.visitor.next_action {
            SuspendableNextAction::Action(next_action) => {
                let location = self
                    .untyped_traverser
                    .continue_traversal_from(next_action, &mut self.visitor);
                LocatedTraversalEvent {
                    location,
                    event: self
                        .visitor
                        .next_event
                        .take()
                        .expect("Visitor always expected to populate an event"),
                }
            }
            SuspendableNextAction::Errored => panic!("Can't get next event as already errored"),
            SuspendableNextAction::Ended => todo!("Can't get next event as already ended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;

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

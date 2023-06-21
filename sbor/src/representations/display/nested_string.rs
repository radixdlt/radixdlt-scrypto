use super::*;
use crate::representations::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;
use TypedTraversalEvent::*;

// TODO - This file could do with a minor refactor to commonize logic to print fields

#[derive(Debug, Clone, Copy)]
pub struct NestedStringDisplayContext<'s, 'a, E: FormattableCustomExtension> {
    pub schema: &'s Schema<E::CustomSchema>,
    pub custom_context: E::CustomDisplayContext<'a>,
    pub print_mode: PrintMode,
}

impl From<fmt::Error> for FormattingError {
    fn from(e: fmt::Error) -> Self {
        FormattingError::Fmt(e)
    }
}

pub fn format_payload_as_nested_string<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    context: &NestedStringDisplayContext<'_, '_, E>,
    payload: &'_ [u8],
    type_index: LocalTypeIndex,
) -> Result<(), FormattingError> {
    let mut traverser = traverse_payload_with_types(payload, context.schema, type_index);
    if let PrintMode::MultiLine {
        first_line_indent, ..
    } = &context.print_mode
    {
        write!(f, "{:first_line_indent$}", "")?;
    }
    format_value_tree(f, &mut traverser, context)?;
    consume_end_event(&mut traverser)?;
    Ok(())
}

pub(crate) fn format_partial_payload_as_nested_string<
    F: fmt::Write,
    E: FormattableCustomExtension,
>(
    f: &mut F,
    partial_payload: &[u8],
    expected_start: ExpectedStart<E::CustomValueKind>,
    check_exact_end: bool,
    current_depth: usize,
    context: &NestedStringDisplayContext<'_, '_, E>,
    type_index: LocalTypeIndex,
) -> Result<(), FormattingError> {
    let mut traverser = traverse_partial_payload_with_types(
        partial_payload,
        expected_start,
        check_exact_end,
        current_depth,
        context.schema,
        type_index,
    );
    if let PrintMode::MultiLine {
        first_line_indent, ..
    } = &context.print_mode
    {
        write!(f, "{:first_line_indent$}", "")?;
    }
    format_value_tree(f, &mut traverser, context)?;
    if check_exact_end {
        consume_end_event(&mut traverser)?;
    }
    Ok(())
}

fn consume_end_event<E: FormattableCustomExtension>(
    traverser: &mut TypedTraverser<E>,
) -> Result<(), FormattingError> {
    traverser.consume_end_event().map_err(FormattingError::Sbor)
}

fn consume_container_end<E: FormattableCustomExtension>(
    traverser: &mut TypedTraverser<E>,
) -> Result<(), FormattingError> {
    traverser
        .consume_container_end_event()
        .map_err(FormattingError::Sbor)
}

fn format_value_tree<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    traverser: &mut TypedTraverser<E>,
    context: &NestedStringDisplayContext<'_, '_, E>,
) -> Result<(), FormattingError> {
    let typed_event = traverser.next_event();
    match typed_event.event {
        ContainerStart(type_index, container_header) => {
            let parent_depth = typed_event.location.typed_ancestor_path.len();
            match container_header {
                ContainerHeader::Tuple(header) => {
                    format_tuple(f, traverser, context, type_index, header, parent_depth)
                }
                ContainerHeader::EnumVariant(header) => {
                    format_enum_variant(f, traverser, context, type_index, header, parent_depth)
                }
                ContainerHeader::Array(header) => {
                    format_array(f, traverser, context, type_index, header, parent_depth)
                }
                ContainerHeader::Map(header) => {
                    format_map(f, traverser, context, type_index, header, parent_depth)
                }
            }
        }
        TerminalValue(type_index, value_ref) => {
            format_terminal_value(f, context, type_index, value_ref)
        }
        _ => Err(FormattingError::Sbor(
            typed_event
                .display_as_unexpected_event("ContainerStart | TerminalValue", &context.schema),
        )),
    }
}

fn format_tuple<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    traverser: &mut TypedTraverser<E>,
    context: &NestedStringDisplayContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    tuple_header: TupleHeader,
    parent_depth: usize,
) -> Result<(), FormattingError> {
    let tuple_data = context
        .schema
        .resolve_matching_tuple_metadata(type_index, tuple_header.length);

    if let Some(type_name) = tuple_data.name {
        write!(f, "Tuple:{}(", type_name)?;
    } else {
        write!(f, "Tuple(")?;
    }

    let child_count = tuple_header.length;

    match (child_count, context.print_mode) {
        (0, _) => {
            write!(f, ")")?;
        }
        (_, PrintMode::SingleLine) => {
            match tuple_data.field_names {
                Some(field_names) => {
                    for i in 0..tuple_header.length {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{} = ", field_names.get(i).unwrap())?;
                        format_value_tree(f, traverser, context)?;
                    }
                }
                _ => {
                    for i in 0..tuple_header.length {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        format_value_tree(f, traverser, context)?;
                    }
                }
            }
            write!(f, ")")?;
        }
        (
            _,
            PrintMode::MultiLine {
                indent_size: spaces_per_indent,
                base_indent,
                ..
            },
        ) => {
            let child_indent_size = base_indent + spaces_per_indent * parent_depth;
            let child_indent = " ".repeat(child_indent_size);
            let parent_indent = &child_indent[0..child_indent_size - spaces_per_indent];
            write!(f, "\n")?;
            match tuple_data.field_names {
                Some(field_names) => {
                    for i in 0..tuple_header.length {
                        write!(f, "{}{} = ", child_indent, field_names.get(i).unwrap())?;
                        format_value_tree(f, traverser, context)?;
                        write!(f, ",\n")?;
                    }
                }
                _ => {
                    for _ in 0..tuple_header.length {
                        write!(f, "{}", child_indent)?;
                        format_value_tree(f, traverser, context)?;
                        write!(f, ",\n")?;
                    }
                }
            }

            write!(f, "{})", parent_indent)?;
        }
    }

    consume_container_end(traverser)?;
    Ok(())
}

fn format_enum_variant<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    traverser: &mut TypedTraverser<E>,
    context: &NestedStringDisplayContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    variant_header: EnumVariantHeader,
    parent_depth: usize,
) -> Result<(), FormattingError> {
    let enum_data = context.schema.resolve_matching_enum_metadata(
        type_index,
        variant_header.variant,
        variant_header.length,
    );

    if let Some(type_name) = enum_data.enum_name {
        write!(f, "Enum:{}(", type_name)?;
    } else {
        write!(f, "Enum(")?;
    }

    let field_length = variant_header.length;

    match (field_length, context.print_mode) {
        (0, _) | (_, PrintMode::SingleLine) => {
            if let Some(variant_name) = enum_data.variant_name {
                write!(f, "{}u8:{}", variant_header.variant, variant_name)?;
            } else {
                write!(f, "{}u8", variant_header.variant)?;
            }
            match enum_data.field_names {
                Some(field_names) => {
                    for i in 0..field_length {
                        write!(f, ", {} = ", field_names.get(i).unwrap())?;
                        format_value_tree(f, traverser, context)?;
                    }
                }
                _ => {
                    for _ in 0..field_length {
                        write!(f, ", ")?;
                        format_value_tree(f, traverser, context)?;
                    }
                }
            }
            write!(f, ")")?;
        }
        (
            _,
            PrintMode::MultiLine {
                indent_size: spaces_per_indent,
                base_indent,
                ..
            },
        ) => {
            let child_indent_size = base_indent + spaces_per_indent * parent_depth;
            let child_indent = " ".repeat(child_indent_size);
            let parent_indent = &child_indent[0..child_indent_size - spaces_per_indent];
            write!(f, "\n")?;
            match enum_data.field_names {
                Some(field_names) => {
                    for i in 0..field_length {
                        write!(f, "{}{} = ", child_indent, field_names.get(i).unwrap())?;
                        format_value_tree(f, traverser, context)?;
                        write!(f, ",\n")?;
                    }
                }
                _ => {
                    for _ in 0..field_length {
                        write!(f, "{}", child_indent)?;
                        format_value_tree(f, traverser, context)?;
                        write!(f, ",\n")?;
                    }
                }
            }

            write!(f, "{})", parent_indent)?;
        }
    }

    consume_container_end(traverser)?;
    Ok(())
}

fn format_array<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    traverser: &mut TypedTraverser<E>,
    context: &NestedStringDisplayContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    array_header: ArrayHeader<E::CustomValueKind>,
    parent_depth: usize,
) -> Result<(), FormattingError> {
    let array_data = context.schema.resolve_matching_array_metadata(type_index);

    // Note - this is (purposefully) subtly different to the implementation in the manifest:
    // We wrap bytes as Array<U8>(Hex("deadbeef")) instead of Bytes("deadbeef") to fit
    // better with the type annotations.

    match array_data.array_name {
        Some(array_name) => {
            write!(f, "Array:{}<", array_name)?;
        }
        None => {
            write!(f, "Array<")?;
        }
    }

    match array_data.element_name {
        Some(element_name) => {
            write!(f, "{}:{}>(", array_header.element_value_kind, element_name)?;
        }
        None => {
            write!(f, "{}>(", array_header.element_value_kind)?;
        }
    }

    let child_count = array_header.length;

    match (
        child_count,
        context.print_mode,
        array_header.element_value_kind,
    ) {
        (0, _, _) => {
            write!(f, ")")?;
        }
        (_, _, ValueKind::U8) => {
            write!(f, "Hex(\"")?;
            let typed_event = traverser.next_event();
            match typed_event.event {
                TerminalValueBatch(_, TerminalValueBatchRef::U8(bytes)) => {
                    f.write_str(&hex::encode(bytes))?;
                }
                _ => Err(FormattingError::Sbor(
                    typed_event.display_as_unexpected_event("TerminalValueBatch", &context.schema),
                ))?,
            };
            write!(f, "\"))")?;
        }
        (child_count, PrintMode::SingleLine, _) => {
            for i in 0..child_count {
                if i > 0 {
                    write!(f, ", ")?;
                }
                format_value_tree(f, traverser, context)?;
            }
            write!(f, ")")?;
        }
        (
            child_count,
            PrintMode::MultiLine {
                indent_size: spaces_per_indent,
                base_indent,
                ..
            },
            _,
        ) => {
            let child_indent_size = base_indent + spaces_per_indent * parent_depth;
            let child_indent = " ".repeat(child_indent_size);
            let parent_indent = &child_indent[0..child_indent_size - spaces_per_indent];
            write!(f, "\n")?;
            for _ in 0..child_count {
                write!(f, "{}", child_indent)?;
                format_value_tree(f, traverser, context)?;
                write!(f, ",\n")?;
            }

            write!(f, "{})", parent_indent)?;
        }
    }

    consume_container_end(traverser)?;
    Ok(())
}

fn format_map<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    traverser: &mut TypedTraverser<E>,
    context: &NestedStringDisplayContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    map_header: MapHeader<E::CustomValueKind>,
    parent_depth: usize,
) -> Result<(), FormattingError> {
    let map_data = context.schema.resolve_matching_map_metadata(type_index);

    match map_data.map_name {
        Some(array_name) => {
            write!(f, "Map:{}<", array_name)?;
        }
        None => {
            write!(f, "Map<")?;
        }
    }

    match map_data.key_name {
        Some(key_name) => {
            write!(f, "{}:{}, ", map_header.key_value_kind, key_name)?;
        }
        None => {
            write!(f, "{}, ", map_header.key_value_kind)?;
        }
    }

    match map_data.value_name {
        Some(value_name) => {
            write!(f, "{}:{}>(", map_header.value_value_kind, value_name)?;
        }
        None => {
            write!(f, "{}>(", map_header.value_value_kind)?;
        }
    }

    let child_count = map_header.length * 2;

    match (child_count, context.print_mode) {
        (0, _) => {
            write!(f, ")")?;
        }
        (child_count, PrintMode::SingleLine) => {
            for i in 0..child_count {
                if i > 0 {
                    write!(f, ", ")?;
                }
                format_value_tree(f, traverser, context)?;
            }
            write!(f, ")")?;
        }
        (
            child_count,
            PrintMode::MultiLine {
                indent_size: spaces_per_indent,
                base_indent,
                ..
            },
        ) => {
            let child_indent_size = base_indent + spaces_per_indent * parent_depth;
            let child_indent = " ".repeat(child_indent_size);
            let parent_indent = &child_indent[0..child_indent_size - spaces_per_indent];
            write!(f, "\n")?;
            for _ in 0..child_count {
                write!(f, "{}", child_indent)?;
                format_value_tree(f, traverser, context)?;
                write!(f, ",\n")?;
            }

            write!(f, "{})", parent_indent)?;
        }
    }

    consume_container_end(traverser)?;
    Ok(())
}

fn format_terminal_value<F: fmt::Write, E: FormattableCustomExtension>(
    f: &mut F,
    context: &NestedStringDisplayContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    value_ref: TerminalValueRef<E::CustomTraversal>,
) -> Result<(), FormattingError> {
    let type_name = context
        .schema
        .resolve_type_metadata(type_index)
        .and_then(|m| m.get_name());
    match value_ref {
        TerminalValueRef::Bool(value) => write!(f, "{}", value)?,
        TerminalValueRef::I8(value) => write!(f, "{}i8", value)?,
        TerminalValueRef::I16(value) => write!(f, "{}i16", value)?,
        TerminalValueRef::I32(value) => write!(f, "{}i32", value)?,
        TerminalValueRef::I64(value) => write!(f, "{}i64", value)?,
        TerminalValueRef::I128(value) => write!(f, "{}i128", value)?,
        TerminalValueRef::U8(value) => write!(f, "{}u8", value)?,
        TerminalValueRef::U16(value) => write!(f, "{}u16", value)?,
        TerminalValueRef::U32(value) => write!(f, "{}u32", value)?,
        TerminalValueRef::U64(value) => write!(f, "{}u64", value)?,
        TerminalValueRef::U128(value) => write!(f, "{}u128", value)?,
        TerminalValueRef::String(value) => write!(f, "\"{}\"", value)?,
        TerminalValueRef::Custom(ref value) => {
            match type_name {
                Some(type_name) => {
                    write!(f, "{}:{}(", value_ref.value_kind(), type_name)?;
                }
                None => {
                    write!(f, "{}(", value_ref.value_kind())?;
                }
            }
            E::display_string_content(f, &context.custom_context, value)?;
            write!(f, ")")?;
            return Ok(());
        }
    }
    // Handle the normal terminal values which haven't returned already
    // If the type has a type-name, append it after the :u8, eg "132u64:Epoch"
    // This might arise from - eg - transparent wrapped types
    if let Some(type_name) = type_name {
        write!(f, ":{}", type_name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::*;

    #[derive(Sbor, Hash, Eq, PartialEq)]
    enum TestEnum {
        UnitVariant,
        SingleFieldVariant { field: u8 },
        DoubleStructVariant { field1: u8, field2: u8 },
    }

    #[derive(Sbor)]
    struct MyUnitStruct;

    #[derive(BasicSbor)]
    struct MyComplexTupleStruct(
        Vec<u16>,
        Vec<u16>,
        Vec<u8>,
        Vec<u8>,
        IndexMap<TestEnum, MyFieldStruct>,
        BTreeMap<String, MyUnitStruct>,
        TestEnum,
        TestEnum,
        TestEnum,
        MyFieldStruct,
        Vec<MyUnitStruct>,
        BasicValue,
    );

    #[derive(Sbor)]
    struct MyFieldStruct {
        field1: u64,
        field2: Vec<String>,
    }

    #[test]
    fn complex_value_formatting() {
        let (type_index, schema) =
            generate_full_schema_from_single_type::<MyComplexTupleStruct, NoCustomSchema>();
        let value = MyComplexTupleStruct(
            vec![1, 2, 3],
            vec![],
            vec![],
            vec![1, 2, 3],
            indexmap! {
                TestEnum::UnitVariant => MyFieldStruct { field1: 1, field2: vec!["hello".to_string()] },
                TestEnum::SingleFieldVariant { field: 1 } => MyFieldStruct { field1: 2, field2: vec!["world".to_string()] },
                TestEnum::DoubleStructVariant { field1: 1, field2: 2 } => MyFieldStruct { field1: 3, field2: vec!["!".to_string()] },
            },
            btreemap! {
                "hello".to_string() => MyUnitStruct,
                "world".to_string() => MyUnitStruct,
            },
            TestEnum::UnitVariant,
            TestEnum::SingleFieldVariant { field: 1 },
            TestEnum::DoubleStructVariant {
                field1: 3,
                field2: 5,
            },
            MyFieldStruct {
                field1: 21,
                field2: vec!["hello".to_string(), "world!".to_string()],
            },
            vec![MyUnitStruct, MyUnitStruct],
            Value::Tuple {
                fields: vec![
                    Value::Enum {
                        discriminator: 32,
                        fields: vec![],
                    },
                    Value::Enum {
                        discriminator: 21,
                        fields: vec![Value::I32 { value: -3 }],
                    },
                ],
            },
        );
        let payload = basic_encode(&value).unwrap();

        let expected_annotated_single_line = r###"Tuple:MyComplexTupleStruct(Array<U16>(1u16, 2u16, 3u16), Array<U16>(), Array<U8>(), Array<U8>(Hex("010203")), Map<Enum:TestEnum, Tuple:MyFieldStruct>(Enum:TestEnum(0u8:UnitVariant), Tuple:MyFieldStruct(field1 = 1u64, field2 = Array<String>("hello")), Enum:TestEnum(1u8:SingleFieldVariant, field = 1u8), Tuple:MyFieldStruct(field1 = 2u64, field2 = Array<String>("world")), Enum:TestEnum(2u8:DoubleStructVariant, field1 = 1u8, field2 = 2u8), Tuple:MyFieldStruct(field1 = 3u64, field2 = Array<String>("!"))), Map<String, Tuple:MyUnitStruct>("hello", Tuple:MyUnitStruct(), "world", Tuple:MyUnitStruct()), Enum:TestEnum(0u8:UnitVariant), Enum:TestEnum(1u8:SingleFieldVariant, field = 1u8), Enum:TestEnum(2u8:DoubleStructVariant, field1 = 3u8, field2 = 5u8), Tuple:MyFieldStruct(field1 = 21u64, field2 = Array<String>("hello", "world!")), Array<Tuple:MyUnitStruct>(Tuple:MyUnitStruct(), Tuple:MyUnitStruct()), Tuple(Enum(32u8), Enum(21u8, -3i32)))"###;
        let display_context = ValueDisplayParameters::Annotated {
            display_mode: DisplayMode::NestedString,
            print_mode: PrintMode::SingleLine,
            schema: &schema,
            custom_context: Default::default(),
            type_index,
        };
        assert_eq!(
            &BasicRawPayload::new_from_valid_slice_with_checks(&payload)
                .unwrap()
                .to_string(display_context),
            expected_annotated_single_line,
        );

        let expected_annotated_multi_line = r###"Tuple:MyComplexTupleStruct(
            Array<U16>(
                1u16,
                2u16,
                3u16,
            ),
            Array<U16>(),
            Array<U8>(),
            Array<U8>(Hex("010203")),
            Map<Enum:TestEnum, Tuple:MyFieldStruct>(
                Enum:TestEnum(0u8:UnitVariant),
                Tuple:MyFieldStruct(
                    field1 = 1u64,
                    field2 = Array<String>(
                        "hello",
                    ),
                ),
                Enum:TestEnum(
                    field = 1u8,
                ),
                Tuple:MyFieldStruct(
                    field1 = 2u64,
                    field2 = Array<String>(
                        "world",
                    ),
                ),
                Enum:TestEnum(
                    field1 = 1u8,
                    field2 = 2u8,
                ),
                Tuple:MyFieldStruct(
                    field1 = 3u64,
                    field2 = Array<String>(
                        "!",
                    ),
                ),
            ),
            Map<String, Tuple:MyUnitStruct>(
                "hello",
                Tuple:MyUnitStruct(),
                "world",
                Tuple:MyUnitStruct(),
            ),
            Enum:TestEnum(0u8:UnitVariant),
            Enum:TestEnum(
                field = 1u8,
            ),
            Enum:TestEnum(
                field1 = 3u8,
                field2 = 5u8,
            ),
            Tuple:MyFieldStruct(
                field1 = 21u64,
                field2 = Array<String>(
                    "hello",
                    "world!",
                ),
            ),
            Array<Tuple:MyUnitStruct>(
                Tuple:MyUnitStruct(),
                Tuple:MyUnitStruct(),
            ),
            Tuple(
                Enum(32u8),
                Enum(
                    -3i32,
                ),
            ),
        )"###;
        let display_context = ValueDisplayParameters::Annotated {
            display_mode: DisplayMode::NestedString,
            print_mode: PrintMode::MultiLine {
                indent_size: 4,
                base_indent: 8,
                first_line_indent: 0,
            },
            schema: &schema,
            custom_context: Default::default(),
            type_index,
        };
        assert_eq!(
            &BasicRawPayload::new_from_valid_slice_with_checks(&payload)
                .unwrap()
                .to_string(display_context),
            expected_annotated_multi_line,
        );
    }
}

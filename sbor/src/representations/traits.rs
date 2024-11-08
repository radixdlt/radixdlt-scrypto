use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub trait CustomDisplayContext<'a>: Default + Copy {
    type CustomExtension: FormattableCustomExtension<CustomDisplayContext<'a> = Self>;
}

pub trait FormattableCustomExtension: CustomExtension + Copy {
    type CustomDisplayContext<'a>: CustomDisplayContext<'a>;

    /// The gives the inner formatted representation of the value.
    /// This function should write the value content to the formatter.
    ///
    /// * The rust-like representation is as a newtype: CustomValueKind(<value_content>)
    /// * The nested string representation is identical: CustomValueKind(<value_content>)
    fn display_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error>;

    /// This should output code to generate the value.
    fn code_generation_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error>;

    /// The gives the inner debug representation of the value.
    /// This function should write the value content to the formatter.
    ///
    /// If overriden, this should be a more concise representation than [`display_string_content`].
    ///
    /// [`display_string_content`]: [FormattableCustomExtension::display_string_content]
    fn debug_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        f: &mut F,
        context: &Self::CustomDisplayContext<'a>,
        value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error> {
        Self::display_string_content(f, context, value)
    }
}

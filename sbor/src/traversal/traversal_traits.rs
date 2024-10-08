use crate::internal_prelude::*;
use core::ops::ControlFlow;

pub trait CustomTraversal: Copy + Debug + Clone + PartialEq + Eq + 'static {
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

// We add this allow so that the placeholder names don't have to start with underscores
#[allow(unused_variables)]
pub trait UntypedPayloadVisitor<'de, T: CustomTraversal> {
    type Output<'t>;

    #[inline]
    #[must_use]
    fn on_container_start<'t>(
        &mut self,
        details: OnContainerStart<'t, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        ControlFlow::Continue(())
    }

    #[inline]
    #[must_use]
    fn on_terminal_value<'t>(
        &mut self,
        details: OnTerminalValue<'t, 'de, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        ControlFlow::Continue(())
    }

    #[inline]
    #[must_use]
    fn on_terminal_value_batch<'t>(
        &mut self,
        details: OnTerminalValueBatch<'t, 'de, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        ControlFlow::Continue(())
    }

    #[inline]
    #[must_use]
    fn on_container_end<'t>(
        &mut self,
        details: OnContainerEnd<'t, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_error<'t>(&mut self, details: OnError<'t, T>) -> Self::Output<'t>;

    #[must_use]
    fn on_traversal_end<'t>(&mut self, details: OnTraversalEnd<'t, T>) -> Self::Output<'t>;
}

pub struct OnContainerStart<'t, T: CustomTraversal> {
    pub header: ContainerHeader<T>,
    pub location: Location<'t, T>,
    /// If requesting to break, the traversal can be continued with this action.
    /// This will be optimized out if the visitor doesn't use it.
    pub resume_action: NextAction<T>,
}

pub struct OnTerminalValue<'t, 'de, T: CustomTraversal> {
    pub value: TerminalValueRef<'de, T>,
    pub location: Location<'t, T>,
    /// If requesting to break, the traversal can be continued with this action.
    /// This will be optimized out if the visitor doesn't use it.
    pub resume_action: NextAction<T>,
}

pub struct OnTerminalValueBatch<'t, 'de, T: CustomTraversal> {
    pub value_batch: TerminalValueBatchRef<'de>,
    pub location: Location<'t, T>,
    /// If requesting to break, the traversal can be continued with this action.
    /// This will be optimized out if the visitor doesn't require it.
    pub resume_action: NextAction<T>,
}

pub struct OnContainerEnd<'t, T: CustomTraversal> {
    pub header: ContainerHeader<T>,
    pub location: Location<'t, T>,
    /// If requesting to break, the traversal can be continued with this action.
    /// This will be optimized out if the visitor doesn't require it.
    pub resume_action: NextAction<T>,
}

pub struct OnTraversalEnd<'t, T: CustomTraversal> {
    pub location: Location<'t, T>,
}

pub struct OnError<'t, T: CustomTraversal> {
    pub error: DecodeError,
    pub location: Location<'t, T>,
}

// We add this allow so that the placeholder names don't have to start with underscores
#[allow(unused_variables)]
pub trait TypedPayloadVisitor<'de, E: CustomExtension> {
    type Output<'t, 's>
    where
        's: 't;

    #[inline]
    #[must_use]
    fn on_container_start<'t, 's>(
        &mut self,
        details: OnContainerStartTyped<'t, 's, E::CustomTraversal>,
    ) -> ControlFlow<Self::Output<'t, 's>> {
        ControlFlow::Continue(())
    }

    #[inline]
    #[must_use]
    fn on_terminal_value<'t, 's>(
        &mut self,
        details: OnTerminalValueTyped<'t, 's, 'de, E::CustomTraversal>,
    ) -> ControlFlow<Self::Output<'t, 's>> {
        ControlFlow::Continue(())
    }

    #[inline]
    #[must_use]
    fn on_terminal_value_batch<'t, 's>(
        &mut self,
        details: OnTerminalValueBatchTyped<'t, 's, 'de, E::CustomTraversal>,
    ) -> ControlFlow<Self::Output<'t, 's>> {
        ControlFlow::Continue(())
    }

    #[inline]
    #[must_use]
    fn on_container_end<'t, 's>(
        &mut self,
        details: OnContainerEndTyped<'t, 's, E::CustomTraversal>,
    ) -> ControlFlow<Self::Output<'t, 's>> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_error<'t, 's>(&mut self, details: OnErrorTyped<'t, 's, E>) -> Self::Output<'t, 's>;

    #[must_use]
    fn on_traversal_end<'t, 's>(&mut self, details: OnTraversalEndTyped) -> Self::Output<'t, 's>;
}

pub struct OnContainerStartTyped<'t, 's, T: CustomTraversal> {
    pub local_type_id: LocalTypeId,
    pub header: ContainerHeader<T>,
    pub location: TypedLocation<'t, 's, T>,
}

pub struct OnTerminalValueTyped<'t, 's, 'de, T: CustomTraversal> {
    pub local_type_id: LocalTypeId,
    pub value: TerminalValueRef<'de, T>,
    pub location: TypedLocation<'t, 's, T>,
}

pub struct OnTerminalValueBatchTyped<'t, 's, 'de, T: CustomTraversal> {
    pub local_type_id: LocalTypeId,
    pub value_batch: TerminalValueBatchRef<'de>,
    pub location: TypedLocation<'t, 's, T>,
}

pub struct OnContainerEndTyped<'t, 's, T: CustomTraversal> {
    pub local_type_id: LocalTypeId,
    pub location: TypedLocation<'t, 's, T>,
}

pub struct OnTraversalEndTyped {}

pub struct OnErrorTyped<'t, 's, E: CustomExtension> {
    pub error: TypedTraversalError<E>,
    pub location: TypedLocation<'t, 's, E::CustomTraversal>,
}

use crate::internal_prelude::*;

pub struct ValidatingVisitor;

impl<'de, T: CustomTraversal + 'static> UntypedPayloadVisitor<'de, T> for ValidatingVisitor {
    type Output<'t> = Result<usize, DecodeError>;

    fn on_error<'t>(&mut self, details: OnError<'t, T>) -> Self::Output<'t> {
        Err(details.error)
    }

    fn on_traversal_end<'t>(&mut self, details: OnTraversalEnd<'t, T>) -> Self::Output<'t> {
        Ok(details.location.end_offset)
    }
}

pub struct EventStreamVisitor<'de, T: CustomTraversal> {
    pub next_action: SuspendableNextAction<T>,
    pub next_event: Option<TraversalEvent<'de, T>>,
}

pub enum SuspendableNextAction<T: CustomTraversal> {
    Action(NextAction<T>),
    Errored,
    Ended,
}

impl<'de, T: CustomTraversal + 'static> UntypedPayloadVisitor<'de, T>
    for EventStreamVisitor<'de, T>
{
    type Output<'t> = Location<'t, T>;

    fn on_container_start<'t>(
        &mut self,
        details: OnContainerStart<'t, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        self.next_action = SuspendableNextAction::Action(details.resume_action);
        self.next_event = Some(TraversalEvent::ContainerStart(details.header));
        ControlFlow::Break(details.location)
    }

    fn on_terminal_value<'t>(
        &mut self,
        details: OnTerminalValue<'t, 'de, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        self.next_action = SuspendableNextAction::Action(details.resume_action);
        self.next_event = Some(TraversalEvent::TerminalValue(details.value));
        ControlFlow::Break(details.location)
    }

    fn on_terminal_value_batch<'t>(
        &mut self,
        details: OnTerminalValueBatch<'t, 'de, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        self.next_action = SuspendableNextAction::Action(details.resume_action);
        self.next_event = Some(TraversalEvent::TerminalValueBatch(details.value_batch));
        ControlFlow::Break(details.location)
    }

    fn on_container_end<'t>(
        &mut self,
        details: OnContainerEnd<'t, T>,
    ) -> ControlFlow<Self::Output<'t>> {
        self.next_action = SuspendableNextAction::Action(details.resume_action);
        self.next_event = Some(TraversalEvent::ContainerEnd(details.header));
        ControlFlow::Break(details.location)
    }

    fn on_error<'t>(&mut self, details: OnError<'t, T>) -> Self::Output<'t> {
        self.next_action = SuspendableNextAction::Errored;
        self.next_event = Some(TraversalEvent::DecodeError(details.error));
        details.location
    }

    fn on_traversal_end<'t>(&mut self, details: OnTraversalEnd<'t, T>) -> Self::Output<'t> {
        self.next_action = SuspendableNextAction::Ended;
        self.next_event = Some(TraversalEvent::End);
        details.location
    }
}

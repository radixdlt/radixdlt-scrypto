use radix_engine_interface::prelude::{BlueprintInfo, Emitter};
use crate::system::checkers::ApplicationEventChecker;

#[derive(Debug, Default)]
pub struct ResourceEventChecker {
}

#[derive(Debug, Default)]
pub struct ResourceEventCheckerResults {
}

impl ApplicationEventChecker for ResourceEventChecker {
    type ApplicationEventCheckerResults = ResourceEventCheckerResults;

    fn on_event(&mut self, _info: BlueprintInfo, _emitter: Emitter, _event: &Vec<u8>) {
    }

    fn on_finish(&self) -> Self::ApplicationEventCheckerResults {
        ResourceEventCheckerResults {}
    }
}

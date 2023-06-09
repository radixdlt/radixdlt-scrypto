use crate::internal_prelude::*;

pub mod transfer_xrd;

pub fn get_all_scenarios() -> Vec<Box<dyn ScenarioCore>> {
    vec![Box::new(transfer_xrd::TransferXrdScenario::new())]
}

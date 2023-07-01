use super::*;

pub fn get_builder_for_every_scenario() -> AllScenariosIterator {
    AllScenariosIterator::default()
}

#[derive(Default)]
pub struct AllScenariosIterator {
    index: usize,
}

impl Iterator for AllScenariosIterator {
    type Item = Box<dyn FnOnce(ScenarioCore) -> Box<dyn ScenarioInstance>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            1 => Some(Box::new(|core| {
                transfer_xrd::TransferXrdScenarioCreator::create(core)
            })),
            2 => Some(Box::new(|core| {
                radiswap::RadiswapScenarioCreator::create(core)
            })),
            3 => Some(Box::new(|core| metadata::MetadataScenario::create(core))),
            _ => None,
        }
    }
}

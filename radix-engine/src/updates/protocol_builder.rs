use super::*;
use crate::internal_prelude::*;

#[derive(Clone)]
pub struct ProtocolBuilder {
    settings: ProtocolSettings,
}

#[derive(Clone)]
pub struct ProtocolSettings {
    pub network_definition: NetworkDefinition,
    pub babylon: BabylonSettings,
    pub anemone: AnemoneSettings,
    pub bottlenose: BottlenoseSettings,
    pub cuttlefish_part1: CuttlefishPart1Settings,
    pub cuttlefish_part2: CuttlefishPart2Settings,
}

impl ProtocolSettings {
    pub fn resolve_generator_for_update(
        &self,
        protocol_version: &ProtocolVersion,
    ) -> Box<dyn ProtocolUpdateGenerator> {
        match protocol_version {
            ProtocolVersion::Unbootstrapped => Box::new(NoOpGenerator),
            ProtocolVersion::Babylon => Box::new(self.babylon.create_generator()),
            ProtocolVersion::Anemone => Box::new(self.anemone.create_generator()),
            ProtocolVersion::Bottlenose => Box::new(self.bottlenose.create_generator()),
            ProtocolVersion::CuttlefishPart1 => Box::new(self.cuttlefish_part1.create_generator()),
            ProtocolVersion::CuttlefishPart2 => Box::new(self.cuttlefish_part2.create_generator()),
        }
    }
}

impl ProtocolBuilder {
    pub fn for_simulator() -> Self {
        Self::for_network(&NetworkDefinition::simulator())
    }

    pub fn for_network(network_definition: &NetworkDefinition) -> Self {
        Self {
            settings: ProtocolSettings {
                network_definition: network_definition.clone(),
                babylon: BabylonSettings::all_enabled_as_default_for_network(network_definition),
                anemone: AnemoneSettings::all_enabled_as_default_for_network(network_definition),
                bottlenose: BottlenoseSettings::all_enabled_as_default_for_network(
                    network_definition,
                ),
                cuttlefish_part1: CuttlefishPart1Settings::all_enabled_as_default_for_network(
                    network_definition,
                ),
                cuttlefish_part2: CuttlefishPart2Settings::all_enabled_as_default_for_network(
                    network_definition,
                ),
            },
        }
    }

    pub fn configure_babylon(
        mut self,
        creator: impl FnOnce(BabylonSettings) -> BabylonSettings,
    ) -> Self {
        self.settings.babylon = creator(self.settings.babylon);
        self
    }

    pub fn configure_anemone(
        mut self,
        creator: impl FnOnce(AnemoneSettings) -> AnemoneSettings,
    ) -> Self {
        self.settings.anemone = creator(self.settings.anemone);
        self
    }

    pub fn configure_bottlenose(
        mut self,
        creator: impl FnOnce(BottlenoseSettings) -> BottlenoseSettings,
    ) -> Self {
        self.settings.bottlenose = creator(self.settings.bottlenose);
        self
    }

    pub fn configure_cuttlefish(
        mut self,
        creator: impl FnOnce(CuttlefishPart1Settings) -> CuttlefishPart1Settings,
    ) -> Self {
        self.settings.cuttlefish_part1 = creator(self.settings.cuttlefish_part1);
        self
    }

    pub fn unbootstrapped(self) -> ProtocolExecutor {
        self.from_to(
            ProtocolVersion::Unbootstrapped,
            ProtocolVersion::Unbootstrapped,
        )
    }

    pub fn from_bootstrap_to(self, protocol_version: ProtocolVersion) -> ProtocolExecutor {
        self.from_to(ProtocolVersion::Unbootstrapped, protocol_version)
    }

    pub fn from_bootstrap_to_latest(self) -> ProtocolExecutor {
        self.from_bootstrap_to(ProtocolVersion::LATEST)
    }

    pub fn only_babylon(self) -> ProtocolExecutor {
        self.from_bootstrap_to(ProtocolVersion::Babylon)
    }

    /// The `start_protocol_version` is assumed to be currently active.
    /// If you want to also run bootstrap (i.e. enact `ProtocolVersion::Babylon`), use the `from_bootstrap_to` method.
    pub fn from_to(
        self,
        start_protocol_version: ProtocolVersion,
        end_protocol_version: ProtocolVersion,
    ) -> ProtocolExecutor {
        ProtocolExecutor::new(
            ProtocolExecutorStart::FromCompleted(start_protocol_version),
            end_protocol_version,
            self.settings,
        )
    }

    /// Discovers the start point from the database
    pub fn from_current_to_latest(self) -> ProtocolExecutor {
        self.from_current_to(ProtocolVersion::LATEST)
    }

    /// Discovers the start point from the database
    pub fn from_current_to(self, end_protocol_version: ProtocolVersion) -> ProtocolExecutor {
        ProtocolExecutor::new(
            ProtocolExecutorStart::ResumeFromCurrent,
            end_protocol_version,
            self.settings,
        )
    }
}

enum ProtocolExecutorStart {
    FromCompleted(ProtocolVersion),
    ResumeFromCurrent,
}

pub struct ProtocolExecutor {
    starting_at: ProtocolExecutorStart,
    update_until: ProtocolVersion,
    settings: ProtocolSettings,
}

impl ProtocolExecutor {
    fn new(
        starting_at: ProtocolExecutorStart,
        update_until: ProtocolVersion,
        settings: ProtocolSettings,
    ) -> Self {
        Self {
            starting_at,
            update_until,
            settings,
        }
    }

    pub fn commit_each_protocol_update(
        self,
        store: &mut (impl SubstateDatabase + CommittableSubstateDatabase),
    ) {
        for update_execution in self.each_protocol_update_executor(&*store) {
            update_execution.run_and_commit(store);
        }
    }

    /// For defaults:
    /// * For the hooks, you can use `&mut ()`
    /// * For the modules you can use `&mut VmModules::default()`
    pub fn commit_each_protocol_update_advanced(
        self,
        store: &mut (impl SubstateDatabase + CommittableSubstateDatabase),
        hooks: &mut impl ProtocolUpdateExecutionHooks,
        modules: &impl VmInitialize,
    ) {
        for update_execution in self.each_protocol_update_executor(&*store) {
            update_execution.run_and_commit_advanced(store, hooks, modules);
        }
    }

    pub fn each_target_protocol_version(
        &self,
        store: &impl SubstateDatabase,
    ) -> impl Iterator<Item = (ProtocolVersion, (usize, usize))> {
        let starting_at = match self.starting_at {
            ProtocolExecutorStart::FromCompleted(protocol_version) => ProtocolUpdateStatusSummary {
                protocol_version,
                update_status: ProtocolUpdateStatus::Complete,
            },
            ProtocolExecutorStart::ResumeFromCurrent => {
                ProtocolUpdateStatusSummarySubstate::load(store).into_unique_version()
            }
        };
        let from_protocol_version = starting_at.protocol_version;
        ProtocolVersion::all_between_inclusive(starting_at.protocol_version, self.update_until)
            .filter_map(move |version| {
                if from_protocol_version == version {
                    match &starting_at.update_status {
                        ProtocolUpdateStatus::Complete => None,
                        ProtocolUpdateStatus::InProgress { latest_commit } => Some((
                            version,
                            (
                                latest_commit.batch_group_index,
                                latest_commit.batch_index + 1,
                            ),
                        )),
                    }
                } else {
                    Some((version, (0, 0)))
                }
            })
    }

    pub fn each_protocol_update_executor(
        self,
        store: &impl SubstateDatabase,
    ) -> impl Iterator<Item = ProtocolUpdateExecutor> {
        self.each_target_protocol_version(store)
            .map(move |(version, start_from_inclusive)| {
                ProtocolUpdateExecutor::continue_for_version(
                    version,
                    &self.settings,
                    start_from_inclusive,
                )
            })
    }
}

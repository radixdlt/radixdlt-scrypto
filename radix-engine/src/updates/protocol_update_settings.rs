use super::*;
use crate::internal_prelude::*;

/// This requires [`ScryptoSbor`] so it can be used to override configuration in the node for tests.
pub trait UpdateSettings: Sized + ScryptoSbor {
    type UpdateGenerator: ProtocolUpdateGenerator;

    fn protocol_version() -> ProtocolVersion;

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self;

    fn all_disabled() -> Self;

    fn create_generator(&self) -> Self::UpdateGenerator;

    fn enable(mut self, prop: impl FnOnce(&mut Self) -> &mut UpdateSetting<NoSettings>) -> Self {
        *prop(&mut self) = UpdateSetting::Enabled(NoSettings);
        self
    }

    fn enable_with<T: UpdateSettingContent>(
        mut self,
        prop: impl FnOnce(&mut Self) -> &mut UpdateSetting<T>,
        setting: T,
    ) -> Self {
        *prop(&mut self) = UpdateSetting::Enabled(setting);
        self
    }

    fn disable<T: UpdateSettingContent>(
        mut self,
        prop: impl FnOnce(&mut Self) -> &mut UpdateSetting<T>,
    ) -> Self {
        *prop(&mut self) = UpdateSetting::Disabled;
        self
    }

    fn set(mut self, updater: impl FnOnce(&mut Self)) -> Self {
        updater(&mut self);
        self
    }
}

pub trait DefaultForNetwork {
    fn default_for_network(network_definition: &NetworkDefinition) -> Self;
}

#[derive(Clone, Sbor)]
pub enum UpdateSetting<T: UpdateSettingContent> {
    Enabled(T),
    Disabled,
}

impl UpdateSetting<NoSettings> {
    pub fn new(is_enabled: bool) -> Self {
        if is_enabled {
            Self::Enabled(NoSettings)
        } else {
            Self::Disabled
        }
    }
}

impl<T: UpdateSettingContent> UpdateSetting<T> {
    pub fn enabled_as_default_for_network(network_definition: &NetworkDefinition) -> Self {
        Self::Enabled(T::default_setting(network_definition))
    }
}

pub trait UpdateSettingContent {
    fn default_setting(_: &NetworkDefinition) -> Self;
}

#[derive(Clone, Copy, Debug, Default, Sbor)]
pub struct NoSettings;

impl UpdateSettingContent for NoSettings {
    fn default_setting(_: &NetworkDefinition) -> Self {
        NoSettings
    }
}

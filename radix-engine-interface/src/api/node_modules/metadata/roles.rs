pub use sbor::rust::vec;
pub use sbor::rust::vec::Vec;

pub const METADATA_SETTER_ROLE: &str = "metadata_setter";
pub const METADATA_SETTER_UPDATER_ROLE: &str = "metadata_setter_updater";

pub const METADATA_LOCKER_ROLE: &str = "metadata_locker";
pub const METADATA_LOCKER_UPDATER_ROLE: &str = "metadata_locker_updater";

pub struct MetadataRoles<T> {
    pub metadata_setter: T,
    pub metadata_setter_updater: T,
    pub metadata_locker: T,
    pub metadata_locker_updater: T,
}

impl<T> MetadataRoles<T> {
    pub fn list(self) -> Vec<(&'static str, T)> {
        vec![
            (METADATA_SETTER_ROLE, self.metadata_setter),
            (METADATA_SETTER_UPDATER_ROLE, self.metadata_setter_updater),
            (METADATA_LOCKER_ROLE, self.metadata_locker),
            (METADATA_LOCKER_UPDATER_ROLE, self.metadata_locker_updater),
        ]
    }
}

#[macro_export]
macro_rules! metadata_roles {
    {$($role:ident => $rule:expr;)*} => ({
        internal_roles!(MetadataRoles, $($role => $rule;)*)
    });
}

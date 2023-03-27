use crate::{api::types::*, blueprints::resource::AccessRule};
use sbor::rust::fmt::Debug;

pub trait ClientAuthApi<E: Debug> {
    fn get_auth_zone(&mut self) -> Result<NodeId, E>;

    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), E>;
}

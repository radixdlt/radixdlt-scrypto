use sbor::rust::fmt::Debug;

use crate::api::component::ComponentAddress;
use crate::api::types::*;
use crate::blueprints::resource::AccessRule;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct IdentityCreateInvocation {
    pub access_rule: AccessRule,
}

impl Invocation for IdentityCreateInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for IdentityCreateInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for IdentityCreateInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Identity(IdentityInvocation::Create(self)).into()
    }
}

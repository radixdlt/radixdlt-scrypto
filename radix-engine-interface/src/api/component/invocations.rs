use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ComponentGlobalizeWithOwnerInvocation {
    pub component_id: ComponentId,
    pub owner_badge: NonFungibleGlobalId,
}

impl Invocation for ComponentGlobalizeWithOwnerInvocation {
    type Output = ComponentAddress;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Component(ComponentFn::GlobalizeWithOwner))
    }
}

impl SerializableInvocation for ComponentGlobalizeWithOwnerInvocation {
    type ScryptoOutput = ComponentAddress;

    fn native_fn() -> NativeFn {
        NativeFn::Component(ComponentFn::GlobalizeWithOwner)
    }
}

impl Into<CallTableInvocation> for ComponentGlobalizeWithOwnerInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Component(ComponentInvocation::GlobalizeWithOwner(self)).into()
    }
}

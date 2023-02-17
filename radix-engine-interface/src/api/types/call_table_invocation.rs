use crate::api::node_modules::auth::*;
use crate::api::package::PackageAddress;
use crate::api::package::*;
use crate::api::types::*;
use crate::data::scrypto_encode;
use crate::data::ScryptoValue;
use crate::*;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

// TODO: Remove enum
#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum CallTableInvocation {
    Native(NativeInvocation),
    Scrypto(ScryptoInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub fn_name: String,
    pub receiver: Option<(ScryptoReceiver, NodeModuleId)>,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoInvocation {
    type Output = ScryptoValue;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Scrypto(ScryptoFnIdentifier {
            package_address: self.package_address,
            blueprint_name: self.blueprint_name.clone(),
            ident: self.fn_name.clone(),
        })
    }
}

impl Into<CallTableInvocation> for ScryptoInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Scrypto(self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum NativeInvocation {
    AccessRulesChain(AccessRulesChainInvocation),
    Package(PackageInvocation),
}

impl Into<CallTableInvocation> for NativeInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Native(self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessRulesChainInvocation {
    AddAccessCheck(AccessRulesAddAccessCheckInvocation),
    SetMethodAccessRule(AccessRulesSetMethodAccessRuleInvocation),
    SetMethodMutability(AccessRulesSetMethodMutabilityInvocation),
    SetGroupAccessRule(AccessRulesSetGroupAccessRuleInvocation),
    SetGroupMutability(AccessRulesSetGroupMutabilityInvocation),
    GetLength(AccessRulesGetLengthInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum PackageInvocation {
    Publish(PackagePublishInvocation),
    PublishNative(PackagePublishNativeInvocation),
    ClaimRoyalty(PackageClaimRoyaltyInvocation),
}

impl NativeInvocation {
    pub fn refs(&self) -> HashSet<RENodeId> {
        let mut refs = HashSet::new();
        match self {
            NativeInvocation::Package(package_method) => match package_method {
                PackageInvocation::Publish(..) => {}
                PackageInvocation::PublishNative(..) => {}
                PackageInvocation::ClaimRoyalty(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Package(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::AccessRulesChain(access_rules_method) => match access_rules_method {
                AccessRulesChainInvocation::AddAccessCheck(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetMethodAccessRule(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetMethodMutability(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetGroupAccessRule(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetGroupMutability(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::GetLength(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
        }

        refs
    }
}

fn get_native_fn<T: SerializableInvocation>(_: &T) -> NativeFn {
    T::native_fn()
}

impl NativeInvocation {
    pub fn flatten(&self) -> (NativeFn, Vec<u8>) {
        let (native_fn, encoding) = match self {
            NativeInvocation::AccessRulesChain(i) => match i {
                AccessRulesChainInvocation::AddAccessCheck(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetMethodAccessRule(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetMethodMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetGroupAccessRule(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetGroupMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::GetLength(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Package(i) => match i {
                PackageInvocation::Publish(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::PublishNative(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::ClaimRoyalty(i) => (get_native_fn(i), scrypto_encode(i)),
            },
        };

        (
            native_fn,
            encoding.expect("Failed to encode native invocation"),
        )
    }
}

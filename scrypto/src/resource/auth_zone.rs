use sbor::rust::collections::BTreeSet;
use sbor::rust::string::ToString;
use sbor::*;

use crate::core::{FnIdentifier, Receiver};
use crate::engine::{api::*, call_engine};
use crate::math::Decimal;
use crate::resource::*;
use crate::native_functions;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneClearInput {}

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    native_functions! {
        Receiver::AuthZoneRef => {
            pub fn pop() -> Proof {
                AuthZonePopInput {}
            }

            pub fn create_proof(resource_address: ResourceAddress) -> Proof {
                AuthZoneCreateProofInput {
                    resource_address
                }
            }

            pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
                AuthZoneCreateProofByAmountInput {
                    amount, resource_address
                }
            }

            pub fn create_proof_by_ids(ids: &BTreeSet<NonFungibleId>, resource_address: ResourceAddress) -> Proof {
                AuthZoneCreateProofByIdsInput {
                    ids: ids.clone(),
                    resource_address
                }
            }
        }
    }

    pub fn push<P: Into<Proof>>(proof: P) {
        let proof: Proof = proof.into();
        let input = RadixEngineInput::InvokeMethod(
            Receiver::AuthZoneRef,
            FnIdentifier::Native("push".to_string()),
            scrypto::buffer::scrypto_encode(&(AuthZonePushInput { proof })),
        );
        call_engine(input)
    }
}

use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::Decimal;
use crate::*;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::types::*;
use radix_engine_interface::constants::RESOURCE_PACKAGE;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;
use sbor::*;

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePopInput {}

pub type AuthZonePopOutput = Proof;

pub const AUTH_ZONE_PUSH_IDENT: &str = "push";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

impl Clone for AuthZonePushInput {
    fn clone(&self) -> Self {
        Self {
            proof: Proof(self.proof.0),
        }
    }
}

pub type AuthZonePushOutput = ();

pub const AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofOfAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfAmountOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT: &str = "create_proof_of_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofOfNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfNonFungiblesOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT: &str = "create_proof_of_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofOfAllInput {
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfAllOutput = Proof;

pub const AUTH_ZONE_DROP_PROOFS_IDENT: &str = "drop_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDropProofsInput {}

pub type AuthZoneDropProofsOutput = ();

pub const AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT: &str = "drop_signature_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDropSignatureProofsInput {}

pub type AuthZoneDropSignatureProofsOutput = ();

pub const AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT: &str = "drop_regular_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDropRegularProofsInput {}

pub type AuthZoneDropRegularProofsOutput = ();

pub const AUTH_ZONE_DRAIN_IDENT: &str = "drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDrainInput {}

pub type AuthZoneDrainOutput = Vec<Proof>;

pub const AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT: &str = "assert_access_rule";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneAssertAccessRuleInput {
    pub rule: AccessRule,
}

pub type AuthZoneAssertAccessRuleOutput = ();

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
#[sbor(transparent)]
pub struct AuthZoneRef(pub NodeId);

impl Describe<ScryptoCustomTypeKind> for AuthZoneRef {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::Novel(const_sha1::sha1("OwnedAuthZone".as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names("OwnedAuthZone"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(
                    Some(RESOURCE_PACKAGE),
                    AUTH_ZONE_BLUEPRINT.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

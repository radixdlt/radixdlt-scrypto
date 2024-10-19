use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_common::prelude::ManifestResourceConstraints;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Worktop(pub Own);

impl Worktop {
    pub fn drop<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        let _rtn = api.call_function(
            RESOURCE_PACKAGE,
            WORKTOP_BLUEPRINT,
            WORKTOP_DROP_IDENT,
            scrypto_encode(&WorktopDropInput {
                worktop: OwnedWorktop(self.0),
            })
            .unwrap(),
        )?;

        Ok(())
    }

    pub fn put<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_PUT_IDENT,
            scrypto_encode(&WorktopPutInput { bucket }).unwrap(),
        )?;

        Ok(())
    }

    pub fn take<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_TAKE_IDENT,
            scrypto_encode(&WorktopTakeInput {
                resource_address,
                amount,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn take_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&WorktopTakeNonFungiblesInput {
                resource_address,
                ids,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn take_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_TAKE_ALL_IDENT,
            scrypto_encode(&WorktopTakeAllInput { resource_address }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn assert_contains<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_ASSERT_CONTAINS_IDENT,
            scrypto_encode(&WorktopAssertContainsInput { resource_address }).unwrap(),
        )?;
        Ok(())
    }

    pub fn assert_contains_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT,
            scrypto_encode(&WorktopAssertContainsAmountInput {
                resource_address,
                amount,
            })
            .unwrap(),
        )?;
        Ok(())
    }

    pub fn assert_contains_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT,
            scrypto_encode(&WorktopAssertContainsNonFungiblesInput {
                resource_address,
                ids,
            })
            .unwrap(),
        )?;
        Ok(())
    }

    pub fn assert_resources_include<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        constraints: ManifestResourceConstraints,
        api: &mut Y,
    ) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_ASSERT_RESOURCES_INCLUDE_IDENT,
            scrypto_encode(&WorktopAssertResourcesIncludeInput { constraints }).unwrap(),
        )?;
        Ok(())
    }

    pub fn assert_resources_only<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        constraints: ManifestResourceConstraints,
        api: &mut Y,
    ) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_ASSERT_RESOURCES_ONLY_IDENT,
            scrypto_encode(&WorktopAssertResourcesOnlyInput { constraints }).unwrap(),
        )?;
        Ok(())
    }

    pub fn drain<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Vec<Bucket>, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            WORKTOP_DRAIN_IDENT,
            scrypto_encode(&WorktopDrainInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}

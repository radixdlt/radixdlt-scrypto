use lazy_static::lazy_static;
use radix_engine_interface::blueprints::package::*;

use crate::blueprints::access_controller::v1::*;
use crate::blueprints::access_controller::v2::*;
use crate::blueprints::account::*;
use crate::blueprints::consensus_manager::*;
use crate::blueprints::identity::*;
use crate::blueprints::locker::*;
use crate::blueprints::package::*;
use crate::blueprints::pool::v1::package::*;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_processor::*;
use crate::blueprints::transaction_tracker::TransactionTrackerNativePackage;
use crate::object_modules::metadata::*;
use crate::object_modules::role_assignment::*;
use crate::object_modules::royalty::*;

lazy_static! {
    pub static ref CONSENSUS_MANAGER_PACKAGE_DEFINITION: PackageDefinition =
        ConsensusManagerNativePackage::definition();
    pub static ref ACCOUNT_PACKAGE_DEFINITION: PackageDefinition =
        AccountNativePackage::definition();
    pub static ref IDENTITY_PACKAGE_DEFINITION: PackageDefinition =
        IdentityNativePackage::definition();
    pub static ref ACCESS_CONTROLLER_PACKAGE_DEFINITION_V1_0: PackageDefinition =
        AccessControllerV1NativePackage::definition();
    pub static ref ACCESS_CONTROLLER_PACKAGE_DEFINITION_V2_0: PackageDefinition =
        AccessControllerV2NativePackage::definition();
    pub static ref POOL_PACKAGE_DEFINITION_V1_0: PackageDefinition =
        PoolNativePackage::definition(PoolV1MinorVersion::Zero);
    pub static ref POOL_PACKAGE_DEFINITION_V1_1: PackageDefinition =
        PoolNativePackage::definition(PoolV1MinorVersion::One);
    pub static ref TRANSACTION_TRACKER_PACKAGE_DEFINITION: PackageDefinition =
        TransactionTrackerNativePackage::definition();
    pub static ref RESOURCE_PACKAGE_DEFINITION: PackageDefinition =
        ResourceNativePackage::definition();
    pub static ref PACKAGE_PACKAGE_DEFINITION: PackageDefinition =
        PackageNativePackage::definition();
    pub static ref TRANSACTION_PROCESSOR_PACKAGE_DEFINITION: PackageDefinition =
        TransactionProcessorNativePackage::definition();
    pub static ref LOCKER_PACKAGE_DEFINITION: PackageDefinition = LockerNativePackage::definition();
    pub static ref METADATA_PACKAGE_DEFINITION: PackageDefinition =
        MetadataNativePackage::definition();
    pub static ref ROYALTY_PACKAGE_DEFINITION: PackageDefinition =
        RoyaltyNativePackage::definition();
    pub static ref ROLE_ASSIGNMENT_PACKAGE_DEFINITION: PackageDefinition =
        RoleAssignmentNativePackage::definition();
}

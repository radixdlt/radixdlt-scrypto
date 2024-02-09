// TODO: is this the right place to define these constants

use radix_engine_common::types::PartitionOffset;

pub const PACKAGE_CODE_ID: u64 = 0u64;
pub const RESOURCE_CODE_ID: u64 = 1u64;
pub const IDENTITY_CODE_ID: u64 = 2u64;
pub const CONSENSUS_MANAGER_CODE_ID: u64 = 3u64;
pub const ACCOUNT_CODE_ID: u64 = 5u64;
pub const ACCESS_CONTROLLER_CODE_ID: u64 = 6u64;
pub const TRANSACTION_PROCESSOR_CODE_ID: u64 = 7u64;
pub const METADATA_CODE_ID: u64 = 10u64;
pub const ROYALTY_CODE_ID: u64 = 11u64;
pub const ROLE_ASSIGNMENT_CODE_ID: u64 = 12u64;
pub const POOL_V1_0_CODE_ID: u64 = 13u64;
pub const TRANSACTION_TRACKER_CODE_ID: u64 = 14u64;
pub const TEST_UTILS_CODE_ID: u64 = 15u64;
pub const CONSENSUS_MANAGER_SECONDS_PRECISION_CODE_ID: u64 = 16u64;
pub const POOL_V1_1_CODE_ID: u64 = 17u64;

pub const PACKAGE_FIELDS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);
pub const PACKAGE_BLUEPRINTS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);
pub const PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET: PartitionOffset = PartitionOffset(2u8);
// There is no partition offset for the package schema collection as it is directly mapped to SCHEMAS_PARTITION
pub const PACKAGE_ROYALTY_PARTITION_OFFSET: PartitionOffset = PartitionOffset(3u8);
pub const PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(4u8);
pub const PACKAGE_VM_TYPE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(5u8);
pub const PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(6u8);
pub const PACKAGE_INSTRUMENTED_CODE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(7u8);

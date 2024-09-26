use crate::internal_prelude::*;
use crate::kernel::kernel::*;
use crate::system::system_callback::*;
use crate::system::type_info::TypeInfoSubstate;
use crate::transaction::SystemFieldKind;
use crate::updates::*;
use crate::vm::*;
use lazy_static::lazy_static;
use radix_transactions::validation::TransactionValidationConfigurationSubstate;
use sbor::*;

fn generate<S: ScryptoDescribe + ScryptoCheckedBackwardsCompatibleSchema>(
) -> ScryptoSingleTypeSchema {
    generate_single_type_schema::<S, _>()
}

pub fn resolve_system_field_schema(
    system_field: SystemFieldKind,
) -> &'static ScryptoSingleTypeSchema {
    match system_field {
        SystemFieldKind::TypeInfo => &TYPE_INFO_SUBSTATE_SCHEMA,
        SystemFieldKind::VmBoot => &VM_BOOT_SUBSTATE_SCHEMA,
        SystemFieldKind::SystemBoot => &SYSTEM_BOOT_SUBSTATE_SCHEMA,
        SystemFieldKind::KernelBoot => &KERNEL_BOOT_SUBSTATE_SCHEMA,
        SystemFieldKind::TransactionValidationConfiguration => {
            &TRANSACTION_VALIDATION_CONFIGURATION_SUBSTATE_SCHEMA
        }
        SystemFieldKind::ProtocolUpdateStatusSummary => {
            &PROTOCOL_UPDATE_STATUS_SUMMARY_SUBSTATE_SCHEMA
        }
    }
}

pub fn resolve_system_schema_schema() -> &'static ScryptoSingleTypeSchema {
    &SCHEMA_SUBSTATE_SCHEMA
}

#[derive(ScryptoSbor, ScryptoSborAssertion)]
#[sbor_assert(backwards_compatible(cuttlefish = "FILE:schema_substate_cuttlefish_schema.bin"))]
#[sbor(transparent, transparent_name)]
struct SchemaEntrySubstate(PackageSchemaEntrySubstate);

// This is for having schemas to help map system substates
lazy_static! {
    static ref TYPE_INFO_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema = generate::<TypeInfoSubstate>();
    static ref KERNEL_BOOT_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema =
        generate::<KernelBootSubstate>();
    static ref SYSTEM_BOOT_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema =
        generate::<SystemBootSubstate>();
    static ref VM_BOOT_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema = generate::<VmBootSubstate>();
    static ref TRANSACTION_VALIDATION_CONFIGURATION_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema =
        generate::<TransactionValidationConfigurationSubstate>();
    static ref PROTOCOL_UPDATE_STATUS_SUMMARY_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema =
        generate::<ProtocolUpdateStatusSummarySubstate>();
    static ref SCHEMA_SUBSTATE_SCHEMA: ScryptoSingleTypeSchema = generate::<SchemaEntrySubstate>();
}

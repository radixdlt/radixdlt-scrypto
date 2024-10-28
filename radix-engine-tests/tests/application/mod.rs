// We used to use automod, but it breaks various tools
// such as cargo fmt, so let's just list them explicitly.
mod common_transactions;
mod common_transformation_costs;
mod determinism;
mod execution_trace;
mod fuzz_transactions;
mod local_component;
mod metering;
mod preview;
mod preview_v2;
mod stake_reconciliation;
mod static_resource_movements_visitor;
mod storage;
mod stored_external_component;
mod stored_local_component;
mod stored_resource;
mod transaction;

// We used to use automod, but it breaks various tools
// such as cargo fmt, so let's just list them explicitly.
mod address;
mod allocated_address;
mod arguments;
mod assert_bucket_contents;
mod assert_next_call_returns;
mod auth_account;
mod auth_component;
mod auth_mutability;
mod auth_package_token;
mod auth_resource;
mod auth_scenarios;
mod auth_vault;
mod auth_zone;
mod balance_changes;
mod bootstrap;
mod bucket;
mod component;
mod consensus_manager;
mod core;
mod crypto_utils;
mod data_validation;
mod deep_sbor;
mod epoch;
mod error_injection;
mod events;
mod execution_cost;
mod external_bridge;
mod faucet;
mod fee;
mod fee_reserve_states;
mod identity;
mod instructions;
mod invalid_stored_values;
mod kv_store;
mod leaks;
mod metadata;
mod metadata2;
mod metadata3;
mod metadata_component;
mod metadata_identity;
mod metadata_package;
mod metadata_validator;
mod module;
mod package;
mod package_schema;
mod proxy;
mod recallable;
mod reference;
mod remote_generic_args;
mod role_assignment;
mod royalty;
mod royalty_auth;
mod royalty_edge_cases;
mod schema_sanity_check;
mod subintent_auth;
mod subintent_leaks;
mod subintent_lock_fee;
mod subintent_structure;
mod subintent_txn_shape;
mod subintent_verify_parent;
mod subintent_yield;
mod system;
mod system_access_rule;
mod system_actor_collection;
mod system_actor_field;
mod system_call_method;
mod system_db_checker;
mod system_errors;
mod system_genesis_packages;
mod system_global_address;
mod system_kv_store;
mod system_lock_fee;
mod system_module_methods;
mod system_reference;
mod system_role_assignment;
mod toolkit_receipt;
mod track;
mod transaction_limits;
mod transaction_runtime;
mod transaction_tracker;
mod typed_substate_layout;
mod vault;
mod vault_burn;
mod vault_freeze;
mod worktop;

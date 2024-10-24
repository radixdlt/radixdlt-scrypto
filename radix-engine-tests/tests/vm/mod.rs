// We used to use automod, but it breaks various tools
// such as cargo fmt, so let's just list them explicitly.
mod cached_wasm_limits;
mod decimal;
mod logging;
mod logging_limits;
mod native_vm;
mod scrypto_address_reservation;
mod scrypto_cast;
mod scrypto_costing;
mod scrypto_env;
mod scrypto_sbor;
mod scrypto_validation;
mod scrypto_validator;
mod stack_size;
mod system_wasm_buffers;
mod wasm_limits;
mod wasm_memory;
mod wasm_metering;
mod wasm_non_mvp;
mod wasm_validation;

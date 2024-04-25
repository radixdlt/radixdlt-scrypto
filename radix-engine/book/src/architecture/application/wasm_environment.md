# WASM Environment

## Function Entrypoint

An exported function in WASM may be called if it is mapped via `export_name` in a function of a Blueprint Definition.
The argument to the function is a single `i64` which represents a buffer. The first 32 bits refers to a `BufferId`
and the second 32 bits refers to the length of the buffer.

Once sufficient space has been allocated for the buffer, the contents of the buffer can be retrieved by using
the `buffer_consume` call. The contents of the buffer will match the sbor schema described in the function
schema of the Blueprint Definition.

## System Calls

Various `extern` functions are available to be called during execution. These are referred to as `system calls`
and provide the ability to read/write state, invoke methods and functions, and other system-related logic.

The full set of calls can be found in [wasm_api.rs](../../../../../scrypto/src/engine/wasm_api.rs).

## Function Return

The return value from a called exported function in WASM must be an `i64` where the first 32 bits refers to the
32-bit address pointer and the second 32 bits refers to the size of the return object. The contents of the buffer
must match the return value schema of the function in the Blueprint Definition.

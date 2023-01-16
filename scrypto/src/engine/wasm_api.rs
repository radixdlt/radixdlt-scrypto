use radix_engine_interface::api::types::{
    FnIdentifier, LockHandle, RENodeId, ScryptoRENode, ScryptoReceiver, SubstateOffset,
};
use radix_engine_interface::api::{ActorApi, ComponentApi, EngineApi, Invokable};
use radix_engine_interface::data::{ScryptoDecode, ScryptoEncode, ScryptoValue};
use radix_engine_interface::model::CallTableInvocation;
use radix_engine_interface::wasm::*;
use sbor::rust::vec::Vec;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(api: u8, input: *mut u8) -> *mut u8;
}

trait EngineWasmApi {
    const ID: u8 = 0;
    type Input: ScryptoEncode;
    type Output: ScryptoDecode;
}

struct InvokeMethod;

impl EngineWasmApi for InvokeMethod {
    const ID: u8 = 0;
    type Input = (ScryptoReceiver, String, ScryptoValue);
    type Output = ScryptoValue;
}

pub struct Invoke;

impl EngineWasmApi for Invoke {
    const ID: u8 = 0;
    type Input = CallTableInvocation;
    type Output = ScryptoValue;
}

pub struct CreateNode;

impl EngineWasmApi for CreateNode {
    const ID: u8 = 0;
    type Input = ScryptoRENode;
    type Output = ScryptoValue;
}

pub struct GetVisibleNodeIds;

impl EngineWasmApi for GetVisibleNodeIds {
    const ID: u8 = 0;
    type Input = ();
    type Output = ScryptoValue;
}

pub struct DropNode;

impl EngineWasmApi for DropNode {
    const ID: u8 = 0;
    type Input = RENodeId;
    type Output = ScryptoValue;
}

pub struct LockSubstate;

impl EngineWasmApi for LockSubstate {
    const ID: u8 = 0;
    type Input = (RENodeId, SubstateOffset, bool);
    type Output = ScryptoValue;
}

pub struct DropLock;

impl EngineWasmApi for DropLock {
    const ID: u8 = 0;
    type Input = LockHandle;
    type Output = ScryptoValue;
}

pub struct Read;

impl EngineWasmApi for Read {
    const ID: u8 = 0;
    type Input = LockHandle;
    type Output = ScryptoValue;
}

pub struct Write;

impl EngineWasmApi for Write {
    const ID: u8 = 0;
    type Input = (LockHandle, Vec<u8>);
    type Output = ScryptoValue;
}

pub struct GetActor;

impl EngineWasmApi for GetActor {
    const ID: u8 = 0;
    type Input = ();
    type Output = ScryptoValue;
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
fn call_engine_wasm_api<W: EngineWasmApi>(input: W::Input) -> W::Output {
    use crate::buffer::*;

    let input_ptr = scrypto_encode_to_buffer(&input).unwrap();
    let output_ptr = unsafe { radix_engine(input_ptr) };
    scrypto_decode_from_buffer(output_ptr).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn call_engine_wasm_api<W: EngineWasmApi>(input: W::Input) -> W::Output {
    todo!()
}

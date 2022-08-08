use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::marker::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::{ScryptoActor, TypeName};
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::model::ExecutableInstruction;
use transaction::validation::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

pub struct Kernel<
    'p, // parent lifetime
    'g, // lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
    I,  // WASM instance type
    C,  // Fee reserve type
> where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The Transaction signer public keys
    transaction_signers: Vec<EcdsaPublicKey>,
    /// Whether running in sudo mode
    is_system: bool,
    /// The max call depth
    max_depth: usize,
    /// Whether to show trace messages
    trace: bool,

    /// State track
    track: &'g mut Track<'s>,
    /// Wasm engine
    wasm_engine: &'g mut W,
    /// Wasm Instrumenter
    wasm_instrumenter: &'g mut WasmInstrumenter,

    /// Fee reserve
    fee_reserve: &'g mut C,
    /// Fee table
    fee_table: &'g FeeTable,

    /// ID allocator
    id_allocator: IdAllocator,
    /// Call frames
    call_frames: Vec<CallFrame<'p, 'g, 's, W, I, C>>,
}

impl<'p, 'g, 's, W, I, C> Kernel<'p, 'g, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    pub fn new(
        transaction_hash: Hash,
        transaction_signers: Vec<EcdsaPublicKey>,
        is_system: bool,
        max_depth: usize,
        trace: bool,

        track: &'g mut Track<'s>,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,

        fee_reserve: &'g mut C,
        fee_table: &'g FeeTable,
    ) -> Self {
        Self {
            transaction_hash,
            transaction_signers,
            is_system,
            max_depth,
            trace,
            track,
            wasm_engine,
            wasm_instrumenter,
            fee_reserve,
            fee_table,
            id_allocator: IdAllocator::new(IdSpace::Application),
            call_frames: Vec::new(),
        }
    }

    pub fn invoke_function(
        &mut self,
        type_name: TypeName,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        let mut root_frame = CallFrame::new_root(
            self.trace,
            self.transaction_hash,
            self.transaction_signers.clone(), // TODO: remove clone
            self.is_system,
            self.max_depth,
            &mut self.id_allocator,
            self.track,
            self.wasm_engine,
            self.wasm_instrumenter,
            self.fee_reserve,
            self.fee_table,
        );
        root_frame.invoke_function(type_name, fn_ident, input)
    }
}

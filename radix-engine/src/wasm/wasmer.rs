// use crate::wasm::constants::*;
// use crate::wasm::errors::*;
// use crate::wasm::traits::*;
// use sbor::rust::ptr;
// use sbor::rust::string::String;
// use sbor::rust::string::ToString;
// use sbor::rust::vec::Vec;
// use scrypto::values::ScryptoValue;
// use wasmer::*;

// use super::WasmiEnvModule;

// pub struct WasmerScryptoModule<'s> {
//     module: Module,
//     store: &'s Store,
// }

// pub struct WasmerScryptoInstance {
//     instance: Instance,
// }

// pub struct WasmerEnvModule<'r, R: ScryptoRuntime> {
//     runtime: &'r mut R,
// }

// pub struct WasmerEngine {
//     store: Store,
// }

// impl<'s, 'r, I: ScryptoInstance<R>, R: ScryptoRuntime> ScryptoModule<'r, I, R>
//     for WasmerScryptoModule<'s>
// {
//     fn instantiate(&self, runtime: &'r mut R) -> I {
//         fn radix_engine<'ra, RR: ScryptoRuntime>(env: &'ra WasmerEnvModule<RR>, ptr: i32) -> i32 {
//             1
//         }
//         fn use_tbd<'ra, RR: ScryptoRuntime>(env: &'ra WasmerEnvModule<RR>, amount: i32) {}

//         // Create an import object.
//         // let import_object = imports! {
//         //     MODULE_ENV_NAME => {
//         //         RADIX_ENGINE_FUNCTION_NAME => Function::new_native_with_env(&self.store, WasmerEnvModule { runtime }, radix_engine),
//         //         USE_TBD_FUNCTION_NAME => Function::new_native_with_env(&self.store, WasmerEnvModule { runtime }, use_tbd),
//         //     }
//         // };

//         todo!()
//     }
// }

// impl WasmerScryptoInstance {
//     pub fn send_value(&mut self, value: &ScryptoValue) -> Result<usize, InvokeError> {
//         let slice = &value.raw;
//         let n = slice.len();

//         let result = self
//             .instance
//             .exports
//             .get_function(EXPORT_SCRYPTO_ALLOC)
//             .map_err(|_| InvokeError::MemoryAllocError)?
//             .call(&[Val::I32(n as i32)])
//             .map_err(|_| InvokeError::MemoryAllocError)?;

//         if let Some(Value::I32(ptr)) = result.as_ref().get(0) {
//             let memory = self
//                 .instance
//                 .exports
//                 .get_memory(EXPORT_MEMORY)
//                 .map_err(|_| InvokeError::MemoryAllocError)?;
//             let size = memory.size().bytes().0;
//             if size > (*ptr as usize) && size - (*ptr as usize) >= n {
//                 unsafe {
//                     let dest = memory.data_ptr().add(*ptr as usize);
//                     ptr::copy(slice.as_ptr(), dest, n);
//                 }
//                 return Ok(*ptr as usize);
//             }
//         }

//         Err(InvokeError::MemoryAllocError)
//     }

//     pub fn read_value(&self, ptr: usize) -> Result<ScryptoValue, InvokeError> {
//         let memory = self
//             .instance
//             .exports
//             .get_memory(EXPORT_MEMORY)
//             .map_err(|_| InvokeError::MemoryAccessError)?;
//         let size = memory.size().bytes().0;
//         if size > ptr && size - ptr >= 4 {
//             // read len
//             let mut temp = [0u8; 4];
//             unsafe {
//                 let from = memory.data_ptr();
//                 ptr::copy(from, temp.as_mut_ptr(), 4);
//             }
//             let n = u32::from_le_bytes(temp) as usize;

//             // read value
//             if size - ptr - 4 >= (n as usize) {
//                 // TODO: avoid copying
//                 let mut temp = Vec::with_capacity(n);
//                 unsafe {
//                     let from = memory.data_ptr().add(4);
//                     ptr::copy(from, temp.as_mut_ptr(), n);
//                     temp.set_len(n);
//                 }
//                 return ScryptoValue::from_slice(&temp).map_err(InvokeError::InvalidScryptoValue);
//             }
//         }

//         Err(InvokeError::MemoryAccessError)
//     }
// }

// impl<R: ScryptoRuntime> ScryptoInstance<R> for WasmerScryptoInstance {
//     fn invoke_export(
//         &mut self,
//         export_name: &str,
//         input: &ScryptoValue,
//     ) -> Result<ScryptoValue, InvokeError> {
//         let pointer = self.send_value(input)?;
//         let result = self
//             .instance
//             .exports
//             .get_function(export_name)
//             .map_err(|_| InvokeError::FunctionNotFound)?
//             .call(&[Val::I32(pointer as i32)]);

//         todo!()
//     }

//     fn function_exports(&self) -> Vec<String> {
//         self.instance
//             .exports
//             .iter()
//             .filter(|e| matches!(e.1, Extern::Function(_)))
//             .map(|e| e.0.to_string())
//             .collect()
//     }
// }

// impl WasmerEngine {
//     pub fn new() -> Self {
//         Self {
//             store: Store::default(),
//         }
//     }
// }

// impl ScryptoValidator for WasmerEngine {
//     fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError> {
//         Ok(())
//     }
// }

// impl ScryptoInstrumenter for WasmerEngine {
//     fn instrument(&mut self, code: &[u8]) -> Result<Vec<u8>, InstrumentError> {
//         Ok(code.to_vec())
//     }
// }

// impl <'l, 'r, R: ScryptoRuntime> ScryptoLoader<'l, 'r, WasmerScryptoModule<'l>, WasmerScryptoInstance, R> for WasmerEngine {
//     fn load(&'l mut self, code: &[u8]) -> WasmerScryptoModule<'l> {
//         let module = Module::new(&self.store, code).expect("Failed to parse wasm module");

//         WasmerScryptoModule {
//             module,
//             store: &self.store,
//         }
//     }
// }

use scrypto::rust::format;
use wasmi::*;

/// Radix Engine entrance function index.
pub const ENGINE_FUNCTION_INDEX: usize = 0;
/// Radix Engine entrance function name.
pub const ENGINE_FUNCTION_NAME: &str = "radix_engine";

/// An `env` module resolver defines how symbols in `env` are resolved.
pub struct EnvModuleResolver;

impl ModuleImportResolver for EnvModuleResolver {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        match field_name {
            ENGINE_FUNCTION_NAME => {
                if signature.params() != [ValueType::I32, ValueType::I32, ValueType::I32]
                    || signature.return_type() != Some(ValueType::I32)
                {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(
                    signature.clone(),
                    ENGINE_FUNCTION_INDEX,
                ))
            }
            _ => Err(Error::Instantiation(format!(
                "Export {} not found",
                field_name
            ))),
        }
    }
}

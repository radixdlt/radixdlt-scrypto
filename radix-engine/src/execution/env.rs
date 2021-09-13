use scrypto::rust::format;
use wasmi::*;

/// Kernel entrance function index.
pub const KERNEL_INDEX: usize = 0;
/// Kernel entrance function name.
pub const KERNEL_NAME: &str = "kernel";

/// Decides what symbols are available in the `env` module.
pub struct EnvModuleResolver;

impl ModuleImportResolver for EnvModuleResolver {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        match field_name {
            KERNEL_NAME => {
                if signature.params() != [ValueType::I32, ValueType::I32, ValueType::I32]
                    || signature.return_type() != Some(ValueType::I32)
                {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(signature.clone(), KERNEL_INDEX))
            }
            _ => Err(Error::Instantiation(format!(
                "Export {} not found",
                field_name
            ))),
        }
    }
}

use wasmi::*;

/// Kernel entrance function index.
pub const KERNEL: usize = 0;

/// Decides the symbols available in the `env` module.
pub struct EnvModuleResolver;

impl ModuleImportResolver for EnvModuleResolver {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        match field_name {
            "kernel" => {
                if signature.params() != [ValueType::I32, ValueType::I32, ValueType::I32]
                    || signature.return_type() != Some(ValueType::I32)
                {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(signature.clone(), KERNEL))
            }
            _ => Err(Error::Instantiation(format!(
                "Export {} not found",
                field_name
            ))),
        }
    }
}

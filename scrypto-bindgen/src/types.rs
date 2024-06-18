use radix_common::prelude::*;
use radix_rust::prelude::{hashmap, HashMap};

// This structure describes function argument and return types replacements.
#[derive(Debug)]
pub struct FunctionSignatureReplacementsInput {
    pub blueprint_name: String, // Name of the blueprint, which shall have replaced types
    pub func_name: String,      // Name of the function, which shall have replaced types
    pub replacement_map: FunctionSignatureReplacements,
}

pub type FunctionSignaturesReplacementMap = HashMap<String, FunctionSignatureReplacements>;
pub type BlueprintFunctionSignaturesReplacementMap =
    HashMap<String, FunctionSignaturesReplacementMap>;

#[derive(Debug, Clone)]
pub struct FunctionSignatureReplacements {
    pub arg: HashMap<usize, String>, // Map of argument indexes and their new type names
    pub output: Option<String>,      // Name of the new return type
}

pub fn prepare_replacement_map(
    replacement_vec: &[FunctionSignatureReplacementsInput],
) -> BlueprintFunctionSignaturesReplacementMap {
    let mut blueprint_replacement_map: BlueprintFunctionSignaturesReplacementMap = hashmap!();

    for item in replacement_vec {
        if blueprint_replacement_map
            .get(&item.blueprint_name)
            .is_some()
        {
            let function_map = blueprint_replacement_map
                .get_mut(&item.blueprint_name)
                .unwrap();
            function_map.insert(item.func_name.clone(), item.replacement_map.clone());
        } else {
            let mut function_map = hashmap!();
            function_map.insert(item.func_name.clone(), item.replacement_map.clone());
            blueprint_replacement_map.insert(item.blueprint_name.clone(), function_map);
        };
    }
    blueprint_replacement_map
}

impl FromStr for FunctionSignatureReplacementsInput {
    type Err = String;

    // Get FunctionSignatureReplacementsInput from a string.
    // Syntax:
    //   blueprint_name=<blueprint_name>;func_name=<function_name>;<argument_index>:<type_replacement>;r:<type_replacement>
    // Example:
    //   - replace first argument and return type to FungibleBucket of the function 'new'
    //      blueprint_name=Faucet;func_name=new;0=FungibleBucket;r=FungibleBucket
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut items = input.split(";");

        let mut blueprint_name_items = items
            .next()
            .ok_or("Cannot determine blueprint name")?
            .split("=");

        let blueprint_name = match blueprint_name_items.next() {
            Some(val) if val == "blueprint_name" => blueprint_name_items.next(),
            Some(_) => None,
            None => None,
        }
        .ok_or("blueprint_name not found")?
        .to_string();

        let mut func_name_items = items
            .next()
            .ok_or("Cannot determine function name")?
            .split("=");

        let func_name = match func_name_items.next() {
            Some(val) if val == "func_name" => func_name_items.next(),
            Some(_) => None,
            None => None,
        }
        .ok_or("func_name not found")?
        .to_string();

        let mut arg = hashmap!();
        let mut output = None;

        for item in items {
            let mut s = item.split("=");
            match s.next() {
                Some(val) if val == "r" => {
                    let ty = s.next().ok_or("Return type not available".to_string())?;
                    output = Some(ty.to_string());
                }
                Some(val) => {
                    let idx: usize = val.parse().map_err(|_| "Failed to parse integer")?;
                    let ty = s.next().ok_or("Arg type not available")?;
                    arg.insert(idx, ty.to_string());
                }
                None => Err("Argument index or return type not available")?,
            }
        }

        Ok(Self {
            blueprint_name,
            func_name,
            replacement_map: FunctionSignatureReplacements { arg, output },
        })
    }
}

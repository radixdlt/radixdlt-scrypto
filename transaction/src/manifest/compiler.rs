use radix_engine_interface::address::Bech32Decoder;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::crypto::hash;

use sbor::rust::collections::HashMap;

use crate::manifest::*;
use crate::model::TransactionManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

pub fn compile(
    s: &str,
    network: &NetworkDefinition,
    blobs: Vec<Vec<u8>>,
) -> Result<TransactionManifest, CompileError> {
    let bech32_decoder = Bech32Decoder::new(network);

    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let instructions = parser::Parser::new(tokens)
        .parse_manifest()
        .map_err(CompileError::ParserError)?;
    let mut blobs_by_hash = HashMap::new();
    for blob in blobs {
        blobs_by_hash.insert(hash(&blob), blob);
    }
    generator::generate_manifest(&instructions, &bech32_decoder, blobs_by_hash)
        .map_err(CompileError::GeneratorError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Instruction;
    use radix_engine_interface::api::types::{
        NativeFunctionIdent, ResourceManagerFunction, ScryptoMethodIdent, ScryptoReceiver,
    };
    use radix_engine_interface::core::Expression;
    use radix_engine_interface::crypto::Blob;
    use radix_engine_interface::data::*;
    use radix_engine_interface::math::{Decimal, PreciseDecimal};
    use radix_engine_interface::model::*;
    use sbor::rust::collections::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn test_compile() {
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());
        let manifest = include_str!("../../examples/complex.rtm");
        let blobs = vec![
            include_bytes!("../../examples/code.blob").to_vec(),
            include_bytes!("../../examples/abi.blob").to_vec(),
        ];
        let code_hash = hash(&blobs[0]);
        let abi_hash = hash(&blobs[1]);

        let component1 = bech32_decoder
            .validate_and_decode_component_address(
                "account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064",
            )
            .unwrap();
        let component2 = bech32_decoder
            .validate_and_decode_component_address(
                "component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum",
            )
            .unwrap();

        assert_eq!(
            crate::manifest::compile(manifest, &NetworkDefinition::simulator(), blobs)
                .unwrap()
                .instructions,
            vec![
                Instruction::CallMethod {
                    method_ident: ScryptoMethodIdent {
                        receiver: ScryptoReceiver::Global(component1),
                        method_name: "withdraw_by_amount".to_string(),
                    },
                    args: args!(
                        Decimal::from(5u32),
                        bech32_decoder
                            .validate_and_decode_resource_address(
                                "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"
                            )
                            .unwrap()
                    )
                },
                Instruction::TakeFromWorktopByAmount {
                    amount: Decimal::from(2),
                    resource_address: bech32_decoder
                        .validate_and_decode_resource_address(
                            "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"
                        )
                        .unwrap(),
                },
                Instruction::CallMethod {
                    method_ident: ScryptoMethodIdent {
                        receiver: ScryptoReceiver::Global(component2),
                        method_name: "buy_gumball".to_string(),
                    },
                    args: args!(Bucket(512))
                },
                Instruction::AssertWorktopContainsByAmount {
                    amount: Decimal::from(3),
                    resource_address: bech32_decoder
                        .validate_and_decode_resource_address(
                            "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"
                        )
                        .unwrap(),
                },
                Instruction::AssertWorktopContains {
                    resource_address: bech32_decoder
                        .validate_and_decode_resource_address(
                            "resource_sim1qzhdk7tq68u8msj38r6v6yqa5myc64ejx3ud20zlh9gseqtux6"
                        )
                        .unwrap(),
                },
                Instruction::TakeFromWorktop {
                    resource_address: bech32_decoder
                        .validate_and_decode_resource_address(
                            "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"
                        )
                        .unwrap(),
                },
                Instruction::CreateProofFromBucket { bucket_id: 513 },
                Instruction::CloneProof { proof_id: 514 },
                Instruction::DropProof { proof_id: 514 },
                Instruction::DropProof { proof_id: 515 },
                Instruction::CallMethod {
                    method_ident: ScryptoMethodIdent {
                        receiver: ScryptoReceiver::Global(component1),
                        method_name: "create_proof_by_amount".to_string(),
                    },
                    args: args!(
                        Decimal::from(5u32),
                        bech32_decoder
                            .validate_and_decode_resource_address(
                                "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"
                            )
                            .unwrap()
                    )
                },
                Instruction::PopFromAuthZone,
                Instruction::DropProof { proof_id: 516 },
                Instruction::ReturnToWorktop { bucket_id: 513 },
                Instruction::TakeFromWorktopByIds {
                    ids: BTreeSet::from([
                        NonFungibleId::from_str("5c200721031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f").unwrap(),
                        NonFungibleId::from_str("5c200721031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f").unwrap(),
                    ]),
                    resource_address: bech32_decoder
                        .validate_and_decode_resource_address(
                            "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"
                        )
                        .unwrap()
                },
                Instruction::CallNativeFunction {
                    function_ident: NativeFunctionIdent {
                        blueprint_name: "ResourceManager".to_owned(),
                        function_name: ResourceManagerFunction::CreateNoOwner.to_string(),
                    },
                    args: args!(
                        ResourceType::Fungible { divisibility: 0 },
                        HashMap::<String, String>::new(),
                        HashMap::<ResourceMethodAuthKey, (AccessRule, Mutability)>::new(),
                        Some(MintParams::Fungible {
                            amount: "1.0".into()
                        })
                    ),
                },
                Instruction::CallMethod {
                    method_ident: ScryptoMethodIdent {
                        receiver: ScryptoReceiver::Global(component1),
                        method_name: "deposit_batch".to_string(),
                    },
                    args: args!(Expression("ENTIRE_WORKTOP".to_owned()))
                },
                Instruction::DropAllProofs,
                Instruction::CallMethod {
                    method_ident: ScryptoMethodIdent {
                        receiver: ScryptoReceiver::Global(component2),
                        method_name: "complicated_method".to_string(),
                    },
                    args: args!(Decimal::from(1u32), PreciseDecimal::from(2u32))
                },
                Instruction::PublishPackage {
                    code: Blob(code_hash),
                    abi: Blob(abi_hash),
                },
            ]
        );
    }

    #[test]
    fn test_call_method() {
        let manifest = include_str!("../../examples/call_method.rtm");
        crate::manifest::compile(manifest, &NetworkDefinition::simulator(), Vec::new()).unwrap();
    }

    #[test]
    fn test_call_function() {
        let manifest = include_str!("../../examples/call_function.rtm");
        crate::manifest::compile(manifest, &NetworkDefinition::simulator(), Vec::new()).unwrap();
    }
}

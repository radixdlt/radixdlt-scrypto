use inferno::flamegraph;
use scrypto_test::prelude::*;
use std::path::{Path, PathBuf};

#[test]
fn x() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let detailed_execution_cost_breakdown = env.with_costing_module_enabled(|env| {
        let _ = env.with_kernel_trace_module_enabled(|env| {
            env.call_method_typed::<_, _, Bucket>(FAUCET, "free", &())
        })?;

        Ok::<_, RuntimeError>(env.with_kernel(|kernel| {
            kernel
                .kernel_callback()
                .modules
                .costing()
                .unwrap()
                .cost_breakdown
                .as_ref()
                .unwrap()
                .detailed_execution_cost_breakdown
                .clone()
        }))
    })?;

    create_flamegraph_of_execution_breakdown(
        &detailed_execution_cost_breakdown,
        PathBuf::from("file.svg").as_path(),
        "Faucet Example",
    );

    Ok(())
}

fn create_flamegraph_of_execution_breakdown(
    detailed_execution_cost_breakdown: &[(usize, ExecutionCostBreakdownItem)],
    path: &Path,
    title: impl AsRef<str>,
) {
    // The options to use when constructing the flamechart.
    let mut opts = flamegraph::Options::default();
    title.as_ref().clone_into(&mut opts.title);

    // Transforming the detailed execution cost breakdown into a string understood by the flamegraph
    // library.
    let flamegraph_string = transform_detailed_execution_breakdown_into_flamegraph_string(
        detailed_execution_cost_breakdown,
    );

    // Writing the flamegraph string to a temporary file since its required by the flamegraph lib to
    // have a path.
    let result = {
        let tempdir = tempfile::tempdir().unwrap();
        let tempfile = tempdir.path().join("file.txt");
        std::fs::write(&tempfile, flamegraph_string).unwrap();

        let mut result = std::io::Cursor::new(Vec::new());
        flamegraph::from_files(&mut opts, &[tempfile], &mut result).unwrap();

        result.set_position(0);
        result.into_inner()
    };

    std::fs::write(path, result).unwrap();
}

fn transform_detailed_execution_breakdown_into_flamegraph_string(
    detailed_execution_cost_breakdown: &[(usize, ExecutionCostBreakdownItem)],
) -> String {
    let network_definition = NetworkDefinition::mainnet();
    let address_bech32m_encoder = AddressBech32Encoder::new(&network_definition);

    let mut lines = Vec::<String>::new();
    let mut path_stack = vec!["execution".to_owned()];
    for (index, (_, execution_item)) in detailed_execution_cost_breakdown.iter().enumerate() {
        // Constructing the full path
        match execution_item {
            ExecutionCostBreakdownItem::Invocation(invocation) => {
                let actor_string = match invocation.call_frame_data {
                    Actor::Root => "root".to_owned(),
                    Actor::Method(MethodActor {
                        node_id,
                        ref ident,
                        ref object_info,
                        ..
                    }) => {
                        format!(
                            "Method <{}>::{}::{}",
                            address_bech32m_encoder.encode(node_id.as_bytes()).unwrap(),
                            object_info.blueprint_info.blueprint_id.blueprint_name,
                            ident
                        )
                    }
                    Actor::Function(FunctionActor {
                        ref blueprint_id,
                        ref ident,
                        ..
                    }) => {
                        format!(
                            "Function <{}>::{}::{}",
                            address_bech32m_encoder
                                .encode(blueprint_id.package_address.as_bytes())
                                .unwrap(),
                            blueprint_id.blueprint_name,
                            ident
                        )
                    }
                    Actor::BlueprintHook(BlueprintHookActor {
                        hook,
                        ref blueprint_id,
                        ..
                    }) => {
                        format!(
                            "Blueprint Hook <{}>::{}::{:?}",
                            address_bech32m_encoder
                                .encode(blueprint_id.package_address.as_bytes())
                                .unwrap(),
                            blueprint_id.blueprint_name,
                            hook
                        )
                    }
                };
                path_stack.push(format!("Invocation: {actor_string} ({index})"))
            }
            ExecutionCostBreakdownItem::InvocationComplete => {
                path_stack.pop();
            }
            ExecutionCostBreakdownItem::Execution {
                simple_name,
                cost_units,
                ..
            } => {
                lines.push(format!(
                    "{};{} {}",
                    path_stack.join(";"),
                    simple_name,
                    cost_units
                ));
            }
        }
    }

    lines.join("\n")
}

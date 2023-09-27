mod package_loader;

use scrypto_test::prelude::*;
use scrypto_unit::*;

#[test]
fn reconcile_blueprint_auth_setup_overview() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        let auth_configs = test_runner.get_blueprint_auth_config(&package_address);
        for (blueprint_key, auth_config) in auth_configs {
            println!("\n\n{}", "=".repeat(30));
            println!("{:?}", blueprint_key.blueprint);
            println!("{}", "=".repeat(30));
            match auth_config.function_auth {
                FunctionAuth::AllowAll => println!("AllowAll"),
                FunctionAuth::AccessRules(x) => {
                    println!("Based on access rule");
                    for (k, v) in x {
                        println!("{:?} => {:?}", k, v);
                    }
                }
                FunctionAuth::RootOnly => println!("RootOnly"),
            }
            println!("{}", "-".repeat(30));
            match auth_config.method_auth {
                MethodAuthTemplate::AllowAll => println!("AllowAll"),
                MethodAuthTemplate::StaticRoleDefinition(x) => {
                    match x.roles {
                        RoleSpecification::Normal(roles) => {
                            println!("Use normal roles");
                            for (k, v) in roles {
                                println!("{:?}, admin by: {:?}", k, v);
                            }
                        }
                        RoleSpecification::UseOuter => println!("Use outer object roles"),
                    }
                    for (k, v) in x.methods {
                        println!("{:?}, access by: {:?}", k, v);
                    }
                }
            }
        }
    }

    let x = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .set_role(
                GENESIS_HELPER,
                ModuleId::Main,
                RoleKey {
                    key: "system".into(),
                },
                rule!(allow_all),
            )
            .build(),
        vec![],
    );
    println!("{:?}", x);

    let x = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .set_role(
                CONSENSUS_MANAGER,
                ModuleId::Main,
                RoleKey {
                    key: "validator".into(),
                },
                rule!(allow_all),
            )
            .build(),
        vec![],
    );
    println!("{:?}", x);

    println!(
        "ROLE RULES OF GENESIS HELPER: {:?}",
        test_runner.get_role_assignment(&GENESIS_HELPER.into())
    );
    println!(
        "ROLE RULES OF CONS: {:?}",
        test_runner.get_role_assignment(&CONSENSUS_MANAGER.into())
    );
}

#[test]
fn reconcile_blueprint_auth_setup_per_function() {
    let test_runner = TestRunnerBuilder::new().build();
    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        let auth_configs = test_runner.get_blueprint_auth_config(&package_address);
        let blueprint_defs = test_runner.get_blueprint_definitions(&package_address);
        for (blueprint_key, blueprint_def) in blueprint_defs {
            println!("\n\n{}", "=".repeat(30));
            println!("{:?}", blueprint_key.blueprint);
            println!("{}", "=".repeat(30));

            for f in &blueprint_def.interface.functions {
                if f.1.receiver.is_none() {
                    let rule = match &auth_configs.get(&blueprint_key).unwrap().function_auth {
                        FunctionAuth::AllowAll => "PUBLIC".to_owned(),
                        FunctionAuth::AccessRules(x) => format!("{:?}", x.get(f.0).unwrap()),
                        FunctionAuth::RootOnly => "ROOT_ONLY".to_owned(),
                    };
                    println!("{} => {}", f.0, rule);
                }
            }
            println!("{}", "-".repeat(30));

            for f in &blueprint_def.interface.functions {
                if f.1.receiver.is_some() {
                    let rule = match &auth_configs.get(&blueprint_key).unwrap().method_auth {
                        MethodAuthTemplate::AllowAll => "PUBLIC".to_owned(),
                        MethodAuthTemplate::StaticRoleDefinition(x) => {
                            match x.methods.get(&MethodKey { ident: f.0.clone() }).unwrap() {
                                MethodAccessibility::Public => "PUBLIC".to_owned(),
                                MethodAccessibility::OuterObjectOnly => {
                                    "OUTER_OBJECT_ONLY".to_owned()
                                }
                                MethodAccessibility::RoleProtected(role_list) => {
                                    let mut buf = IndexSet::<String>::new();
                                    for role in &role_list.list {
                                        match &x.roles {
                                            RoleSpecification::Normal(roles) => {
                                                buf.insert(role.key.clone());
                                                if !role.key.eq("_owner_") {
                                                    for v in &roles.get(role).unwrap().list {
                                                        buf.insert(v.key.clone());
                                                    }
                                                }
                                            }
                                            RoleSpecification::UseOuter => {
                                                buf.insert(format!("OUTER::{}", role.key));
                                            }
                                        }
                                    }
                                    buf.into_iter().collect::<Vec<String>>().join(", ")
                                }
                                MethodAccessibility::OwnPackageOnly => "PACKAGE_ONLY".to_owned(),
                            }
                        }
                    };
                    println!("{} => {}", f.0, rule);
                }
            }
        }
    }
}

#[test]
fn reconcile_component_auth_setup_overview() {
    let test_runner = TestRunnerBuilder::new().build();
    let component_addresses = test_runner.find_all_globals();
    for component_address in component_addresses {
        println!("\n\n{}", "=".repeat(30));
        println!(
            "{:?}, {:?}",
            component_address,
            component_address.as_node_id().entity_type()
        );
        println!("{}", "=".repeat(30));

        for (role, rule) in test_runner.get_role_assignment(&component_address.into()) {
            println!("{:?} => {:?}", role, rule);
        }
    }
}

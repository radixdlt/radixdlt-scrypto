#[cfg(test)]
mod tests {
    use radix_common::prelude::*;
    use radix_engine::utils::ExtractSchemaError;
    use radix_engine::vm::wasm::PrepareError;
    use radix_engine_interface::types::Level;
    use scrypto_compiler::*;
    use std::process::Command;
    use std::{env, path::PathBuf, process::Stdio};
    use tempdir::TempDir;

    // helper function
    fn prepare() -> (PathBuf, TempDir) {
        let mut test_assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_assets_path.extend(["tests", "assets", "scenario_1", "blueprint", "Cargo.toml"]);
        (
            test_assets_path,
            TempDir::new("scrypto-compiler-test").unwrap(),
        )
    }

    #[test]
    fn test_compilation() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        let build_artifacts = status.unwrap();

        assert_eq!(build_artifacts.len(), 1);
        assert!(build_artifacts[0].wasm.path.exists());
        assert!(build_artifacts[0].package_definition.path.exists());

        assert!(
            std::fs::metadata(&build_artifacts[0].wasm.path)
                .unwrap()
                .len()
                > 0,
            "Wasm file should not be empty."
        );
        assert!(
            std::fs::metadata(&build_artifacts[0].package_definition.path)
                .unwrap()
                .len()
                > 7,
            "Package definition file should not be empty, so should be longer than 7 bytes."
        ); // 7 bytes is for empty rpd file

        let mut target_path = target_directory.path().to_path_buf();
        target_path.extend(["wasm32-unknown-unknown", "release", "test_blueprint.wasm"]);
        assert_eq!(build_artifacts[0].wasm.path, target_path);
        assert_eq!(
            build_artifacts[0].package_definition.path,
            target_path.with_extension("rpd")
        );
    }

    #[test]
    fn test_compilation_in_current_dir() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        let mut package_directory = blueprint_manifest_path.clone();
        package_directory.pop(); // Remove Cargo.toml from path
        std::env::set_current_dir(package_directory).unwrap();

        // Act
        let status = ScryptoCompiler::builder()
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_env_var() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .env("TEST", EnvironmentVariableAction::Set(String::from("1 1")))
            .env("OTHER", EnvironmentVariableAction::Unset)
            .env(
                "RUSTFLAGS",
                EnvironmentVariableAction::Set(String::from("-C opt-level=3")),
            )
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_feature() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .feature("feature-1")
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_feature_and_loglevel() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .feature("feature-1")
            .log_level(Level::Warn)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_fails_with_non_existing_feature() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .feature("feature-2")
            .compile();

        // Assert
        assert!(match status {
            Err(ScryptoCompilerError::CargoBuildFailure(exit_status)) =>
                exit_status.code().unwrap() == 101,
            _ => false,
        });
    }

    #[test]
    fn test_compilation_workspace() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        let mut workspace_directory = blueprint_manifest_path.clone();
        workspace_directory.pop(); // Remove Cargo.toml from path
        workspace_directory.pop(); // Remove blueprint folder
        workspace_directory.push("Cargo.toml"); // Put workspace Cargo.toml file

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(workspace_directory)
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        let build_artifacts = status.unwrap();

        // workspace contains only 3 packages with defined scrypto metadata
        assert_eq!(build_artifacts.len(), 3);

        let names = [
            "test_blueprint.wasm",
            "test_blueprint_2.wasm",
            "test_blueprint_3.wasm",
        ];
        for i in 0..names.len() {
            assert!(build_artifacts[i].wasm.path.exists());
            assert!(build_artifacts[i].package_definition.path.exists());

            assert!(
                std::fs::metadata(&build_artifacts[i].wasm.path)
                    .unwrap()
                    .len()
                    > 0,
                "Wasm file should not be empty."
            );
            assert!(
                std::fs::metadata(&build_artifacts[i].package_definition.path)
                    .unwrap()
                    .len()
                    > 7,
                "Package definition file should not be empty, so should be longer than 7 bytes."
            ); // 7 bytes is for empty rpd file

            let mut target_path = target_directory.path().to_path_buf();
            target_path.extend(["wasm32-unknown-unknown", "release", names[i]]);
            assert_eq!(build_artifacts[i].wasm.path, target_path);
            assert_eq!(
                build_artifacts[i].package_definition.path,
                target_path.with_extension("rpd")
            );
        }

        // test_blueprint_4 package should not be compiled because it doesn't define [profile.metadata.scrypto] metadata.
        assert!(!build_artifacts[0]
            .wasm
            .path
            .with_file_name("test_blueprint_4.wasm")
            .exists());
    }

    #[test]
    fn test_compilation_workspace_in_current_dir() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        let mut workspace_directory = blueprint_manifest_path.clone();
        workspace_directory.pop(); // Remove Cargo.toml from path
        workspace_directory.pop(); // Remove blueprint folder
        std::env::set_current_dir(workspace_directory).unwrap();

        // Act
        let status = ScryptoCompiler::builder()
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        let build_artifacts = status.unwrap();

        // workspace contains only 3 packages with defined scrypto metadata
        assert_eq!(build_artifacts.len(), 3);
    }

    #[test]
    fn test_compilation_workspace_with_package() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        let mut workspace_directory = blueprint_manifest_path.clone();
        workspace_directory.pop(); // Remove Cargo.toml from path
        workspace_directory.pop(); // Remove blueprint folder
        workspace_directory.push("Cargo.toml"); // Put workspace Cargo.toml file

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(workspace_directory)
            .target_directory(target_directory.path())
            .package("test_blueprint_2")
            .package("test_blueprint_3")
            .package("test_blueprint_4") // it is possible to specify explicitly package without scrypto metadata
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        let build_artifacts = status.unwrap();

        assert_eq!(build_artifacts.len(), 3);

        let names = [
            "test_blueprint_2.wasm",
            "test_blueprint_3.wasm",
            "test_blueprint_4.wasm",
        ];
        for i in 0..names.len() {
            assert!(build_artifacts[i].wasm.path.exists());
            assert!(build_artifacts[i].package_definition.path.exists());
        }

        // test_blueprint_1 package should not be compiled
        assert!(!build_artifacts[0]
            .wasm
            .path
            .with_file_name("test_blueprint_1.wasm")
            .exists());
    }

    #[test]
    fn test_compilation_workspace_with_non_existing_package() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        let mut workspace_directory = blueprint_manifest_path.clone();
        workspace_directory.pop(); // Remove Cargo.toml from path
        workspace_directory.pop(); // Remove blueprint folder
        workspace_directory.push("Cargo.toml"); // Put workspace Cargo.toml file

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(workspace_directory)
            .target_directory(target_directory.path())
            .package("test_blueprint_2")
            .package("non_existing_package")
            .package("test_blueprint_3")
            .compile();

        // Assert
        assert!(match status {
            Err(ScryptoCompilerError::CargoWrongPackageId(package)) =>
                package == "non_existing_package",
            _ => false,
        });
    }

    #[test]
    fn test_compilation_workspace_without_scrypto_package() {
        // Arrange
        let mut blueprint_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        blueprint_manifest_path.extend([
            "tests",
            "assets",
            "scenario_2",
            "some_project",
            "Cargo.toml",
        ]);
        let target_directory = TempDir::new("scrypto-compiler-test").unwrap();

        let mut workspace_directory = blueprint_manifest_path.clone();
        workspace_directory.pop(); // Remove Cargo.toml from path
        workspace_directory.pop(); // Remove project folder
        workspace_directory.push("Cargo.toml"); // Put workspace Cargo.toml file

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(workspace_directory)
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert_matches!(status, Err(ScryptoCompilerError::NothingToCompile));
    }

    #[test]
    fn test_compilation_profile_release() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Release)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_debug() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Debug)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_test() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Test)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_bench() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Bench)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_custom() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Custom(String::from("custom")))
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_without_wasm_optimisations() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .optimize_with_wasm_opt(None)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_stdio() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .compile_with_stdio(Some(Stdio::piped()), Some(Stdio::null()), None);

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_wasm_reference_types_disabled() {
        // Arrange
        let mut blueprint_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        blueprint_manifest_path.extend(["tests", "assets", "call_indirect", "Cargo.toml"]);

        // Act
        // ScryptoCompiler compiles WASM by default with reference-types disabled.
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_wasm_reference_types_enabled() {
        // Arrange
        let mut blueprint_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        blueprint_manifest_path.extend(["tests", "assets", "call_indirect", "Cargo.toml"]);

        // Check clang/LLVM version
        let clang_version = Command::new("clang").arg("--version").output().unwrap();

        // clang --version exemplary output
        // Ubuntu clang version 17.0.6 (++20231209124227+6009708b4367-1~exp1~20231209124336.77)
        // Target: x86_64-pc-linux-gnu
        // Thread model: posix
        // InstalledDir: /usr/lib/llvm-17/bin
        let clang_version = String::from_utf8_lossy(&clang_version.stdout);
        let mut idx = clang_version
            .find("clang version")
            .expect("Failed to get clang version");
        idx += "clang version ".len();
        let version = &clang_version[idx..]
            .split_whitespace()
            .next()
            .expect("Failed to get version");
        let major_version = version
            .split(".")
            .next()
            .expect("Failed to get major version");
        let major_version: u8 = major_version.parse().unwrap();

        let action = if major_version >= 19 {
            // Since LLVM 19 reference-types are enabled by default, no dedicated CFLAGS needed.
            // Unset TARGET_CFLAGS to build with default WASM features.
            EnvironmentVariableAction::Unset
        } else {
            // In previous versions reference-types must be enabled explicitly.
            EnvironmentVariableAction::Set("-mreference-types".to_string())
        };
        // Act
        let status = ScryptoCompiler::builder()
            .env("TARGET_CFLAGS", action)
            .manifest_path(blueprint_manifest_path)
            .compile();

        // Assert
        // Error is expected here because Radix Engine expects WASM with reference-types disabled.
        // See `call_indirect.c` for more details.
        assert_matches!(
            status.unwrap_err(),
            ScryptoCompilerError::SchemaExtractionError(
                ExtractSchemaError::InvalidWasm(PrepareError::ValidationError(msg))) if msg.contains("reference-types not enabled: zero byte expected")
        )
    }
}

"""
This script finds all of the Cargo.toml files in the script directory, finds their workspace cargo
manifest, and runs a `cargo check` on the workspace manifest. Essentially this is a check on all of
the crates we have regardless of what workspace they're in.
"""

from typing import TypeVar
import subprocess
import json
import os


def workspace_path(manifest_path: str) -> str:
    process: subprocess.Popen = subprocess.Popen(
        [
            "cargo",
            "metadata",
            "--manifest-path",
            manifest_path,
            "--format-version",
            "1",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    stdout: bytes
    stderr: bytes
    stdout, stderr = process.communicate()

    try:
        return json.loads(stdout)["workspace_root"]
    except:
        raise Exception(manifest_path, stdout.decode(), stderr.decode())


T = TypeVar("T")
def unique(l: list[T]) -> list[T]:
    return list(set(l))


def main() -> None:
    # Find all of the `Cargo.toml` files recursively in the script directory.
    workspace_manifest_paths: list[str] = sorted(
        unique(
            [
                os.path.join(
                    workspace_path(os.path.join(dirpath, filename)), "Cargo.toml"
                )
                for dirpath, _, filenames in os.walk(
                    os.path.dirname(os.path.realpath(__file__))
                )
                for filename in filenames
                if filename == "Cargo.toml"
            ]
        )
    )

    # Running cargo check on the manifest.
    for workspace_manifest_file_path in workspace_manifest_paths:
        if "dec_macros" in workspace_manifest_file_path:
            continue

        process: subprocess.Popen = subprocess.Popen(
            [
                "cargo",
                "check",
                "--all-targets",
                "--manifest-path",
                workspace_manifest_file_path,
            ],
            stderr=subprocess.PIPE,
            stdout=subprocess.PIPE,
        )

        stderr: bytes
        _, stderr = process.communicate()

        emoji: str = "❌" if process.returncode != 0 else "✅"
        print(f"{emoji} {workspace_manifest_file_path}")

        if process.returncode != 0:
            print(stderr.decode())
            raise Exception("Check failed on one of the workspaces")


if __name__ == "__main__":
    main()
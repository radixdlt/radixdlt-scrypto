import re
import os
import subprocess
from typing import cast, BinaryIO, TypedDict


SCRIPT_PATH: str = os.path.dirname(os.path.realpath(__file__))


def main() -> None:
    # The path to the workspace manifest file
    workspace_manifest_file: str = os.path.abspath(
        os.path.join(SCRIPT_PATH, "Cargo.toml")
    )

    # Listing all of the Rust Analyzer autocomplete tests to run all of them.
    list_tests_output: bytes = cast(
        BinaryIO,
        subprocess.Popen(
            [
                "cargo",
                "test",
                "--manifest-path",
                workspace_manifest_file,
                "--package",
                "rust-analyzer-tests",
                "--test",
                "autocomplete",
                "--",
                "--list",
                "--format=terse",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        ).stdout,
    ).read()

    # Extract all of the test names from the string.
    test_names: list[str] = re.findall(r"(.*)?: test", list_tests_output.decode())

    # Run each test and record the time the autocompletion took.
    test_results: list[tuple[str, int]] = []
    for test_name in test_names:
        # Run the test
        test_run_output: bytes = cast(
            BinaryIO,
            subprocess.Popen(
                [
                    "cargo",
                    "test",
                    "--release",
                    "--manifest-path",
                    workspace_manifest_file,
                    "--package",
                    "rust-analyzer-tests",
                    "--test",
                    "autocomplete",
                    "--",
                    test_name,
                    "--exact",
                    "--nocapture",
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            ).stdout,
        ).read()

        # Find the amount of time it took to run the benchmark
        duration: int = int(
            re.findall(r"Autocomplete took: (\d+)ms", test_run_output.decode())[0]
        )

        # Insert to the list
        test_results.append((test_name, duration))

    # Output the results as a markdown table to stdout
    markdown_table: str = "| Test Name | Autocomplete (ms) |\n| -- | -- |\n"
    for (test_name, duration) in test_results:
        markdown_table += f"| {test_name} | {duration} |\n"
    print(markdown_table)


if __name__ == "__main__":
    main()

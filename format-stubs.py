from typing import Callable
import re
import os

SCRIPT_DIRECTORY: str = os.path.dirname(os.path.abspath(__file__))
STUBS_FILE_PATH: str = os.path.join(
    SCRIPT_DIRECTORY, "scrypto", "src", "component", "stubs.rs"
)
REGEX_PATTERN: str = r'extern_blueprint_internal!\s*{\s*(PackageAddress.*]\))\s*,\s*(\w+)\s*,\s("[\w\d_]*")\s*,\s("[\w\d_]*")\s*,\s("[\w\d_]*")\s*,\s*(\w+)\s*({.*}\s*),\s*({.*?})'

def main() -> None:
    # Reading the contents of the stubs file and splitting it by line.
    with open(STUBS_FILE_PATH, "r") as file:
        content: str = file.read()
        lines: list[str] = content.split("\n")

    # Filtering the lines we've read down to the files we believe will require replacement. This is
    # any line that contains `extern_blueprint_internal!`
    lines_requiring_replacement: list[str] = list(
        filter(lambda line: "extern_blueprint_internal!" in line, lines)
    )

    # Store the line and its replacement in a KV map where the key is the original line and value is
    # the replacement.
    line_and_replacement: dict[str, str] = {
        line: produce_replacement(line)
        for line
        in lines_requiring_replacement
    }
    
    # Open the file and write the replacements to it
    for (old, new) in line_and_replacement.items():
        content: str = content.replace(old, new)
    with open(STUBS_FILE_PATH, 'w') as file:
        file.write(content)

def produce_replacement(line: str) -> str:
    # Extracting everything out of the passed extern_blueprint_internal line.
    found: list[str] = re.findall(REGEX_PATTERN, line)[0]
    package_address: str = found[0]
    blueprint_ident: str = found[1]
    blueprint_name: str = found[2]
    owned_name: str = found[3]
    global_name: str = found[4]
    functions_name: str = found[5]
    functions: str = found[6]
    methods: str = found[7]

    # Formatting the package address through rustfmt.
    sep: str = "    "
    add_sep: Callable[[str], str] = lambda line: sep + line
    formatted_package_address: str = "\n".join(rustfmt(f'fn x() {{ {package_address} }}').split("\n")[1:-2])
    formatted_functions: str = "\n".join(map(add_sep, rustfmt(f"trait x {functions.replace('-> ()', '')}").split("\n")[1:-2]))
    formatted_methods: str = "\n".join(map(add_sep, rustfmt(f"trait x {methods.replace('-> ()', '')}").split("\n")[1:-2]))

    replacement: str = f"extern_blueprint_internal! {{\n{formatted_package_address},\n{sep}{blueprint_ident},\n{sep}{blueprint_name},\n{sep}{owned_name},\n{sep}{global_name},\n{sep}{functions_name} {{\n{formatted_functions}\n{sep}}},\n{sep}{{\n{formatted_methods}\n{sep}}}\n}}"
    return replacement

def rustfmt(string: str) -> str:
    return os.popen(f'echo "{string}" | rustfmt').read()

if __name__ == "__main__":
    main()

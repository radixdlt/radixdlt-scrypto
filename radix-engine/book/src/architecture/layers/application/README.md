# Application Layer

Applications in Radix Engine are deployed through a format called a Package.
Packages consist of zero or more blueprints each of which are uniquely identified
by string name within a package. A blueprint is globally identifiable by
`<package_address> + <blueprint_name>`.

Each blueprint is defined by its *Blueprint Definition* which includes information such as the function
definition and state schemas of the Blueprint. Methods/Functions are mapped either to exported WASM
functions in a provided WASM binary or to native binary.

## Package Deployment

Deployment of new packages are done through invoking the `publish_wasm` or `publish_wasm_advanced` function.


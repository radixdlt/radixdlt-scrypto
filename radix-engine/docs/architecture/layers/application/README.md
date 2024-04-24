# Application Layer

Applications in Radix Engine are responsible for defining two things:
1. New Blueprint Definitions. This includes static information about blueprints such as the schema of a blueprint.
2. Associated Logic for each Blueprint function/method. This may be described as WASM bytecode or Native binary code (though native code can only be used by native packages).

These are bundled up in a format called a Package.

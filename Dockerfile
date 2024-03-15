FROM rust:slim-bullseye as base-image

RUN apt update && apt install -y \
    cmake=3.18.4-2+deb11u1 \
    clang=1:11.0-51+nmu5 \
    build-essential=12.9 \
    llvm=1:11.0-51+nmu5

FROM base-image as builder

# Copy scrypto and radix-engine library crates
ADD assets /app/assets
ADD blueprint-schema-init /app/blueprint-schema-init
ADD Cargo.toml /app/Cargo.toml
ADD radix-native-sdk /app/radix-native-sdk
ADD radix-engine /app/radix-engine
ADD radix-engine-common /app/radix-engine-common
ADD radix-engine-common-macros /app/radix-engine-common-macros
ADD radix-engine-derive /app/radix-engine-derive
ADD radix-engine-interface /app/radix-engine-interface
ADD radix-engine-profiling /app/radix-engine-profiling
ADD radix-engine-profiling-macros /app/radix-engine-profiling-macros
ADD radix-rust app/radix-rust
ADD radix-substate-store-impls /app/radix-substate-store-impls
ADD radix-substate-store-interface /app/radix-substate-store-interface
ADD radix-substate-store-queries /app/radix-substate-store-queries
ADD sbor /app/sbor
ADD sbor-derive /app/sbor-derive
ADD sbor-derive-common /app/sbor-derive-common
ADD transaction /app/transaction

# Copy simulator binary crate
ADD simulator /app/simulator

WORKDIR /app

RUN cargo install --path ./simulator

FROM base-image
COPY --from=builder /app/simulator/target/release/scrypto /usr/local/bin/scrypto
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src

ENTRYPOINT ["scrypto", "build", "--path", "/src"]

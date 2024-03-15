FROM rust:slim-bullseye as base-image

RUN apt update && apt install -y \
    cmake=3.18.4-2+deb11u1 \
    clang=1:11.0-51+nmu5 \
    build-essential=12.9 \
    llvm=1:11.0-51+nmu5

FROM base-image as builder

# Copy scrypto and radix-engine library crates
ADD Cargo.toml /app/Cargo.toml
ADD assets /app/assets
ADD blueprint-schema-init /app/blueprint-schema-init
ADD native-sdk /app/native-sdk
ADD radix-engine /app/radix-engine
ADD radix-engine-common /app/radix-engine-common
ADD radix-engine-common-macros /app/radix-engine-common-macros
ADD radix-engine-derive /app/radix-engine-derive
ADD radix-engine-interface /app/radix-engine-interface
ADD radix-engine-profiling /app/radix-engine-profiling
ADD radix-engine-profiling-macros /app/radix-engine-profiling-macros
ADD radix-rust app/radix-rust
ADD sbor /app/sbor
ADD sbor-derive /app/sbor-derive
ADD sbor-derive-common /app/sbor-derive-common
ADD substate-store-impls /app/substate-store-impls
ADD substate-store-interface /app/substate-store-interface
ADD substate-store-queries /app/substate-store-queries
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

# Below version includes rust 1.79.0
ARG RUST_IMAGE_VERSION=@sha256:b514dde8cc7bbada14d33981ae4b13521e92261e37c50547e6dcd8d8c8795f42
# If you want to use latest version then uncomment next line
# ARG RUST_IMAGE_VERSION=:slim-bullseye
# Alternatively you can build docker with argument: --build-arg="RUST_IMAGE_VERSION=:slim-bullseye"

FROM rust${RUST_IMAGE_VERSION} as base-image

RUN apt update && apt install -y \
    cmake=3.18.4-2+deb11u1 \
    clang=1:11.0-51+nmu5 \
    build-essential=12.9 \
    llvm=1:11.0-51+nmu5

FROM base-image as builder

# Copy library crates
ADD Cargo.toml /app/Cargo.toml
ADD radix-blueprint-schema-init /app/radix-blueprint-schema-init
ADD radix-common /app/radix-common
ADD radix-common-derive /app/radix-common-derive
ADD radix-engine /app/radix-engine
ADD radix-engine-interface /app/radix-engine-interface
ADD radix-engine-profiling /app/radix-engine-profiling
ADD radix-engine-profiling-derive /app/radix-engine-profiling-derive
ADD radix-native-sdk /app/radix-native-sdk
ADD radix-rust app/radix-rust
ADD radix-sbor-derive /app/radix-sbor-derive
ADD radix-substate-store-impls /app/radix-substate-store-impls
ADD radix-substate-store-interface /app/radix-substate-store-interface
ADD radix-substate-store-queries /app/radix-substate-store-queries
ADD radix-transactions /app/radix-transactions
ADD sbor /app/sbor
ADD sbor-derive /app/sbor-derive
ADD sbor-derive-common /app/sbor-derive-common
ADD scrypto-compiler /app/scrypto-compiler

# Copy radix-clis crate
ADD radix-clis /app/radix-clis

WORKDIR /app

RUN cargo install --path ./radix-clis

FROM base-image
COPY --from=builder /app/radix-clis/target/release/scrypto /usr/local/bin/scrypto
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src

ENTRYPOINT ["scrypto", "build", "--path", "/src"]

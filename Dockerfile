# https://hub.docker.com/_/rust/tags
FROM rust:1.81.0-slim-bookworm AS base-image

RUN apt update && apt install -y \
    cmake=3.25.1-1 \
    clang=1:14.0-55.7~deb12u1 \
    build-essential=12.9 \
    llvm=1:14.0-55.7~deb12u1

FROM base-image AS builder

# Copy library crates
ADD Cargo.toml /app/Cargo.toml
ADD radix-blueprint-schema-init /app/radix-blueprint-schema-init
ADD radix-common /app/radix-common
ADD radix-common-derive /app/radix-common-derive
ADD radix-clis /app/radix-clis
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
ADD scrypto-bindgen /app/scrypto-bindgen
ADD scrypto-compiler /app/scrypto-compiler

# Add non-production dependencies...
# These only need to be included because cargo tries to read their Cargo.toml files when it's preparing the workspace
# Ideally, to minimize the image size, we could probably just write an almost-empty Cargo.toml file at each of these paths
# Will save this optimization for a later day
ADD radix-engine-monkey-tests /app/radix-engine-monkey-tests
ADD radix-engine-tests /app/radix-engine-tests
ADD radix-engine-toolkit-common /app/radix-engine-toolkit-common
ADD radix-transaction-scenarios /app/radix-transaction-scenarios
ADD sbor-tests /app/sbor-tests
ADD scrypto /app/scrypto
ADD scrypto-derive /app/scrypto-derive
ADD scrypto-derive-tests /app/scrypto-derive-tests
ADD scrypto-test /app/scrypto-test

WORKDIR /app

RUN cargo install --path ./radix-clis

# This dev-container image can be built with the following command:
# docker build . --target scrypto-dev-container -t scrypto-dev-container
FROM base-image AS scrypto-dev-container
RUN apt install -y curl bash-completion git
# Install improved prompt for better dev experience - https://starship.rs/
RUN curl -sS https://starship.rs/install.sh | sh -s -- -y
RUN echo 'eval "$(starship init bash)"\n . /etc/bash_completion' >> /root/.bashrc

COPY --from=builder /app/target/release/scrypto /usr/local/bin/scrypto
COPY --from=builder /app/target/release/resim /usr/local/bin/resim
COPY --from=builder /app/target/release/rtmc /usr/local/bin/rtmc
COPY --from=builder /app/target/release/rtmd /usr/local/bin/rtmd
COPY --from=builder /app/target/release/scrypto-bindgen /usr/local/bin/scrypto-bindgen
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt
RUN rustup component add clippy

FROM base-image AS scrypto-builder
COPY --from=builder /app/target/release/scrypto /usr/local/bin/scrypto
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src

ENTRYPOINT ["scrypto", "build", "--path", "/src"]

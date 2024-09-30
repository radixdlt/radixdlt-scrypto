#!/bin/sh

echo > /tmp/build.log
#find ../radix-engine-tests/assets/blueprints  -type d -maxdepth 1 -mindepth 1

# find folders containing Cargo.toml
fd  --min-depth 2 -x echo {//} ';' "Cargo.toml"  ../radix-engine-tests/assets/blueprints | while read d ; do
  {
    echo path = $d START
    cargo run --bin scrypto -- \
      build --env RUSTFLAGS="--deny deprecated" --path $d >/tmp/a.log 2>&1 \
        && \
          { echo "path = $d OK" ; } \
        || \
          { echo "path = $d ERR" ; cat /tmp/a.log ;}
  } | tee -a /tmp/build.log
done

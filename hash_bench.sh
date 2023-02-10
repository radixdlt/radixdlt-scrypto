#!/bin/bash

set -x
set -e
set -o pipefail

tstamp=`date -u  +%Y%m%d%H%M%S`
mkdir -p results


raw_file=results/hash_bench_${tstamp}.raw
res_file=results/hash_bench_${tstamp}.log

# bench SHA2
sha2_features=" \
sha2-default \
sha2-force-soft \
sha2-asm \
sha2-sha2-asm \
sha2-asm-aarch64 \
sha2-compress \
"

for f in $sha2_features ; do
    echo "hash_${f}"
    cargo bench -p radix-engine-interface --features $f --bench hash hash/SHA2 -- --save-baseline hash_${f}
done | tee $raw_file

# bench Blake2
blake2_features=" \
blake2-default \
blake2-simd \
blake2-simd_asm \
blake2-simd_opt \
blake2-size_opt \
"

# Apparently blake2-simd does not work on stable
rustup default nightly
for f in $blake2_features ; do
    echo "hash_${f}"
    cargo bench -p radix-engine-interface --features $f --bench hash hash/Blake2 -- --save-baseline hash_${f}
done | tee -a $raw_file
rustup default stable

# bench Blake2 stable
f=blake2-default
{
    echo "hash_${f}_stable";
    cargo bench -p radix-engine-interface --features $f --bench hash hash/Blake2 -- --save-baseline hash_${f}_stable;
} | tee -a $raw_file

# bench blake2_simd
f=blake2b_simd
{
    echo "hash_${f}";
    cargo bench -p radix-engine-interface --bench hash hash/blake2b_simd -- --save-baseline hash_${f};
} | tee -a $raw_file

set +x

cat $raw_file | \
    awk '!/thrpt:/&&NR>1{print OFS}{printf "%s ",$0}END{print OFS}' | \
    grep -A1 "^hash"  | sed -E 's/^ *time:/time:/g' | \
    awk '!/^time:/&&NR>1{print OFS}{printf "%s ",$0}END{print OFS}' | \
    grep "^hash" | \
    awk '{printf  $1"\t"$5"\t"$6"\t"$12"\t"$13 "\n"}' | tee $res_file

echo "results: $raw_file $res_file"


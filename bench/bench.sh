#!/bin/bash

set -e

cd "$(dirname "$0")"

RED='\033[0;31m'
NC='\033[0m' # No Color

# name without extension, num_iters, input number
function bench_program() {
    local name=$1
    local num_iters=$2
    local input=$3

    echo -e "### ${RED}Benchmarking $name ${NC}"

    rustc --crate-type=cdylib "$name.rs" -C opt-level=3 -o "${name}_rs.so" > /dev/null 2>&1
    cargo r -- "$name.con"  --library --release > /dev/null 2>&1
    cp "./build_artifacts/$name.so" "${name}_con.so"

    cc bench.c -L . -l:./"${name}"_rs.so -l:./"${name}"_con.so -Wl,-rpath -o bench_"${name}"

    ./bench_"${name}" "$num_iters" "$input"
}

bench_program "factorial" 5000000 20
bench_program "fib" 5000 20

rm ./*.so
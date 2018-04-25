#!/usr/bin/env bash

set -eux

cd "$(dirname $0)"

rustup target add wasm32-unknown-unknown --toolchain nightly

cargo +nightly build --release --target wasm32-unknown-unknown

WASM="$(pwd)/target/wasm32-unknown-unknown/release/intrusive_splay_tree_wasm.wasm"

if [[ -x "$(which wasm-gc)" ]]; then
    new_wasm="${WASM/\.wasm/.gc.wasm}"
    wasm-gc "$WASM" "$new_wasm"
    WASM="$new_wasm"
fi

if [[ -x "$(which wasm-opt)" ]];then
    new_wasm="${WASM/\.wasm/.opt.wasm}"
    wasm-opt -Oz "$WASM" -o "$new_wasm"
    WASM="$new_wasm"
fi

cd "$(dirname $WASM)"
ls -1 | grep '\.wasm$' | xargs wc -c
ls -1 | grep '\.wasm$' | xargs -I '{}' sh -c 'echo -n "{} gzipped is "; cat "{}" | gzip --best | wc -c'

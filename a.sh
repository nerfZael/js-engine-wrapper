#!/./bin/sh
set -x
# Install the wasm32 rust build target
rustup target add wasm32-unknown-unknown

# Install the toml-cli
cargo install toml-cli

# Install wasm-snip
cargo install wasm-snip

# Install wasm-bindgen
cargo install wasm-bindgen-cli

# Install wasm-tools
cargo install wasm-tools

# Ensure the Wasm module is configured to use imported memory
export RUSTFLAGS="-C link-arg=-z -C link-arg=stack-size=65536 -C link-arg=--import-memory"

# Build the module
cargo build --manifest-path Cargo.toml \
  --target wasm32-unknown-unknown --release

# Enable the "WASM_INTERFACE_TYPES" feature, which will remove the __wbindgen_throw import.
# See: https://github.com/rustwasm/wasm-bindgen/blob/7f4663b70bd492278bf0e7bba4eeddb3d840c868/crates/cli-support/src/lib.rs#L397-L403
export WASM_INTERFACE_TYPES=1

# Run wasm-bindgen over the module, replacing all placeholder __wbindgen_... imports
wasm-bindgen ./target/wasm32-unknown-unknown/release/module.wasm --out-dir ./bin --out-name bg_module.wasm

# Run wasm-tools strip to remove the wasm-interface-types custom section
wasm-tools strip ./bin/bg_module.wasm -d wasm-interface-types -o ./bin/strip_module.wasm
rm -rf ./bin/bg_module.wasm

# Run wasm-snip to trip down the size of the binary, removing any dead code
wasm-snip ./bin/strip_module.wasm -o ./bin/snipped_module.wasm
rm -rf ./bin/strip_module.wasm

# Use wasm-opt to perform the "asyncify" post-processing step over all modules
export ASYNCIFY_STACK_SIZE=24576
wasm-opt --asyncify -Os ./bin/snipped_module.wasm -o ./bin/wrap.wasm
rm -rf ./bin/snipped_module.wasm

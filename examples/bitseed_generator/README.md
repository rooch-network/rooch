# Inscribe Generate Bitseed Contract

A CosmWasm smart contract for generating and verifying custom attributes for bitseed assets or NFTs.

## Features

- Custom attribute generation rules (e.g., range, list)
- Deterministic generation based on seed and user input
- Attribute verification
- Extensible design

## Build, Test, and Optimize

### Prerequisites

Before building and optimizing the contract, you need to install the following dependencies:

1. Rust and Cargo (https://www.rust-lang.org/tools/install)
2. wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)
3. wasm-opt (part of the Binaryen toolkit):
   - On Ubuntu/Debian: `sudo apt-get install binaryen`
   - On macOS with Homebrew: `brew install binaryen`
   - For other systems, visit: https://github.com/WebAssembly/binaryen
4. wasm-snip:
   ```
   cargo install wasm-snip
   ```

### Build

```
cargo wasm
```

### Test

```
cargo test
```

### Optimize

To optimize the WASM binary, run:

```
make optimize
```

This command will build the contract, optimize it using `wasm-opt`, and apply `wasm-snip` to further reduce the binary size. The optimized WASM file will be placed in the `./artifacts` directory.

## Usage

Deploy to a CosmWasm-compatible blockchain and interact using:
- `InscribeGenerate`: Generate attributes
- `InscribeVerify`: Verify attributes
- `IndexerGenerate`: Generate for indexing

Refer to the contract's query messages for input/output structures.

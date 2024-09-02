# Inscribe Generate Contract

A CosmWasm smart contract for generating and verifying custom attributes for digital assets or NFTs.

## Features

- Custom attribute generation rules (e.g., range, list)
- Deterministic generation based on seed and user input
- Attribute verification
- Extensible design

## Build and Test

### Build

```
cargo wasm
```

### Test

```
cargo test
```

## Usage

Deploy to a CosmWasm-compatible blockchain and interact using:
- `InscribeGenerate`: Generate attributes
- `InscribeVerify`: Verify attributes
- `IndexerGenerate`: Generate for indexing

Refer to the contract's query messages for input/output structures.

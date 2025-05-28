# DID Keygen Command Usage Guide

> **Note**: This document supplements the [Rooch DID Implementation Guide](./rooch-did-implementation-guide.en.md) and provides detailed command usage examples.

The DID keygen command provides a convenient way to generate keys for DID operations.

## Command Overview

```bash
rooch did keygen <COMMAND>
```

Supported subcommands:
- `ed25519` - Generate Ed25519 key pair
- `secp256k1` - Generate Secp256k1 key pair
- `did-key` - Generate did:key identifier from multibase public key

## Ed25519 Key Generation

### Basic Usage

Generate Ed25519 key pair (public key only):
```bash
rooch did keygen ed25519
```

Example output:
```json
{
  "key_type": "Ed25519",
  "public_key": {
    "multibase": "z1BvzpeZSvoGY178AXWQCcphK9FQqnGmT4RKEkTHtMSybF",
    "hex": "0xa26a07f86986722dde18e021dc793c86da0dc687ce5f1fc065f96288d69fb3aa",
    "base64": "AKJqB/hphnIt3hjgIdx5PIbaDcaHzl8fwGX5YojWn7Oq",
    "raw_multibase": null
  },
  "private_key": null,
  "did_key": "did:key:z6MkqPFsEohN8p2UDd1EByATfns94z7dgehR7L9gHZrNNCNd"
}
```

### Generate Raw Public Key

Use the `--raw` flag to generate a raw public key without the scheme flag:
```bash
rooch did keygen ed25519 --raw
```

This will include the `raw_multibase` field in the output, suitable for DID verification methods.

### Include Private Key

**⚠️ Warning: Private key information is sensitive, please use with caution.**

```bash
rooch did keygen ed25519 --include-private --raw
```

Example output:
```json
{
  "key_type": "Ed25519",
  "public_key": {
    "multibase": "z18y1uAidokabgSwuhueACRx9Eia2tEBxg1KAkf8TDqZwU",
    "hex": "0x765b59cd96ae8173ae5f318fb50ce3160b1d1acc25cac0f78371a7f08f942f17",
    "base64": "AHZbWc2WroFzrl8xj7UM4xYLHRrMJcrA94Nxp/CPlC8X",
    "raw_multibase": "z8y1uAidokabgSwuhueACRx9Eia2tEBxg1KAkf8TDqZwU"
  },
  "private_key": {
    "hex": "0x9c07f46bda2e5f4fa939d178ef80b0171ef4e8fcecf5ea9c75b166ab08e90784",
    "base64": "AJwH9GvaLl9PqTnReO+AsBce9Oj87PXqnHWxZqsI6QeE",
    "bech32": "roochsecretkey1qzwq0artmgh97naf88gh3muqkqt3aa8glnk0t65uwkckd2cgayrcgxaq60x"
  },
  "did_key": "did:key:z6MknRGwkxtF6869ZSkQbD83H3hEY9Jje5D2hL5gVQREknir"
}
```

## Secp256k1 Key Generation

Usage for Secp256k1 key generation is the same as for Ed25519:

```bash
# Basic generation
rooch did keygen secp256k1

# Generate raw public key
rooch did keygen secp256k1 --raw

# Include private key
rooch did keygen secp256k1 --include-private --raw
```

Example output:
```json
{
  "key_type": "Secp256k1",
  "public_key": {
    "multibase": "z2L3vnMQRLduvnxRCXpbMqVSKNNdCxBFYaAWCAyn7m4L5Nx",
    "hex": "0x0360003fcf2ff923c149c7bf406e92d5f2b6257dc1e87d4346c9616ae727cbed31",
    "base64": "AQNgAD/PL/kjwUnHv0BuktXytiV9weh9Q0bJYWrnJ8vtMQ==",
    "raw_multibase": "z219hULUu4fr2EQ64tSSPbup2renDSYGVxh9yjSJSa8ENL"
  },
  "private_key": null,
  "did_key": "did:key:zQ3shm6rVBdNXzVF7eBkJpXGejQQJJYQQppYVd5jXje7hLiE4"
}
```

## DID Key Generation

Generate a did:key identifier from an existing multibase public key:

```bash
rooch did keygen did-key <MULTIBASE_PUBLIC_KEY> --key-type <KEY_TYPE>
```

### Ed25519 Example

```bash
rooch did keygen did-key z8y1uAidokabgSwuhueACRx9Eia2tEBxg1KAkf8TDqZwU --key-type ed25519
```

### Secp256k1 Example

```bash
rooch did keygen did-key z219hULUu4fr2EQ64tSSPbup2renDSYGVxh9yjSJSa8ENL --key-type secp256k1
```

## Usage in DID Operations

### 1. Generating Keys for CADOP

```bash
# Generate user key
rooch did keygen ed25519 --raw

# Generate custodian service key
rooch did keygen ed25519 --raw

# Create CADOP DID using generated keys
rooch did create cadop \
  --user-did-key <USER_DID_KEY> \
  --custodian-service-key <CUSTODIAN_RAW_MULTIBASE>
```

### 2. Generating Keys for Verification Methods

```bash
# Generate new verification method key
rooch did keygen ed25519 --raw

# Add to DID document
rooch did manage add-vm \
  --did-address <DID_ADDRESS> \
  --fragment key-2 \
  --public-key <RAW_MULTIBASE> \
  --relationships auth,assert
```

## Output Format Explanation

### Public Key Formats

- **multibase**: Full public key including scheme flag, base58btc encoded.
- **hex**: Hexadecimal format, with 0x prefix.
- **base64**: Base64 encoded format.
- **raw_multibase**: Raw public key bytes without scheme flag, base58btc encoded.

### Private Key Formats

- **hex**: Hexadecimal format, with 0x prefix.
- **base64**: Base64 encoded format.
- **bech32**: Rooch formatted bech32 encoded private key.

### DID Key Format

Generated did:key identifiers follow the W3C DID Key specification, using correct multicodec prefixes:
- Ed25519: `did:key:z6Mk...` (starts with z6Mk, multicodec 0xed01)
- Secp256k1: `did:key:zQ3s...` (starts with zQ3s, multicodec 0xe701)

Format: `did:key:MULTIBASE(base58-btc, MULTICODEC(key-type, raw-key-bytes))`

## Security Considerations

1. **Private Key Security**: Exercise extreme caution when using `--include-private`. Ensure operations are performed in a secure environment.
2. **Key Storage**: Store generated private keys securely. Do not expose them in logs or insecure locations.
3. **Test Environment**: It is recommended to verify key correctness in a test environment before using them in production.

## Common Use Cases

1. **Testing DID Functionality**: Quickly generate key pairs for testing.
2. **CADOP Integration**: Generate necessary keys for custodian-assisted DID creation.
3. **Verification Method Management**: Add new verification methods to DID documents.
4. **Key Format Conversion**: Convert between different key formats. 
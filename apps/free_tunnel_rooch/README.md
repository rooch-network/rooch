# Free Tunnel Rooch

A bridge protocol implementation on Rooch blockchain that enables cross-chain token transfers.

## Project Structure

The project consists of two main packages:

### 1. Atomic Package (`/atomic`)
Core bridge protocol implementation with features:
- Atomic coin minting and burning
- Atomic coin locking and unlocking 
- Multi-signature executor validation
- Permission management for admin and proposers
- Support for multiple coin types

### 2. Minter Manager Package (`/minter_manager`)
Minter management implementation for coin operations.

## Development

### Prerequisites

- [Rooch CLI](https://rooch.network/docs/get-started/installation)

### Build

Build both packages separately:

```bash
# Build atomic package
cd atomic
rooch move build

# Build minter manager package
cd minter_manager
rooch move build
```

### Test

Run tests for each package:

```bash
# Test minter manager package
cd minter_manager
rooch move test --named-addresses minter_manager="0xaaff"

# Test free tunnel package
cd free_tunnel
rooch move test --named-addresses minter_manager="0xaaff",free_tunnel_rooch="0xbbee"
```

Since the `free_tunnel_rooch` and `minter_manager` are not filled in the `Move.toml`, you need to pass them as named addresses in the command.

### Project Structure
```
.
├── atomic/                 # Core bridge protocol
│   ├── sources/
│   │   ├── lock/          # Lock contract implementation
│   │   ├── mint/          # Mint contract implementation
│   │   ├── Permissions.move
│   │   ├── ReqHelpers.move
│   │   └── Utils.move
│   └── test/
│       └── integrationTest.sh
│
└── minter_manager/        # Minter management implementation
    └── sources/
        └── MinterManager.move
```
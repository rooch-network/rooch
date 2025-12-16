# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rooch Network is a Bitcoin-focused Layer 2 blockchain solution positioned as a "VApp (Verifiable Application) Container" with the Move programming language. The project aims to provide verifiability of both computations and states within applications, ensuring transparency and reliability of operations.

## Development Environment Setup

### Prerequisites
```bash
# Rust toolchain (required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Node.js and pnpm (for TypeScript/E2E testing)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g pnpm

# Optional: Docker (for containerized testing)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
```

### Initial Setup
```bash
# Clone repository
git clone https://github.com/rooch-network/rooch.git
cd rooch

# Install Rust dependencies
cargo build

# Install Node.js dependencies for E2E testing
cd sdk/typescript/test-suite
pnpm install

# Initialize Rooch configuration (optional)
rooch init
```

### Environment Variables
```bash
# Build profiles
export ROOCH_BINARY_BUILD_PROFILE=debug  # Development builds
export ROOCH_BINARY_BUILD_PROFILE=optci  # Optimized builds for testing

# Debug configuration
export RUST_LOG=debug     # Enable debug logging
export RUST_BACKTRACE=1   # Show backtrace on errors
```

## Commands for AI Assistant

### Build System (Primary)
The project uses a comprehensive Makefile as the primary development interface:

```bash
# High-level commands
make build          # Build Rust (release) and all Move components
make test           # Run all Rust and Move tests
make quick-check    # Quick compilation check (debug)
make ci-checks      # Run all CI checks
make dev            # Development cycle: clean, build, test
make clean-all      # Clean all artifacts

# Specific builds
make build-rust     # Build Rust components
make build-move     # Build Move frameworks
make test-move      # Run Move tests
make test-rust      # Run Rust tests

# Linting and formatting
make lint           # Run all linters (includes non-ASCII comment check)
make lint-rust      # Run Rust clippy and machete
```

### CLI Commands for Testing
```bash
# Building and installation
cargo build && cp target/debug/rooch ~/.cargo/bin/

# Core workflow for testing
rooch init                    # Initialize config
rooch move build -p <path>    # Build Move project
rooch move test -p <path> [filter]  # Run Move tests

# Server operations
rooch server start -n local   # Start local server for testing

# Package-specific testing
cargo test --package <name>         # Run Rust tests
```

### SDK E2E Testing (TypeScript)
```bash
cd sdk/typescript/test-suite
pnpm install
pnpm test:e2e              # Run E2E tests with server
pnpm test:unit             # Run unit tests
pnpm test                  # Run all tests
pnpm lint                  # ESLint and Prettier checks
```

## High-Level Architecture

### Core Components

#### 1. **MoveOS** (`/moveos/`)
A standalone Move runtime environment providing:
- **State storage**: Local database with RocksDB as default backend
- **Move VM execution**: Move byte code execution with WASM support
- **State proofs**: SMT-based state tree for verification
- **Rust-To-Move ABI**: Cross-language integration patterns

Key subcomponents:
- `moveos-store/`: Storage abstraction layer
- `moveos-types/`: Type definitions for Move objects
- `moveos-verifier/`: Move code verification
- `smt/`: Sparse Merkle Tree implementation

#### 2. **Rooch Core** (`/crates/rooch-*/`)
Main blockchain implementation with modular architecture:

**Core Infrastructure:**
- `rooch`: Main binary and CLI interface
- `rooch-config`: Configuration management with network profiles
- `rooch-executor`: Transaction execution engine
- `rooch-sequencer`: Transaction ordering and sequencing
- `rooch-proposer`: Block proposal and consensus

**Storage and State:**
- `rooch-store`: High-level storage operations
- `rooch-db`: Database operations and migrations
- `rooch-pruner`: State pruning and GC system

**Network and APIs:**
- `rooch-rpc-server`: JSON-RPC server with OpenAPI specs
- `rooch-indexer`: Data indexing for queries
- `bitcoin-client`: Bitcoin network integration

#### 3. **Move Frameworks** (`/frameworks/`)
Four-tier framework architecture by address:
- `move-stdlib` (0x1): Standard Move library
- `moveos-stdlib` (0x2): MoveOS-specific libraries
- `rooch-framework` (0x3): Rooch blockchain framework
- `bitcoin-move` (0x4): Bitcoin-specific Move libraries
- `rooch-nursery`: Experimental features

### State Management Architecture

The storage system uses a multi-layered approach:

```
┌─────────────────────────────────────┐
│           MoveOS Framework         │
│  (Object Storage, Move Types)       │
├─────────────────────────────────────┤
│         StateTree (SMT)             │
│  (State Proofs, Verification)       │
├─────────────────────────────────────┤
│        MoveOSStore Layer            │
│  (Node Store, Metadata Store)     │
├─────────────────────────────────────┤
│       RocksDB Backend               │
│  (Column Families, Persistence)    │
└─────────────────────────────────────┘
```

**Column Families:**
- `cf_smt_nodes`: SMT node data with hash as key
- `cf_state_roots`: State root timeline with timestamps
- `cf_state_node_store`: Object state data
- `cf_indexer_*`: Various indexing structures

### Current Development Focus

Based on recent activity, major development areas include:

1. **Pruner/GC System Refactoring**: Complete redesign of state pruning with:
   - Multi-phase garbage collection (BuildReach → SweepExpired → Incremental)
   - Bloom filter optimizations for memory efficiency
   - Stop-the-world GC operations with minimal downtime

2. **Bitcoin Integration**: Enhanced Bitcoin UTXO and Inscription support

3. **Testing Infrastructure**: Comprehensive E2E testing with TypeScript/Node.js tooling

## Critical Development Rules

### Language and Encoding
- **Move files**: ASCII-only comments and strings required by Move compiler
- **Rust files**: English comments and documentation only
- **Exception**: Only test data strings (not comments) in `moveos/moveos-types/src/move_std/string.rs` may use UTF-8; this is intentional for testing purposes

### Testing Patterns

#### Rust Unit Tests
```bash
# Run specific package tests
cargo test --package rooch-pruner --release
cargo test --package moveos-types test_utf8

# Run with output
cargo test --package moveos-types test_utf8 -- --nocapture
```

#### Move Tests
```bash
# Framework tests
make test-move-frameworks
make test-move-did

# Project tests
rooch move test -p examples/basic_object
rooch move test -p frameworks/rooch-framework [filter]
```

#### E2E Tests (TypeScript)
```bash
cd sdk/typescript/test-suite
pnpm test:e2e
pnpm test:unit
```

### Code Quality Standards

#### Build Verification
- Always run `make quick-check` after changes
- Use `make lint` to check for non-ASCII comments
- Verify with `make test` before submitting PRs

#### Error Handling Patterns
```rust
// Rooch error handling with anyhow
context("Failed to process transaction")?;
bail!("Invalid input: {}", reason);
```

```move
// Move error constants
const ErrorCodeOne: u64 = 1;
const ErrorCodeTwo: u64 = 2;
assert!(condition, ErrorCodeOne);
```

## File Organization Patterns

### Workspace Structure
- **Cargo Workspace**: 68+ crates organized by domain
- **Version Management**: Workspace-level versioning (see `Cargo.toml` for current version)
- **Framework Publishing**: Address-based deployment (0x1, 0x2, 0x3, 0x4)

### Common File Locations
- **Move Frameworks**: `frameworks/*/sources/`
- **Rooch Core**: `crates/rooch-*/src/`
- **MoveOS Core**: `moveos/*/src/`
- **Integration Tests**: `sdk/typescript/*/src/case/`
- **Examples**: `examples/*/sources/`

### Configuration Files
- **Rooch Config**: `~/.rooch/rooch_config.json`
- **Network Profiles**: local, dev, test, mainnet
- **Bitcoin Integration**: Optional RPC configuration

## Development Workflow

### New Feature Development
1. **Framework Changes**: Implement in `frameworks/rooch-framework/sources/`
2. **Core Changes**: Implement in appropriate `crates/rooch-*/src/`
3. **Testing**: Add unit tests + Move tests + E2E tests if applicable
4. **Verification**: `make build && make test && make lint`
5. **Documentation**: Update relevant docs if public APIs change

### Bug Fix Workflow
1. **Reproduction**: Create test case that reproduces issue
2. **Fix**: Implement minimal fix with comprehensive test coverage
3. **Validation**: Ensure all tests pass, including regression tests
4. **Cleanup**: Remove any temporary debugging code

### Performance Optimization
1. **Profiling**: Use `cargo bench` for Rust performance testing
2. **Metrics**: Leverage Prometheus metrics for runtime monitoring
3. **Database**: Profile RocksDB operations and column family usage
4. **Memory**: Monitor SMT node storage and Bloom filter efficiency

## Key Integrations

### Bitcoin Integration
- **UTXO Management**: Bitcoin address derivation and transaction handling
- **Inscription Support**: Ordinals and BRC-20 token standards
- **Cross-chain**: Bitcoin-to-Rooch asset bridging

### Testing Infrastructure
- **TestBox**: Containerized Rooch instances for E2E testing
- **Prometheus**: Metrics collection and validation
- **Vitest**: TypeScript testing framework
- **Cucumber**: Integration test scenarios

### Storage Backend
- **Primary**: RocksDB with optimized column families
- **State Tree**: SMT for efficient state proofs
- **Indexing**: Multiple indexing strategies for query performance

## Troubleshooting Common Issues

### Build Issues
```bash
# Clean build if encountering compilation errors
make clean-all
cargo clean

# Update Rust toolchain if needed
rustup update stable

# Clear Cargo cache (last resort)
rm -rf ~/.cargo/registry/cache
```

### Test Failures
```bash
# Run tests with verbose output
cargo test --package <name> -- --nocapture

# Run specific test patterns
cargo test --package rooch-pruner gc
make test-move-frameworks

# Check test environment
rooch env
```

### Essential Testing Commands
```bash
# Linting (critical for this repository)
make lint-rust                      # Check for non-ASCII comments
make lint                           # Run all linters

# Build verification
make quick-check                    # Fast compilation check
make build                          # Full build

# Environment variables for optimized testing
export ROOCH_BINARY_BUILD_PROFILE=optci  # Use optimized binary for tests
```

### Key File Locations for Context
- **Framework APIs**: `frameworks/*/sources/` - Core Move interfaces and types
- **Test Examples**: `sdk/typescript/*/src/case/` - Usage patterns and integration tests
- **Move Examples**: `examples/*/sources/` - Sample Move projects

This architecture enables Rooch to provide scalable, verifiable applications with strong Bitcoin ecosystem integration while maintaining Move language safety guarantees.

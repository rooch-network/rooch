# Quick Start: Rooch Dev Base Image

## Build the Image

```bash
# From repository root
docker build -f docker/DockerfileDevBase -t rooch-dev-base:latest .
```

## Basic Usage

### Interactive Development Shell

```bash
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  rooch-dev-base:latest \
  bash
```

Inside the container:
```bash
# Build Rooch
cargo build --release

# Start Rooch server
./target/release/rooch server start -n local

# Build Move project
rooch move build -p examples/simple_coin

# Run tests
make test
```

### TypeScript SDK Development

```bash
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch/sdk/typescript/test-suite \
  rooch-dev-base:latest \
  bash

# Inside container:
pnpm install
pnpm test
```

### Bitcoin Regtest Development

```bash
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  -p 18444:18444 \
  rooch-dev-base:latest \
  bash

# Inside container, start bitcoind in regtest mode:
bitcoind -regtest -server=1 -rest=1 -fallbackfee=0.0001 -daemon

# Interact with bitcoind:
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest createwallet test
bitcoin-cli -regtest -generate 1
```

### Continuous Development Container

```bash
# Run container with named volume for persistent workspace
docker run -it --rm \
  --name rooch-dev \
  -v $(pwd):/rooch \
  -w /rooch \
  -p 6767:6767 \
  rooch-dev-base:latest \
  bash

# In another terminal, attach to running container:
docker exec -it rooch-dev bash
```

## Verification

Run the verification script to ensure the image meets all requirements:

```bash
# After building the image
./docker/verify_dev_base.sh rooch-dev-base:latest

# Or for the published image
./docker/verify_dev_base.sh ghcr.io/rooch-network/rooch-dev-base:latest
```

## Expected Versions

When verified, the image should report:
- Rust: 1.82.0
- Node.js: v20.x.x
- pnpm: 9.10.0
- bitcoind: Bitcoin Core x.x.x
- SQLite: 3.46.1

## Common Workflows

### Full Rooch Build and Test

```bash
docker run --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  rooch-dev-base:latest \
  bash -c "
    cargo build --release && \
    make test-move && \
    rooch move build -p frameworks/rooch-framework
  "
```

### CI/CD Pipeline Simulation

```bash
docker run --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  -e RUST_BACKTRACE=1 \
  rooch-dev-base:latest \
  bash -c "
    cargo clean &&
    cargo build --workspace &&
    cargo test --workspace
  "
```

### Move Framework Development

```bash
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  rooch-dev-base:latest \
  bash

# Inside container:
cd frameworks/rooch-framework
rooch move build
rooch move test
```

## Troubleshooting

### Out of Memory During Build

If Docker runs out of memory during compilation:
```bash
# Increase Docker memory limit to 4GB+ in Docker Desktop settings
# Or use a lighter build:
cargo check --workspace  # Faster than full build
```

### Permission Issues

If you encounter permission issues with volume mounts:
```bash
# The container runs as root, so files created in the container
# will be owned by root on the host. Fix with:
sudo chown -R $USER:$USER .
```

### bitcoind Won't Start

```bash
# Check if bitcoind is already running:
ps aux | grep bitcoind

# Use a custom data directory:
bitcoind -regtest -datadir=/tmp/bitcoin-regtest
```

## Holon AI Integration

This image is designed for use with Holon AI agents. Example:

```yaml
# Holon configuration
docker:
  image: rooch-dev-base:latest
  volumes:
    - .:/rooch
  working_dir: /rooch
  environment:
    RUST_LOG: debug
    RUST_BACKTRACE: 1
```

## Registry Publishing

```bash
# Tag for GitHub Container Registry
docker tag rooch-dev-base:latest ghcr.io/rooch-network/rooch-dev-base:latest

# Login to GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Push to registry
docker push ghcr.io/rooch-network/rooch-dev-base:latest
```

## Next Steps

- See [docker/README.md](./README.md) for comprehensive documentation
- See [docs/dev-guide/](../docs/dev-guide/) for Rooch development guides
- See [github/workflows/docker_build_dev_base.yml](../.github/workflows/docker_build_dev_base.yml) for CI/CD setup

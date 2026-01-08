# Rooch Docker Images

This directory contains Dockerfiles for building and running Rooch in various environments.

## Images

### Production Images

#### Dockerfile
Standard production image for running Rooch server. Builds a minimal runtime image with the Rooch binary precompiled.

**Usage:**
```bash
docker build -f docker/Dockerfile -t rooch:latest .
docker run -p 6767:6767 rooch:latest server start
```

#### DockerfileDebug
Debug-enabled image with additional debugging tools and symbols.

### Development Images

#### DockerfileDevBase
**New:** Complete development base image for Rooch development environments.

This image is designed for:
- Holon AI agent and other automated development tools
- Local development environments with full toolchain
- CI/CD pipelines requiring complete build environment
- Integration testing with Move, TypeScript SDK, and Bitcoin regtest

**Included Components:**
- **Rust 1.91.1** - Matches `rust-toolchain.toml`
- **Move compiler** - Via Rust toolchain
- **Node.js 20.x** - For TypeScript SDK development
- **pnpm 9.10.0** - Matches root `package.json`
- **bitcoind** - For local regtest testing
- **SQLite 3.46.1** - Compiled with `SQLITE_THREADSAFE=2` (matching production)

**Building the Image:**
```bash
docker build -f docker/DockerfileDevBase -t rooch-dev-base:latest .
```

**Tagging for Registry:**
```bash
docker tag rooch-dev-base:latest ghcr.io/rooch-network/rooch-dev-base:latest
docker tag rooch-dev-base:latest ghcr.io/rooch-network/rooch-dev-base:v0.12.1
```

**Publishing to Registry:**
```bash
docker push ghcr.io/rooch-network/rooch-dev-base:latest
docker push ghcr.io/rooch-network/rooch-dev-base:v0.12.1
```

**Basic Usage:**
```bash
# Start an interactive shell
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  rooch-dev-base:latest \
  bash

# Build Rooch
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  rooch-dev-base:latest \
  cargo build --release

# Run Move tests
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  rooch-dev-base:latest \
  make test-move

# Run TypeScript SDK tests
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch/sdk/typescript/test-suite \
  rooch-dev-base:latest \
  pnpm install && pnpm test
```

**Running with Bitcoin Regtest:**
```bash
# Start bitcoind in regtest mode
docker run -it --rm \
  -v $(pwd):/rooch \
  -w /rooch \
  -p 18444:18444 \
  rooch-dev-base:latest \
  bash -c "bitcoind -regtest -server=1 -rest=1 & \
           sleep 5 && \
           bitcoin-cli -regtest getblockchaininfo"
```

**Development Container (docker-compose-style workflow):**
```bash
# Run as a development container with persistent workspace
docker run -it --rm \
  --name rooch-dev \
  -v $(pwd):/rooch \
  -w /rooch \
  -p 6767:6767 \
  rooch-dev-base:latest \
  bash

# Inside the container:
cargo build --release
./target/release/rooch server start -n local
```

**Environment Variables:**
The image respects standard Rooch environment variables:
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Show backtrace on errors
- `ROOCH_BINARY_BUILD_PROFILE=debug` - Build profile
- `ENV_TEST_ON_CI=1` - Enable CI-specific test behavior

**Verification:**
```bash
# Verify all tools are installed
docker run --rm rooch-dev-base:latest bash -c "
  rustc --version &&
  cargo --version &&
  node --version &&
  pnpm --version &&
  bitcoind --version &&
  sqlite3 --version
"
```

Expected output:
- `rustc 1.91.1`
- `cargo 1.91.1`
- `node v20.x.x`
- `pnpm 9.10.0`
- `Bitcoin Core version x.x.x`
- `sqlite3 3.46.1`

**Use Cases:**

1. **Holon AI Agent**: Provides a clean, reproducible environment for AI-driven development
2. **Local Development**: Consistent toolchain without polluting host system
3. **CI/CD**: Pre-configured environment for automated builds and tests
4. **Integration Testing**: Complete stack for testing Move, TypeScript SDK, and Bitcoin integration

**Image Size:**
The image is approximately 1.5-2 GB due to the complete development toolchain.

**Maintenance:**
- Update Rust version: Modify the `rustup-init` command in `DockerfileDevBase`
- Update Node.js: Change the NodeSource repository version
- Update pnpm: Modify the npm install version
- Update SQLite: Change the SQLite download URL and version

**Related Documentation:**
- [Rooch Development Guide](../docs/dev-guide/)
- [Move Development Guide](../docs/dev-guide/rooch_move_guide.md)
- [TypeScript SDK Documentation](../sdk/typescript/README.md)

## Kubernetes

See the `kube/` directory for Kubernetes deployment configurations, including:
- `kube/runner/` - CI runner configurations
- Service deployments and ingress configurations

## Best Practices

1. **Use Volume Mounts**: Always mount your workspace directory to avoid rebuilding the image
2. **Resource Limits**: Allocate at least 4GB RAM for compilation tasks
3. **Caching**: Docker layers are optimized for caching; dependencies are installed in order
4. **Security**: This development image is not intended for production use
5. **Networking**: Expose necessary ports (6767 for Rooch RPC, 18444 for Bitcoin regtest)

## Troubleshooting

**Build failures:**
- Ensure Docker has sufficient memory (4GB+ recommended)
- Check for network connectivity when downloading dependencies

**Runtime errors:**
- Verify volume mounts are correct
- Check that ports are not already in use
- Ensure environment variables are properly set

**bitcoind issues:**
- Bitcoin regtest runs on port 18444 by default
- Use `bitcoin-cli -regtest` to interact with regtest node
- Check bitcoind logs for startup issues

## Support

For issues or questions:
- GitHub Issues: https://github.com/rooch-network/rooch/issues
- Documentation: https://docs.rooch.network

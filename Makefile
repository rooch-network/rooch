# Rooch Development Makefile

.PHONY: help \
        all \
        build build-rust build-rust-debug build-rust-release build-move \
        test test-rust test-rust-unit test-rust-integration test-integration test-move test-move-frameworks test-move-did test-move-examples \
        fmt fmt-rust \
        lint lint-rust \
        clean clean-all clean-rust clean-move \
        rust-machete rust-clippy \
        move-framework move-stdlib move-nursery move-bitcoin-framework move-examples \
        ci-checks verify dev quick-check install-tools generate-genesis

# Default target: Show help
help:
	@echo "Rooch Development Commands:"
	@echo ""
	@echo "  High-Level Targets:"
	@echo "    all                 - Build and test everything (Rust release, Move)"
	@echo "    ci-checks           - Run all checks typically performed in CI (Rust release, Move)"
	@echo "    dev                 - Development cycle: clean, build (Rust release, Move), test (Rust, Move)"
	@echo "    quick-check         - Quick compilation check (Rust debug, rooch-framework)"
	@echo "    clean-all           - Clean all Rust and Move build artifacts"
	@echo ""
	@echo "  Build Targets:"
	@echo "    build               - Build Rust (release) and all Move components"
	@echo "    build-rust          - Build Rust project (profile: '$${RUST_PROFILE_DEFAULT}') -> alias for build-rust-debug"
	@echo "    build-rust-debug    - Build Rust project (debug profile)"
	@echo "    build-rust-release  - Build Rust project (release profile, e.g., 'optci')"
	@echo "    build-move          - Build all core Move frameworks (rooch-framework, moveos-stdlib, etc.)"
	@echo ""
	@echo "  Test Targets:"
	@echo "    test                - Run all Rust and Move tests (Rust with release profile)"
	@echo "    test-rust           - Run all Rust tests (unit, integration with release profile)"
	@echo "    test-move           - Run all Move tests (frameworks, examples)"
	@echo "    test-move-frameworks- Run tests for all core Move frameworks"
	@echo "    test-move-did       - Run Move DID module tests (within rooch-framework)"
	@echo "    test-move-examples  - Build and run Move example tests"
	@echo "    test-integration    - Run Cucumber integration tests (testsuite only)"
	@echo "    test-integration    - Run Cucumber integration tests (testsuite only, use FILTER=<name> to filter)"
	@echo ""
	@echo "  Linting and Formatting:"
	@echo "    fmt-rust            - Check Rust code formatting"
	@echo "    lint                - Run all linters (Rust clippy, Rust machete, Move non-ASCII check)"
	@echo "    lint-rust           - Run Rust clippy and machete linters"
	@echo ""
	@echo "  Rust Specific:"
	@echo "    rust-clippy         - Run Rust clippy linter"
	@echo "    rust-machete        - Check for unused Rust dependencies (requires cargo-machete)"
	@echo ""
	@echo "  Move Specific:"
	@echo "    move-framework      - Build rooch-framework only"
	@echo "    move-stdlib         - Build moveos-stdlib only"
	@echo "    move-nursery        - Build rooch-nursery framework"
	@echo "    move-bitcoin-framework - Build bitcoin-move framework"
	@echo "    move-examples       - Build all Move example projects (generic build)"
	@echo ""
	@echo "  Optional Parameters:"
	@echo "    FILTER=<pattern>    - Filter tests by name pattern (works with test-integration, test-move-frameworks)"
	@echo "                          Examples: make test-integration FILTER=payment_channel"
	@echo "                                   make test-move-frameworks FILTER=did"
	@echo ""
	@echo "  Cleaning Targets:"
	@echo "    clean-rust          - Clean Rust build artifacts (cargo clean)"
	@echo "    clean-move          - Clean Move build artifacts (frameworks and examples build dirs)"
	@echo ""
	@echo "  Utilities:"
	@echo "    install-tools       - Install required cargo tools (cargo-machete, cargo-nextest)"
	@echo "    verify              - Verify Rooch CLI availability and version via cargo run"
	@echo ""
	@echo "Note: Rust builds default to 'debug' profile unless 'release' is specified (e.g., build-rust-release)."
	@echo "      Move tasks use an optimized Rooch binary (built with '$(RUST_PROFILE_RELEASE)' profile)."

# Variables
RUST_PROFILE_DEFAULT = dev
RUST_PROFILE_RELEASE = optci # Profile for optimized/CI builds, as used in pr.sh

# Determine the profile for building the rooch CLI used for Move tasks
# Always use the release profile for the rooch binary when interacting with Move packages
# If you want to use the default (dev) profile, the binary is in target/debug, not target/dev
# Can be overridden by environment variable
ROOCH_BINARY_BUILD_PROFILE ?= debug
ROOCH_BIN_PATH = target/$(ROOCH_BINARY_BUILD_PROFILE)/rooch
ROOCH_CMD = $(ROOCH_BIN_PATH)
ROOCH_GENESIS_BIN = target/$(ROOCH_BINARY_BUILD_PROFILE)/rooch-genesis

# Default Rust build alias
build-rust: build-rust-debug

# Cleaning
clean-all: clean-rust clean-move
	@echo "🧹 All Rust and Move build artifacts cleaned"

clean-rust:
	@echo "🧹 Cleaning Rust build artifacts..."
	cargo clean

clean-move:
	@echo "🧹 Cleaning Move build artifacts (frameworks and examples)..."
	rm -rf frameworks/*/build/
	rm -rf examples/*/build/

# Rust Targets
build-rust-debug:
	@echo "🔨 Building Rust project (profile: $(RUST_PROFILE_DEFAULT))..."
	cargo build --profile $(RUST_PROFILE_DEFAULT)

build-rust-release:
	@echo "🔨 Building Rust project (profile: $(RUST_PROFILE_RELEASE))..."
	cargo build --profile $(RUST_PROFILE_RELEASE)

fmt-rust:
	@echo "🔍 Checking Rust code formatting..."
	cargo fmt -- --check

lint-rust: rust-clippy rust-machete

rust-clippy:
	@echo "🔍 Running Rust clippy linter..."
	cargo clippy --workspace --all-targets --all-features --tests --benches -- -D warnings

rust-machete:
	@echo "🔍 Checking for unused Rust dependencies with cargo-machete..."
	@if ! command -v cargo-machete &>/dev/null; then \
		echo "Warning: cargo-machete not found. Skipping check. Install with: cargo install cargo-machete --locked --version 0.7.0"; \
	else \
		cargo machete; \
	fi

test-rust-unit:
	@echo "🧪 Running Rust unit tests with cargo nextest (profile: $(RUST_PROFILE_RELEASE))..."
	export RUST_BACKTRACE=1; \
	cargo nextest run \
		--workspace \
		--all-features \
		--exclude rooch-framework-tests \
		--exclude rooch-integration-test-runner \
		--exclude testsuite \
		-j 8 \
		--retries 2 \
		--success-output final \
		--failure-output immediate-final \
		--cargo-profile $(RUST_PROFILE_RELEASE)

test-rust-integration:
	@echo "🧪 Running specific Rust framework and integration tests (profile: $(RUST_PROFILE_RELEASE))..."
	# Ensure rooch-framework-tests and rooch-integration-test-runner are tested
	cargo test --profile $(RUST_PROFILE_RELEASE) -p rooch-framework-tests -p rooch-integration-test-runner -- --test-threads=8
	# Test rooch-framework-tests specifically for bitcoin_test filter as in pr.sh
	cargo test --profile $(RUST_PROFILE_RELEASE) -p rooch-framework-tests bitcoin_test -- --test-threads=8
	@echo "🧪 Running Rust integration tests for testsuite (profile: $(RUST_PROFILE_RELEASE))..."
	RUST_LOG=warn cargo test --profile $(RUST_PROFILE_RELEASE) -p testsuite --test integration

test-rust: build-rust-release test-rust-unit test-rust-integration

# Move Targets
MOVE_FRAMEWORK_PATHS = \
  frameworks/move-stdlib \
  frameworks/moveos-stdlib \
  frameworks/rooch-framework \
  frameworks/bitcoin-move \
  frameworks/rooch-nursery

build-move: move-stdlib move-framework move-bitcoin-framework move-nursery
	@echo "✅ All core Move frameworks built successfully using Rooch CLI."

move-framework:
	@echo "🔨 Building rooch-framework using Rooch CLI..."
	$(ROOCH_CMD) move build -p frameworks/rooch-framework

move-stdlib:
	@echo "🔨 Building moveos-stdlib using Rooch CLI..."
	$(ROOCH_CMD) move build -p frameworks/moveos-stdlib

move-nursery:
	@echo "🔨 Building rooch-nursery using Rooch CLI..."
	$(ROOCH_CMD) move build -p frameworks/rooch-nursery

move-bitcoin-framework:
	@echo "🔨 Building bitcoin-move using Rooch CLI..."
	$(ROOCH_CMD) move build -p frameworks/bitcoin-move


test-move-frameworks:
	@echo "🧪 Running tests for all Move frameworks using Rooch CLI..."
	@for crate_path in $(MOVE_FRAMEWORK_PATHS); do \
		echo "Testing Move framework: $$crate_path"; \
		if [ -n "$(FILTER)" ]; then \
			echo "  Filtering tests with: $(FILTER)"; \
			$(ROOCH_CMD) move test -p $$crate_path -f "$(FILTER)" || exit 1; \
		else \
			$(ROOCH_CMD) move test -p $$crate_path || exit 1; \
		fi \
	done
	@echo "✅ All Move framework tests passed."

test-move-did: # Kept for specific DID testing as in original Makefile
	@echo "🧪 Running Move DID module tests (in rooch-framework) using Rooch CLI..."
	$(ROOCH_CMD) move test -p frameworks/rooch-framework did

move-examples: # Generic build for all examples
	@echo "🔨 Building all Move example projects using Rooch CLI..."
	@for dir in examples/*/; do \
		if [ -f "$$dir/Move.toml" ]; then \
			echo "Building Move example: $$dir"; \
			$(ROOCH_CMD) move build -p "$$dir" || exit 1; \
		fi \
	done
	@echo "✅ All Move examples built successfully (generic build)."

test-move-examples:
	@echo "🧪 Building and running tests for all Move examples using Rooch CLI..."
	@for dir in examples/*/; do \
		if [ -f "$$dir/Move.toml" ]; then \
			name_addr=$$(basename "$$dir"); \
			echo "Building and Testing Move example: $$dir (named address: $$name_addr)"; \
			$(ROOCH_CMD) move build -d -p "$$dir" && \
			$(ROOCH_CMD) move test -p "$$dir"|| exit 1; \
		fi \
	done
	@echo "✅ All Move example tests passed."

test-move: test-move-frameworks

test-integration:
	@echo "🧪 Running Cucumber integration tests (testsuite only, profile: $(RUST_PROFILE_DEFAULT))..."
	@if [ -n "$(FILTER)" ]; then \
		echo "  Filtering tests with: $(FILTER)"; \
		RUST_LOG=warn cargo test --profile $(RUST_PROFILE_DEFAULT) -p testsuite --test integration -- --name $(FILTER); \
	else \
		RUST_LOG=warn cargo test --profile $(RUST_PROFILE_DEFAULT) -p testsuite --test integration; \
	fi

# Overarching targets
build: build-rust-release build-move
	@echo "🎉 All Rust (profile: $(RUST_PROFILE_RELEASE)) and Move components built successfully"

test: test-rust test-move
	@echo "🎉 All Rust and Move tests completed successfully."

lint: fmt-rust lint-rust
	@echo "✅ All linting and formatting checks passed."

check: lint

all: build test lint
	@echo "🎉✅🎉 Project All: Build, Test, Lint completed successfully! 🎉✅🎉"

ci-checks: lint build test
	@echo "✅ All CI checks (lint, build, test) passed successfully."

# Original verify, dev, check targets adapted
verify:
	@echo "🔍 Verifying Rooch CLI availability and version via cargo run (using $(ROOCH_BINARY_BUILD_PROFILE) profile)..."
	@$(ROOCH_CMD) --version || (echo "❌ Rooch CLI not runnable. Ensure Rust project builds and rooch binary is accessible." && exit 1)
	@echo "✅ Rooch CLI verified"

dev: clean-all build test
	@echo "🎉 Development cycle (clean, build, test) completed successfully."

quick-check: build-rust-debug move-framework
	@echo "✅ Quick compilation check for Rust (debug) and rooch-framework (Move) passed."

# Generate genesis files for mainnet and testnet
generate-genesis:
	@echo "🌱 Generating genesis files for Mainnet and Testnet using $(ROOCH_BINARY_BUILD_PROFILE) profile..."
	@if [ ! -f "$(ROOCH_GENESIS_BIN)" ]; then \
		echo "Error: rooch-genesis binary not found at $(ROOCH_GENESIS_BIN)"; \
		echo "Please build it first with: cargo build --profile $(ROOCH_BINARY_BUILD_PROFILE) --bin rooch-genesis"; \
		exit 1; \
	fi
	@echo "Generating mainnet genesis..."
	@$(ROOCH_GENESIS_BIN) -n main
	@echo "Generating testnet genesis..."
	@$(ROOCH_GENESIS_BIN) -n test
	@echo "✅ Genesis files generated successfully."

# Preserve the old `check` target if it was just framework build for quick syntax
# This is now covered by quick-check or specific move-framework target.
# The original 'clean' only removed Move artifacts and build/, now use clean-all, clean-rust, clean-move.
# The original 'framework' is move-framework.
# The original 'stdlib' is move-stdlib.
# The original 'test' only tested rooch-framework, now test-move-frameworks is more comprehensive.
# The original 'examples' target is move-examples.

# Install required cargo tools
install-tools:
	@echo "🔧 Installing required cargo tools..."
	cargo install cargo-machete --locked --version 0.7.0
	cargo install cargo-nextest --locked --version 0.9.97-b.2
	@echo "✅ All required cargo tools are installed"
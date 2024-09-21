SHELL := /bin/bash

PROJECT_NAME := generator
WASM_TARGET := wasm32-unknown-unknown

.PHONY: all clean unit-test e2e-test test build release install-deps

all: clean build

install-deps:
	@echo "Installing dependencies..."
	@rustup target add $(WASM_TARGET)
	@cargo install wasm-pack
	@cargo install wasm-snip
# The following line assumes you have binaryen installed or available in your package manager
# For example, on Ubuntu you might use: sudo apt-get install binaryen
# On macOS with Homebrew: brew install binaryen
# Please adjust the command according to your package manager or install binaryen manually
# sudo apt-get install binaryen

build:
	@echo "Building WASM module..."
	@wasm-pack build --features debug
	@wasm-opt -Os -o pkg/$(PROJECT_NAME)_bg.wasm pkg/$(PROJECT_NAME)_bg.wasm
	@wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code --skip-producers-section pkg/$(PROJECT_NAME)_bg.wasm -o pkg/$(PROJECT_NAME)_bg.wasm

release:
	@echo "Building WASM module..."
	@wasm-pack build --release --no-default-features
	@wasm-opt -Os -o pkg/$(PROJECT_NAME)_bg.wasm pkg/$(PROJECT_NAME)_bg.wasm
	@wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code --skip-producers-section pkg/$(PROJECT_NAME)_bg.wasm -o pkg/$(PROJECT_NAME)_bg.wasm

unit-test: build
	@cargo test

e2e-test: build
	yarn test

test: unit-test e2e-test

clean:
	@echo "Cleaning up..."
	@rm -rf pkg target

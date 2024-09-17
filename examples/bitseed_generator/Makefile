.PHONY: all clean build test optimize

all: build

build:
	cargo wasm --target-dir="./target"

test:
	cargo test

optimize: build
	@wasm-opt -Os --signext-lowering "./target/wasm32-unknown-unknown/release/bitseed_generator.wasm" -o "./target/wasm32-unknown-unknown/release/bitseed_generator_opt.wasm"
	@wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code --skip-producers-section ./target/wasm32-unknown-unknown/release/bitseed_generator_opt.wasm -o ./target/wasm32-unknown-unknown/release/bitseed_generator_snip.wasm

clean:
	rm -rf ./artifacts && rm -rf ./target


.PHONY: all clean build test optimize

all: build

build:
	cargo wasm --target-dir="./target"

test:
	cargo test

optimize: build
	@mkdir -p ./artifacts
	@wasm-opt -Os --signext-lowering "./target/wasm32-unknown-unknown/release/bitseed_generator.wasm" -o "./artifacts/bitseed_generator_opt.wasm"
	@wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code --skip-producers-section ./artifacts/bitseed_generator_opt.wasm -o ./artifacts/bitseed_generator_snip.wasm

clean:
	rm -rf ./artifacts && rm -rf ./target

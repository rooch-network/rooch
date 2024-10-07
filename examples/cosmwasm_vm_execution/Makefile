.PHONY: all clean build test optimize

all: build

build:
	cargo wasm --target-dir="./target"

test:
	cargo test

optimize: build
	@mkdir -p ./artifacts
	@wasm-opt -Os --signext-lowering "./target/wasm32-unknown-unknown/release/cosmwasm_vm_execution.wasm" -o "./artifacts/cosmwasm_vm_execution_opt.wasm"
	@wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code --skip-producers-section ./artifacts/cosmwasm_vm_execution_opt.wasm -o ./artifacts/cosmwasm_vm_execution_snip.wasm

clean:
	rm -rf ./artifacts && rm -rf ./target

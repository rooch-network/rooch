.PHONY: all clean build test optimize

all: build

build:
	cargo wasm --target-dir="./target"

test:
	cargo test

optimize: build
	docker run -u 1000 --rm -v "$(CURDIR)":/code \
		--entrypoint /code/scripts/optimize.sh \
		cosmwasm/optimizer:0.16.0 .

clean:
	rm -rf ./artifacts && rm -rf ./target


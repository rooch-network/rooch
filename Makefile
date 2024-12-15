# BUILDDIR ?= $(CURDIR)/build
BUILD_FLAGS := --tags '$(BUILD_TAGS)' --ldflags '$(LDFLAGS)'
BUILD_ARGS := $(BUILD_ARGS) -o $(BUILDDIR)

DOCKER ?= $(shell which docker)
GIT_ROOT := $(shell git rev-parse --show-toplevel)

test:
	cargo test

# $(BUILDDIR)/:
# 	mkdir -p $(BUILDDIR)/

build:
	cargo build

build-docker:
	$(DOCKER) build --tag baichuan3/rooch  -f ./docker/DockerfileDeb \
		$(shell git rev-parse --show-toplevel)

.PHONY: build build-docker

FROM ubuntu:bionic AS builder

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        git \
        libssl-dev \
        wget \
        pkg-config \
        libclang-dev clang \
        llvm \
        protobuf-compiler; \
    rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_VERSION=1.26.0 \
    RUSTUP_SHA256=0b2f6c8f85a3d02fde2efc0ced4657869d73fccfce59defb4e8d29233116e6db \
    RUST_ARCH=x86_64-unknown-linux-gnu

RUN set -eux; \
    url="https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${RUST_ARCH}/rustup-init"; \
    wget "$url"; \
    echo "${RUSTUP_SHA256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init

ENV RUST_VERSION=1.68.0

RUN set -eux; \
    ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

WORKDIR /rooch
COPY ./ .
RUN cargo build --release

FROM ubuntu:bionic
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl-dev; \
    rm -rf /var/lib/apt/lists/*

ENV RELEASE_PATH="/rooch/target/release"
COPY --from=builder $RELEASE_PATH/rooch \
     /rooch/

CMD ["/rooch/rooch"]

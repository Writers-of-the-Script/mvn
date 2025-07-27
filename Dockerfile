FROM rust:bullseye AS chef

USER root

RUN apt update && apt -y install \
        curl \
        bash \
        clang \
        llvm \
        libncurses-dev \
        libz-dev \
        zstd

RUN rustup default nightly
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y cargo-chef

WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target $(uname -m)-unknown-linux-gnu --recipe-path recipe.json
COPY . .
RUN cargo build --release --target $(uname -m)-unknown-linux-gnu --bin mvn
RUN cp target/$(uname -m)-unknown-linux-gnu/release/mvn /mvn

FROM debian AS runtime
RUN apt update && apt -y install ca-certificates && apt clean && rm -rf /var/cache/apt/archives /var/lib/apt/lists/*
COPY --from=builder /mvn /mvn
EXPOSE 4000
ENTRYPOINT [ "/mvn" ]

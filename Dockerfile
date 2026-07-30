FROM rust:slim AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        libssl-dev \
        pkg-config \
        protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .
RUN cargo build --release --bin mcb

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --home-dir /home/mcb --shell /usr/sbin/nologin mcb

COPY --from=builder /src/target/release/mcb /usr/local/bin/mcb

USER mcb
WORKDIR /home/mcb

ENTRYPOINT ["mcb"]
CMD ["serve", "--stdio"]

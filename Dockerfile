# syntax=docker/dockerfile:1

# shinonome 上で
#   export(DB→jsonl) → generate-zip(CSV) → build(HTML) → tailwind(生成HTML走査) → rsync(www)
# を実行する。
# DB url / 公開URL / rsync 認証は env で注入する（config.rs の env 上書き）。

### 1) build Rust binary

FROM rust:1.88-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --locked -p komadome

### 2) add Tailwind CLI

FROM debian:bookworm-slim AS tailwind
ARG TAILWIND_VERSION=v3.4.17
ADD https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_VERSION}/tailwindcss-linux-x64 /usr/local/bin/tailwindcss
RUN chmod +x /usr/local/bin/tailwindcss

### 3) runtime

FROM debian:bookworm-slim AS runtime
ARG SUPERCRONIC_VERSION=v0.2.33

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates openssh-client rsync tzdata libssl3 curl \
 && curl -fsSLo /usr/local/bin/supercronic \
      "https://github.com/aptible/supercronic/releases/download/${SUPERCRONIC_VERSION}/supercronic-linux-amd64" \
 && chmod +x /usr/local/bin/supercronic \
 && apt-get purge -y curl \
 && apt-get autoremove -y \
 && rm -rf /var/lib/apt/lists/*

ENV TZ=Asia/Tokyo
RUN useradd -m -u 1000 app
WORKDIR /app

## copy executor + Tailwind CLI

COPY --from=builder /build/target/release/komadome /usr/local/bin/komadome
COPY --from=tailwind /usr/local/bin/tailwindcss /usr/local/bin/tailwindcss

## copy other assets

COPY templates /app/templates
COPY contracts /app/contracts
COPY deploy/komadome.production.toml /app/config/komadome.toml
COPY deploy/tailwind /app/tailwind
COPY deploy/crontab /app/crontab
COPY deploy/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY deploy/run-build.sh /usr/local/bin/run-build.sh

RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/run-build.sh \
 && mkdir -p /app/build /app/data \
 && chown -R app:app /app

USER app
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

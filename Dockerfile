FROM clux/muslrust:nightly AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo +nightly chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/macros macros
RUN cargo +nightly chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo +nightly build --release --target x86_64-unknown-linux-musl --bin slimebot

FROM alpine as runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/slimebot /usr/local/bin/slimebot
EXPOSE 443
ENTRYPOINT ["slimebot"]
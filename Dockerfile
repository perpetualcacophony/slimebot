FROM clux/muslrust:nightly AS chef
USER root
RUN cargo +nightly install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo +nightly chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo +nightly chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo +nightly build --release --target x86_64-unknown-linux-musl

FROM alpine as runtime
COPY --link --from=builder /app/target/x86_64-unknown-linux-musl/release/slimebot /usr/local/bin/slimebot
EXPOSE 443
ENTRYPOINT ["slimebot"]
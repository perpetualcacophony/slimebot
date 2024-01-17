FROM rust as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev libssl-dev
RUN update-ca-certificates

WORKDIR /build
COPY . .

RUN cargo build --target x86_64-unknown-linux-musl --release


FROM alpine

COPY --link --from=builder /build/target/x86_64-unknown-linux-musl/release/slimebot /usr/local/bin/slimebot

EXPOSE 443

ENTRYPOINT ["slimebot"]
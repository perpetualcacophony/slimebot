# rust with musl development utilities & cargo chef preinstalled
FROM ghcr.io/perpetualcacophony/muslrust-chef:nightly AS chef
WORKDIR /build

# add rustup musl target
RUN rustup target add x86_64-unknown-linux-musl



FROM chef AS planner

# cargo chef prepare wants these files
COPY Cargo.toml .
COPY src/main.rs src/

# create the cargo chef recipe file
RUN cargo chef prepare --recipe-path recipe.json



FROM chef AS builder

# copy recipe file to builder
COPY --link --from=planner /build/recipe.json .

# cargo chef cook to cache dependencies from recipe file
RUN cargo +nightly chef cook \
    --release \
    --target x86_64-unknown-linux-musl \
    --recipe-path recipe.json \
    --features docker

# copy the rest of the source code to builder
COPY . .

# touch the build script to ensure cargo runs it
RUN touch build.rs

# build binary
RUN cargo +nightly build \
    --release \
    --target x86_64-unknown-linux-musl \
    --features docker

    

# using alpine for small final image
FROM alpine AS runtime

# add slimebot user & group
RUN addgroup --system slimebot && \
    adduser --system slimebot --ingroup slimebot

EXPOSE 443

# copy binary from builder
COPY --link --from=builder --chown=slimebot:slimebot /build/target/x86_64-unknown-linux-musl/release/slimebot /usr/local/bin/slimebot

USER slimebot:slimebot

ENTRYPOINT slimebot
FROM --platform=linux/amd64 rust:1.82-slim@sha256:1111c28d995d06a7863ba6cea3b3dcb87bebe65af8ec5517caaf2c8c26f38010 AS builder
WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new mochi

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/mochi/

# Set the working directory
WORKDIR /usr/src/mochi

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the sources
COPY src /usr/src/mochi/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/mochi/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM --platform=linux/amd64 alpine:3.20@sha256:1e42bbe2508154c9126d48c2b8a75420c3544343bf86fd041fb7527e017a4b4a

WORKDIR /usr/local/bin

EXPOSE 3000

COPY --from=builder /usr/src/mochi/target/x86_64-unknown-linux-musl/release/mochi ./

CMD ["mochi"]

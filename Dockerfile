FROM --platform=linux/amd64 rust:1.74-slim@sha256:3aee2d2979c2ddb4de910a38f1e91ca162475a67754fb07e2bb74661fd859efe AS builder
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

FROM --platform=linux/amd64 alpine:3.18@sha256:eece025e432126ce23f223450a0326fbebde39cdf496a85d8c016293fc851978

WORKDIR /usr/local/bin

EXPOSE 3000

COPY --from=builder /usr/src/mochi/target/x86_64-unknown-linux-musl/release/mochi ./

CMD ["mochi"]

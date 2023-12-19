FROM --platform=linux/amd64 rust:1.74-slim@sha256:53596c66027523b2289c6e7c96bff119416be22d2cf52734b4962e13371c54cf AS builder
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

FROM --platform=linux/amd64 alpine:3.18@sha256:34871e7290500828b39e22294660bee86d966bc0017544e848dd9a255cdf59e0

WORKDIR /usr/local/bin

EXPOSE 3000

COPY --from=builder /usr/src/mochi/target/x86_64-unknown-linux-musl/release/mochi ./

CMD ["mochi"]

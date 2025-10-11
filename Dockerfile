FROM --platform=linux/amd64 rust:1.90-alpine3.20@sha256:e55d285331a8b14d0cd56a93bf6184891c99a811285c9bc554511feaa8f30f33 AS builder

RUN apk add --no-cache clang lld musl-dev pkgconf openssl-dev openssl-libs-static

WORKDIR /usr/src
RUN USER=root cargo new mochi

COPY Cargo.toml Cargo.lock /usr/src/mochi/
WORKDIR /usr/src/mochi

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

COPY src /usr/src/mochi/src/
## Touch main.rs to prevent cached release build
RUN touch /usr/src/mochi/src/main.rs

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM --platform=linux/amd64 alpine:3.22@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/mochi/target/x86_64-unknown-linux-musl/release/mochi ./
EXPOSE 3000
CMD ["mochi"]

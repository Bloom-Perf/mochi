FROM --platform=linux/amd64 rust:1.85-alpine3.20@sha256:4ec3fedc5eff0bc1ca3ce0e2abbd7190585627eb2b3eedc25e0bf0712b209cd7 AS builder

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

FROM --platform=linux/amd64 alpine:3.21@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/mochi/target/x86_64-unknown-linux-musl/release/mochi ./
EXPOSE 3000
CMD ["mochi"]

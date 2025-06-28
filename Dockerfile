FROM --platform=linux/amd64 rust:1.88-alpine3.20@sha256:3ab9b805c8d2444f6f63f1ae7a38b5e04949d6b0d9e8a314e1ee3ad502deae45 AS builder

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

FROM --platform=linux/amd64 alpine:3.22@sha256:8a1f59ffb675680d47db6337b49d22281a139e9d709335b492be023728e11715

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/mochi/target/x86_64-unknown-linux-musl/release/mochi ./
EXPOSE 3000
CMD ["mochi"]

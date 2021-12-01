FROM rust:alpine as builder

WORKDIR /app
RUN apk add openssl openssl-dev libssh2 musl-dev pkgconfig
COPY . .
RUN cargo build --release

FROM alpine 
RUN apk add openssh libssh2 go 
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
CMD ["backend"]

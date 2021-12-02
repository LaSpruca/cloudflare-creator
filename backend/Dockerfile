FROM rust:alpine as builder

WORKDIR /app
RUN apk add openssl openssl-dev libssh2 musl-dev pkgconfig
COPY . .
RUN cargo build --release

FROM alpine 
RUN apk add openssh libssh2 go
WORKDIR /app
COPY --from=builder /app/log4rs.yaml .
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
EXPOSE 8080
CMD ["backend"]

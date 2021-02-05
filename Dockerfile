FROM rust:buster AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:buster

COPY --from=builder /app/target/release/netloc /usr/local/bin/netloc

CMD [ "netloc" ]

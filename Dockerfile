FROM rust:buster AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:buster

RUN apt-get update && apt-get install -y \
  libssl1.1 \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/netloc /usr/local/bin/netloc

RUN netloc --version

CMD [ "netloc" ]

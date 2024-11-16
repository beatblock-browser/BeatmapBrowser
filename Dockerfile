FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin backend

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y nginx curl && apt-get clean && rm -rf /var/lib/apt/lists/*

FROM surrealdb/surrealdb:latest

COPY config/nginx.conf /etc/nginx/nginx.conf

# HTTP ports
EXPOSE 80 440

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

COPY scripts/setup.sh /usr/local/bin/setup.sh
ENTRYPOINT ["/usr/local/bin/setup.sh"]
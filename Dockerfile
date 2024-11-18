FROM rust:latest AS builder

WORKDIR /app
COPY . .

RUN git submodule sync && git submodule update --recursive
RUN cargo build --release --bin backend

FROM debian:bookworm-slim AS bullseye
RUN apt-get update && apt-get install -y nginx curl bash libssl-dev && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY config/nginx.conf /etc/nginx/nginx.conf

# HTTP ports
EXPOSE 80 440

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

COPY --from=builder /app/scripts/ /usr/local/bin/
COPY --from=builder /app/config/ /usr/local/config/
RUN chmod +x "/usr/local/bin/setup.sh"
ENTRYPOINT ["/usr/local/bin/setup.sh"]
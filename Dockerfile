FROM rust:latest AS builder

WORKDIR /app
COPY . .

ARG PORT=443
ENV PORT $PORT
RUN echo "Running on port $PORT"

RUN rm -rf site
RUN rm -rf oneclick

RUN git clone https://github.com/beatblock-browser/BeatblockBrowser.git site
RUN git clone https://github.com/BigBadE/Beatblock-Oneclick.git oneclick

RUN cargo build --release --bin backend

FROM debian:bookworm-slim AS bullseye
RUN apt-get update && apt-get install -y nginx curl bash libssl-dev && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY config/nginx.conf /etc/nginx/nginx.conf

EXPOSE $PORT

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# Copy builder data
COPY --from=builder /app/scripts/ /usr/local/bin/
COPY --from=builder /app/config/ /usr/local/config/
RUN echo "PORT=$PORT\nDOMAIN=$DOMAIN\nSITE_CERT=$SITE_CERT\nSITE_KEY=$SITE_KEY\n" > /usr/local/config/config.config
RUN cat /usr/local/config/config.config

RUN chmod +x "/usr/local/bin/setup.sh"

ENTRYPOINT ["/usr/local/bin/setup.sh"]
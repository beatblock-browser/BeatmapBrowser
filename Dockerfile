FROM rust:1.82
FROM surrealdb/surrealdb:latest
FROM nginx:1.27.2

COPY scripts /scripts
COPY config /config
COPY backend /backend
COPY site /site
COPY oneclick /oneclick
COPY Cargo.toml Cargo.toml

CMD ["scripts/setup.sh"]
CMD ["cargo", "run", "--bin", "backend", "127.0.0.1:3000", "${DISCORD_TOKEN}"]
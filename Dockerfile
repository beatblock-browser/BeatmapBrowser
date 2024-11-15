FROM rust:1.82
FROM surrealdb:latest

COPY scripts /scripts
COPY config /config
COPY backend /backend
COPY site /site
COPY oneclick /oneclick
COPY Cargo.toml Cargo.toml

CMD ["scripts/install.sh"]
CMD ["cargo", "run", "--bin", "backend", "127.0.0.1:3000", "${DISCORD_TOKEN}"]
FROM rust:buster as chef
RUN cargo install cargo-chef
WORKDIR app

FROM rust:buster AS runtime
RUN apt-get update &&\
    apt-get install -y curl postgresql supervisor
RUN pg_dropcluster $(pg_lsclusters -h | cut -d' ' -f-2 | head -n1)

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json
COPY . .
RUN cargo build --bin api

FROM runtime
RUN echo "[patch.crates-io]\nunveil-service = { path = \"/app/service\" }" > $CARGO_HOME/config.toml
COPY --from=builder /app/target/debug/api /usr/local/bin/unveil-backend
COPY --from=builder /app/service /app/service
COPY docker/entrypoint.sh /bin/entrypoint.sh
COPY docker/supervisord.conf /usr/share/supervisord/supervisord.conf
ENTRYPOINT ["/bin/entrypoint.sh"]

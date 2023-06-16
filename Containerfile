#syntax=docker/dockerfile-upstream:1.4


# Base image
ARG RUSTUP_TOOLCHAIN
FROM docker.io/library/rust:${RUSTUP_TOOLCHAIN}-buster as shuttle-build
RUN apt update && apt install -y curl
RUN cargo install cargo-chef --locked
WORKDIR /build


# Stores source cache
FROM shuttle-build as cache
ARG CARGO_PROFILE
WORKDIR /src
COPY . .
RUN find ${SRC_CRATES} \( -name "*.proto" -or -name "*.rs" -or -name "*.toml" -or -name "Cargo.lock" -or -name "README.md" -or -name "*.sql" \) -type f -exec install -D \{\} /build/\{\} \;
# This is used to carry over in the docker images any *.pem files from shuttle root directory,
# to be used for TLS testing, as described here in the admin README.md.
RUN [ "$CARGO_PROFILE" != "release" ] && \
    find ${SRC_CRATES} -name "*.pem" -type f -exec install -D \{\} /build/\{\} \;


# Stores cargo chef recipe
FROM shuttle-build AS planner
COPY --from=cache /build .
RUN cargo chef prepare --recipe-path recipe.json


# Stores cargo chef recipe
FROM shuttle-build AS builder
ARG CARGO_PROFILE
ARG folder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook \
    # if CARGO_PROFILE is release, pass --release, else use default debug profile
    $(if [ "$CARGO_PROFILE" = "release" ]; then echo --release; fi) \
    --recipe-path recipe.json
COPY --from=cache /build .
RUN cargo build --bin shuttle-${folder} \
    $(if [ "$CARGO_PROFILE" = "release" ]; then echo --release; fi)


# Middle step
ARG RUSTUP_TOOLCHAIN
FROM rust:${RUSTUP_TOOLCHAIN}-buster as shuttle-common
RUN rustup component add rust-src
COPY --from=cache /build /usr/src/shuttle/


# The final image for this shuttle-* crate
FROM shuttle-common as shuttle-crate
ARG folder
ARG prepare_args
# used as env variable in prepare script
ARG PROD
ARG CARGO_PROFILE
ARG RUSTUP_TOOLCHAIN
ENV RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN}
COPY ${folder}/prepare.sh /prepare.sh
RUN /prepare.sh "${prepare_args}"
COPY --from=builder /build/target/${CARGO_PROFILE}/shuttle-${folder} /usr/local/bin/service
ENTRYPOINT ["/usr/local/bin/service"]

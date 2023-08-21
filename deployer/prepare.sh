#!/bin/bash

###############################################################################
# This file is used by our common Containerfile incase the container for this #
# service might need some extra preparation steps for its final image         #
###############################################################################

# Stuff that depends on local source files
if [ "$1" = "--after-src" ]; then
    # Install the shuttle-next runtime for shuttle-next services.
    cargo install shuttle-runtime --path "/usr/src/shuttle/runtime" --bin shuttle-next --features next || exit 1

    while getopts "p," o; do
    case $o in
        "p") # if panamax is used, the '-p' parameter is passed
            # Make future crates requests to our own mirror
            echo '
[source.shuttle-crates-io-mirror]
registry = "sparse+http://panamax:8080/index/"
[source.crates-io]
replace-with = "shuttle-crates-io-mirror"' >> $CARGO_HOME/config.toml
                ;;
            *)
                ;;
        esac
    done
    exit 0
fi

# Patch crates to be on same versions
mkdir -p $CARGO_HOME
touch $CARGO_HOME/config.toml
if [[ $PROD != "true" ]]; then
    echo '
    [patch.crates-io]
    shuttle-service = { path = "/usr/src/shuttle/service" }
    shuttle-runtime = { path = "/usr/src/shuttle/runtime" }

    shuttle-aws-rds = { path = "/usr/src/shuttle/resources/aws-rds" }
    shuttle-persist = { path = "/usr/src/shuttle/resources/persist" }
    shuttle-shared-db = { path = "/usr/src/shuttle/resources/shared-db" }
    shuttle-secrets = { path = "/usr/src/shuttle/resources/secrets" }
    shuttle-static-folder = { path = "/usr/src/shuttle/resources/static-folder" }
    shuttle-service-info = { path = "/usr/src/shuttle/resources/service-info" }
    shuttle-turso = { path = "/usr/src/shuttle/resources/turso" }

    shuttle-actix-web = { path = "/usr/src/shuttle/services/shuttle-actix-web" }
    shuttle-axum = { path = "/usr/src/shuttle/services/shuttle-axum" }
    shuttle-next = { path = "/usr/src/shuttle/services/shuttle-next" }
    shuttle-poem = { path = "/usr/src/shuttle/services/shuttle-poem" }
    shuttle-poise = { path = "/usr/src/shuttle/services/shuttle-poise" }
    shuttle-rocket = { path = "/usr/src/shuttle/services/shuttle-rocket" }
    shuttle-salvo = { path = "/usr/src/shuttle/services/shuttle-salvo" }
    shuttle-serenity = { path = "/usr/src/shuttle/services/shuttle-serenity" }
    shuttle-thruster = { path = "/usr/src/shuttle/services/shuttle-thruster" }
    shuttle-tide = { path = "/usr/src/shuttle/services/shuttle-tide" }
    shuttle-tower = { path = "/usr/src/shuttle/services/shuttle-tower" }
    shuttle-warp = { path = "/usr/src/shuttle/services/shuttle-warp" }' > $CARGO_HOME/config.toml
fi

# Add the wasm32-wasi target
rustup target add wasm32-wasi

# Install common build tools for external crates
# The image should already have these: https://github.com/docker-library/buildpack-deps/blob/65d69325ad741cea6dee20781c1faaab2e003d87/debian/buster/Dockerfile
apt update
apt install -y curl llvm-dev libclang-dev clang cmake

# Install protoc since some users may need it
ARCH="linux-x86_64" && \
VERSION="22.2" && \
curl -OL "https://github.com/protocolbuffers/protobuf/releases/download/v$VERSION/protoc-$VERSION-$ARCH.zip" && \
    unzip -o "protoc-$VERSION-$ARCH.zip" bin/protoc "include/*" -d /usr/local && \
    rm -f "protoc-$VERSION-$ARCH.zip"

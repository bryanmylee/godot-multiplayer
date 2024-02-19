# syntax=docker/dockerfile:1
# Build this Dockerfile from the project root `.`, not from `./game-server-manager`.

################################################################################
# Create a stage for building the game server manager.
ARG RUST_VERSION=1.75.0
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME=game-server-manager
WORKDIR /app

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for 
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=${APP_NAME}/src,target=src \
    --mount=type=bind,source=${APP_NAME}/Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=${APP_NAME}/Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/manager
EOF

################################################################################
# Create a stage for building the game server.
FROM bryanmylee/godot:4.2.stable as build-game-server
ARG SERVER_EXPORT_PRESET_NAME=Server

# Mount the project in the image and build the project.
RUN --mount=type=bind,source=project,target=project,readonly \
    <<EOF
set -e
/bin/godot --headless --path project --export-release "${SERVER_EXPORT_PRESET_NAME}" ../game-server
EOF

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application.
FROM debian:bullseye-slim AS final

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the game server manager from the "build" stage.
COPY --from=build /bin/manager /bin/

# Copy the game server from the "build-game-server" stage.
COPY --from=build-game-server /build/game-server ./game-server

# Expose the port that the application listens on.
EXPOSE 8500
EXPOSE 9000-9249

# What the container should run when it is started.
CMD ["/bin/manager"]

# syntax=docker/dockerfile:1

################################################################################
# Create a stage for downloading the engine and templates.
FROM debian:bullseye-slim AS prebuild

WORKDIR /build

ARG GODOT_ENGINE_URL=https://downloads.tuxfamily.org/godotengine/4.2/Godot_v4.2-stable_linux.x86_64.zip
ARG GODOT_EXPORT_TEMPLATES_URL=https://downloads.tuxfamily.org/godotengine/4.2/Godot_v4.2-stable_export_templates.tpz

RUN apt-get update && apt-get -qq -y install curl unzip

# Download the Godot executable.
RUN curl -o godot.zip ${GODOT_ENGINE_URL}

# Download the Godot export templates.
RUN curl -o export_templates.zip ${GODOT_EXPORT_TEMPLATES_URL}

# Unpack Godot executable.
RUN unzip godot.zip -d godot && mv godot/* /bin/godot

# Unpack Godot export templates.
RUN unzip export_templates.zip -d export_templates && mv export_templates/* /export_templates

################################################################################
# Create a stage for building the application.
FROM debian:bullseye-slim as build

ARG GODOT_VERSION=4.2.stable
ARG SERVER_EXPORT_PRESET_NAME=Server

WORKDIR /build

# Copy the engine and export templates from the "prebuild" stage.
COPY --from=prebuild /bin/godot /bin/
COPY --from=prebuild /export_templates /export_templates

# Setup export templates.
RUN mkdir -p $HOME/.local/share/godot/export_templates && \
	mv /export_templates $HOME/.local/share/godot/export_templates/${GODOT_VERSION}

# Mount the project in the image.
RUN --mount=type=bind,source=project,target=project,readonly \
	<<EOF
set -e
/bin/godot --headless --path project --export-release "${SERVER_EXPORT_PRESET_NAME}" ../server
EOF

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
FROM debian:bullseye-slim AS final

WORKDIR /app

# Copy the executable from the "build" stage.
COPY --from=build /build .

# Fix permissions
RUN chmod +x server

# Expose the port that the application listens on.
EXPOSE 9000

# Run the application.
CMD ["./server", "--server", "--headless"]

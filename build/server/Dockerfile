################################################################################
# Create a stage for building the application.
FROM bryanmylee/godot:4.2.stable as build

ARG SERVER_EXPORT_PRESET_NAME=Server

# Mount the project in the image and build the project.
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

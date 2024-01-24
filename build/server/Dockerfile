FROM ubuntu:20.04

WORKDIR /app

# Copy the binaries into the container
COPY . ./

# Fix permissions
RUN chmod +x server.x86_64

# Expose the port that the application listens on.
EXPOSE 9000

# Run the application.
CMD ./server.x86_64 --server --headless

# Stage 1: Build Stage
FROM rust:bookworm as builder

ENV CARGO_TARGET_DIR=/target

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs cmake

# Create and set the working directory
WORKDIR /usr/src/instagen

# Now copy your actual source files
COPY . .

# Install Tailwind CLI
RUN npm install -g tailwindcss

# Build Tailwind CSS
RUN tailwindcss -i static/css/input.css -o static/css/output.css --minify

# Build the application for release
RUN cargo build --release

# Stage 2: Runtime Stage
FROM debian:bookworm-slim

# Install required libraries
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /target/release/instagen /usr/local/bin/instagen

# Copy SSL certificates
COPY ssl/key.pem /etc/ssl/private/key.pem
COPY ssl/cert.pem /etc/ssl/certs/cert.pem

# Copy the static files including the newly built CSS
COPY --from=builder /usr/src/instagen/static /usr/local/bin/static
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Set environment variables
ENV RUST_LOG=debug,tower_http=trace

# Set the working directory
WORKDIR /usr/local/bin

# Run the binary
CMD ["instagen", "--key-file", "/etc/ssl/private/key.pem", "--cert-file", "/etc/ssl/certs/cert.pem"]

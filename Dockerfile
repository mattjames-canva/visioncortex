# Use an official Rust image as a builder
FROM rust:1 as builder

WORKDIR /usr/src/visioncortex

# Copy the manifest file
COPY Cargo.toml .

# Create a dummy lib.rs to leverage Docker's cache.
# This allows us to install dependencies before adding the source code.
RUN mkdir src && echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies
RUN cargo build --release

# Now, copy your actual source code
COPY src ./src

# Build the actual project, this will be fast because dependencies are cached
RUN cargo build --release

# Final stage: a small Debian image
FROM debian:buster-slim

# Copy the compiled libraries from the builder stage
COPY --from=builder /usr/src/visioncortex/target/release/libvisioncortex.so /usr/local/lib/
COPY --from=builder /usr/src/visioncortex/target/release/libvisioncortex.rlib /usr/local/lib/

# This command keeps the container running, useful for debugging or other tasks.
CMD ["tail", "-f", "/dev/null"] 
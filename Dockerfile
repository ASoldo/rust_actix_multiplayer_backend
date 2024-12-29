# ---------- 1) Builder Stage ------------
FROM rustlang/rust:nightly AS builder
WORKDIR /app
# Copy everything into the builder
COPY . .
# Add SQLx CLI for migrations
RUN cargo install sqlx-cli --no-default-features --features postgres
# Copy the SQLx offline cache for query validation
COPY .sqlx .sqlx
ENV SQLX_OFFLINE=true
# Build the release version of the app
RUN cargo build --release

# ---------- 2) Final Runtime Stage ------------
FROM debian:bookworm-slim
# Install needed libraries
RUN apt-get update && apt-get install -y ca-certificates libc6 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/rust-actix-multiplayer-backend ./
# Copy migrations for runtime execution
COPY --from=builder /app/migrations ./migrations
# Also copy the sqlx binary for runtime migration execution
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
# Expose the port for the application
EXPOSE 8080
# Default environment variables
ENV DATABASE_URL=postgres://admin:admin@db:5432/multiplayer_demo
ENV JWT_SECRET=MySuperSecretKey
ENV RUST_LOG=info
# CMD: Run migrations and start the server
CMD ["sh", "-c", "sqlx migrate run && ./rust-actix-multiplayer-backend"]

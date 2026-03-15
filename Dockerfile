# ═══════════════════════════════════════════════════════════════════════════════
# Stage 1: Build frontend
# ═══════════════════════════════════════════════════════════════════════════════
FROM node:22-alpine AS frontend

WORKDIR /app/frontend

# Copy lockfile and package.json first for dependency layer caching.
COPY frontend/package.json frontend/package-lock.json* ./

RUN npm ci

# Copy frontend source and build
COPY frontend/ .

RUN npm run build

# Output: /app/frontend/dist/

# ═══════════════════════════════════════════════════════════════════════════════
# Stage 2: Build backend (fully static musl binary)
# ═══════════════════════════════════════════════════════════════════════════════
FROM rust:1-alpine AS backend

# musl-dev required for static linking against musl libc
RUN apk add --no-cache musl-dev

# Add the musl target explicitly
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# ── Dependency caching layer ─────────────────────────────────────────────────
# Copy only manifests and lockfile first. Docker caches this layer until
# Cargo.toml or Cargo.lock change, so dependency compilation is skipped
# on source-only changes. This saves 5-10 minutes per build.

COPY Cargo.toml Cargo.lock ./
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml
COPY crates/db/Cargo.toml crates/db/Cargo.toml
COPY crates/auth/Cargo.toml crates/auth/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml

# sqlx offline query cache (needed for compile-time checked queries)
COPY crates/db/.sqlx crates/db/.sqlx
COPY crates/api/.sqlx crates/api/.sqlx

# Create dummy source files so cargo can resolve the workspace graph
RUN mkdir -p crates/shared/src crates/db/src crates/auth/src crates/api/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    touch crates/shared/src/lib.rs crates/db/src/lib.rs crates/auth/src/lib.rs crates/api/src/lib.rs

# Build dependencies only against musl target
ENV SQLX_OFFLINE=true
RUN cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null || true

# Remove dummy build artifacts for our crates (keep cached dependencies)
RUN rm -rf crates/*/src && \
    find target/x86_64-unknown-linux-musl/release/deps -name "api*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "shared*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "db*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "auth*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "libapi*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "libshared*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "libdb*" -delete 2>/dev/null; \
    find target/x86_64-unknown-linux-musl/release/deps -name "libauth*" -delete 2>/dev/null; \
    true

# ── Application build ───────────────────────────────────────────────────────
# Copy real source code and migrations
COPY crates/ crates/
COPY migrations/ migrations/

# Build the application (only our code compiles, deps are cached)
# Produces a fully static binary with zero runtime dependencies
RUN cargo build --release --target x86_64-unknown-linux-musl

# ═══════════════════════════════════════════════════════════════════════════════
# Stage 3: Scratch runtime
# ═══════════════════════════════════════════════════════════════════════════════
FROM scratch

# CA certificates for outbound HTTPS (e.g. connecting to RDS over TLS)
COPY --from=backend /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# The fully static binary
COPY --from=backend /app/target/x86_64-unknown-linux-musl/release/api /api

# Built frontend assets served by the Rust backend
COPY --from=frontend /app/frontend/dist /static

# Migrations for potential startup runs
COPY --from=backend /app/migrations /migrations

# Run as non-root (numeric UID since scratch has no /etc/passwd)
USER 1000

EXPOSE 8080

ENV STATIC_DIR=/static

ENTRYPOINT ["/api"]
# Stage 1 is the build environment. It contains rustc, Cargo and the other
# tools needed to compile the API. These tools will not be copied into the
# final image.
FROM rust:1.91-bookworm AS builder

# All following paths in this stage are relative to /app.
WORKDIR /app

# Give slow registry connections more time before retrying. These settings
# apply only to the builder stage, not the running API container.
ENV CARGO_HTTP_TIMEOUT=120 \
    CARGO_NET_RETRY=5

# Cargo needs the workspace manifest, lockfile, and Rust source trees.
# Copying only these paths also keeps frontend files out of this image.
COPY Cargo.toml Cargo.lock ./
COPY bin ./bin
COPY crates ./crates

# --locked guarantees that Cargo uses the exact dependency versions recorded
# in Cargo.lock. --release creates an optimized executable.
# Keep downloaded crates across rebuilds and failed attempts. BuildKit caches
# are not copied into the runtime image.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --locked --release -p lightning-api

# Stage 2 is the runtime environment. It is smaller because it does not contain
# the Rust compiler, Cargo, or the source code.
FROM debian:bookworm-slim AS runtime

# ca-certificates provides trusted roots for outbound HTTPS connections.
# The unprivileged user avoids running the web server as root.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 appuser

# Copy only the compiled executable from the builder stage.
COPY --from=builder /app/target/release/lightning-api /usr/local/bin/lightning-api

USER appuser

# EXPOSE documents the intended container port. Render still supplies the
# actual PORT environment variable, which the Axum process reads at runtime.
EXPOSE 10000

# Exec-form CMD starts the Rust executable as the container's main process.
CMD ["lightning-api"]

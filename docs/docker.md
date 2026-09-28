# Docker deployment

Docker does not run Rust source code directly. Cargo first compiles the source into a native executable, and the container then runs that executable.

## Why this Dockerfile has two stages

```text
Rust source
    |
    v
builder image (Rust + Cargo) --cargo build--> lightning-api executable
                                                   |
                                                   v
runtime image (Debian + executable only) -------> running API
```

The builder image is large because it contains the compiler and build tools. Those tools are useful during compilation but unnecessary in production. The runtime image receives only the compiled `lightning-api` binary and the TLS certificate bundle.

This pattern is called a **multi-stage build**.

## Dockerfile reference

1. `FROM rust:1.91-bookworm AS builder` creates the compilation stage.
2. `WORKDIR /app` selects the working directory for subsequent instructions.
3. The `COPY` instructions place the Cargo workspace inside the image.
4. `cargo build --locked --release -p lightning-api` compiles only the API package.
5. `FROM debian:bookworm-slim AS runtime` starts a fresh, smaller stage.
6. The `RUN` instruction installs trusted CA certificates and creates a non-root user.
7. `COPY --from=builder ...` copies the compiled binary between stages.
8. `USER appuser` ensures the application is not run as root.
9. `EXPOSE 10000` documents the expected production port.
10. `CMD ["lightning-api"]` starts the executable when the container starts.

## Why the whole Rust workspace is copied

The API package depends on the local `invoice-decoder` crate. Cargo therefore needs:

```text
Cargo.toml
Cargo.lock
bin/api/
crates/invoice-decoder/
```

The root `.dockerignore` prevents unrelated build output, frontend dependencies, and local environment files from entering the Docker build context.

## Ports: an important distinction

`EXPOSE 10000` is documentation; it does not force Axum to use that port and does not publish the port to your computer.

Axum reads the `PORT` environment variable and binds to `0.0.0.0`. Binding to `0.0.0.0` matters inside a container because binding only to `127.0.0.1` would make the process unreachable from outside that container.

Build and run the backend container with:

```bash
docker build -t lightening-decoder-api .
docker run --rm -p 3001:10000 \
  -e PORT=10000 \
  -e ALLOWED_ORIGIN=http://localhost:3000 \
  lightening-decoder-api
```

The mapping `3001:10000` means:

```text
localhost:3001 -> container port 10000 -> Axum
```

These commands build the production image and expose its API on
`http://localhost:3001`. Docker is not required when running the API directly
with Cargo.

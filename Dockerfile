# Stage 1: Build Frontend
FROM docker.io/oven/bun:1 AS frontend-builder
WORKDIR /build

# Copy package manifests and install dependencies
COPY package.json bun.lock ./
COPY client/package.json ./client/
RUN bun install --frozen-lockfile

# Copy the rest of the client code and build
COPY client ./client
WORKDIR /build/client
RUN bun run build
# Generate precomputed gzip files for http
RUN find ./dist -type f -exec gzip -k -q {} \;





# Stage 2: Prepare for Backend Build with Cargo Chef
FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1-alpine AS chef
WORKDIR /app

# Install build dependencies needed by chef cook and the final build
RUN apk add --no-cache musl-dev gcc make libc-dev libressl-dev

# install sqlx to generate the database necessary to build sqlx
RUN cargo install sqlx-cli --no-default-features --features sqlite

FROM chef AS planner
WORKDIR /app
# Copy everything needed for planning
COPY . .
# Compute dependencies
RUN cargo chef prepare --recipe-path recipe.json





# Stage 3: Build Backend Dependencies and Application
FROM chef AS builder
WORKDIR /app
# Copy the dependency recipe
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies based on the recipe
# Pass necessary target and features for dependencies
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl --features static

# Copy application code
COPY . .

# sqlx needs database info to typecheck properly.
ENV DATABASE_URL=sqlite:///app/db.sqlite
RUN sqlx database create && sqlx migrate run

# Build the application, linking against the pre-built dependencies.
RUN cargo build --release --target x86_64-unknown-linux-musl --bin lyra --locked --features static





# Stage 4: Final Runtime Image
FROM docker.io/alpine:latest
WORKDIR /app

# ffmpeg>7 is required
RUN apk add --no-cache ffmpeg>7

# Copy frontend build from Stage 1
COPY --from=frontend-builder /build/client/dist/client ./dist

# Copy backend build from Stage 3 (builder)
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/lyra ./lyra

ENV LYRA_STATIC_PATH=/app/dist
ENV LYRA_DATA_DIR=/data
ENV LYRA_FFMPEG_PATH=/usr/bin/ffmpeg
ENV LYRA_FFPROBE_PATH=/usr/bin/ffprobe
ENV LYRA_HOST=0.0.0.0

EXPOSE 8000
CMD ["/app/lyra"]
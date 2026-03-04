# Stage 1: Build frontend assets
FROM docker.io/oven/bun:1 AS frontend-builder
WORKDIR /build

COPY package.json bun.lock ./
COPY client/package.json ./client/
COPY schema.gql ./schema.gql
RUN bun install --frozen-lockfile

COPY client ./client
WORKDIR /build/client
RUN bun run build
# Precompress static assets for ServeDir::precompressed_gzip
RUN find ./dist -type f -exec gzip -k -q {} \;


# Stage 2: Cargo Chef base
FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features sqlite

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


# Stage 3: Build backend
FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --features static

COPY . .

# sqlx macros need a DB URL at compile time.
ENV DATABASE_URL=sqlite:///app/db.sqlite
RUN sqlx database create && sqlx migrate run

RUN cargo build --release --bin lyra --locked --features static
RUN strip target/release/lyra


# Stage 4: Runtime
FROM sylver/lyra-static-ffmpeg:latest AS ffmpeg-runtime

FROM gcr.io/distroless/cc-debian13:nonroot
WORKDIR /app

COPY --from=frontend-builder /build/client/dist ./dist
COPY --from=builder /app/target/release/lyra ./lyra
COPY --from=ffmpeg-runtime /lyra-ffmpeg /usr/local/bin/lyra-ffmpeg
COPY --from=ffmpeg-runtime /lyra-ffprobe /usr/local/bin/lyra-ffprobe

ENV LYRA_STATIC_PATH=/app/dist
ENV LYRA_DATA_DIR=/config
ENV LYRA_HOST=0.0.0.0
ENV LYRA_FFMPEG_PATH=/usr/local/bin/lyra-ffmpeg
ENV LYRA_FFPROBE_PATH=/usr/local/bin/lyra-ffprobe

EXPOSE 8000
CMD ["/app/lyra"]

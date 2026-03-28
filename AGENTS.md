# Lyra

Lyra is an experimental local-media server in the "Plex alternative" space, with a strong focus on efficiency and reliability.

There are no real deployments yet. Breaking changes are acceptable, resetting the database is acceptable, and compatibility layers are usually not worth keeping. When refactoring, prefer removing old code over carrying dead paths forward.

## Core Principles

- Use GraphQL for app data and HLS for playback.
- When writing SQL, either use `sea_query` or use sqlx macros for validation. Do not use sea_querys raw SQL support, as it opts us out of compile-time validation.
- Use bun over npm, "bunx" vs "npx" etc
- Keep clients thin. If complexity has to exist, prefer putting it on the server.
  - Some exceptions to this exist, mainly for features that will only ever exist in the web client (settings, plex imports, etc).

## Architecture

- `crates/lyra` is the main server. It owns HTTP, GraphQL, scanning, metadata, jobs, assets, and HLS endpoints.
- `client/` is the web client. It talks to the server over GraphQL for application data and HLS for playback.
- The workspace is split into focused crates for parsing, metadata, probing, packaging, thumbnails, timeline previews, and related media tasks.

Important supporting crates:

- `crates/lyra-parser`: parses file paths into structured media information.
- `crates/lyra-packager`: builds HLS package state and playlists from media analysis.
- `crates/lyra-ffprobe`: ffmpeg/ffprobe integration and media probing helpers.
- `crates/lyra-thumbnail`: representative thumbnail generation.
- `crates/lyra-timeline-preview`: timeline preview sprite generation.
- `crates/lyra-metadata` and `crates/lyra-metadata-tmdb`: remote metadata interfaces and providers.

## Domain Model

- A `Library` is a configured filesystem root.
- A `File` is a discovered media file on disk.
- A `Root` is the top-level title grouping inside a library, usually a movie or a series.
- A `Season` is an optional grouping under a series root.
- An `Item` is the playable unit, usually a movie or episode.
- "Nodes" generally refers to `RootNode`, `SeasonNode`, and `ItemNode`.
- `item_files` links items to one or more physical files.
- Nodes are derived from files during import rather than from external metadata. This lets unmatched media still appear in the UI, keeps it visible if metadata resolution fails, and gives playback and UI flows stable targets.
- Metadata has two broad layers:
  - `local`: derived from filenames, directory structure, and file-local analysis.
  - `remote`: enriched from external providers.
- The GraphQL API uses explicit node types such as `RootNode`, `SeasonNode`, and `ItemNode` rather than a generic node abstraction.
- IDs are deterministic so rescans preserve grouping and references.
- The UI generally browses and queries nodes. Files usually matter only once playback or file-specific analysis is needed.

## Data Flow

1. Scan libraries and upsert discovered files.
2. Parse file paths into structured media hints.
3. Derive roots, seasons, items, and item-file links.
4. Store local metadata.
5. Run background jobs for enrichment such as remote metadata, probe data, thumbnails, and timeline previews.
6. Resolve playback from item to file to on-demand HLS packaging and segment generation.

## Where To Look

- Scanner and media derivation: `crates/lyra/src/scanner/`
- GraphQL schema and node types: `crates/lyra/src/graphql/`
- Background job runtime and handlers: `crates/lyra/src/jobs/`
- Metadata matching and storage: `crates/lyra/src/metadata/`
- HLS serving: `crates/lyra/src/hls/`
- Packaging logic: `crates/lyra-packager/`

## Comments

- Add a short comment before complex or non-obvious functions and code paths, especially when handling edge cases or choosing a less obvious approach on purpose.
- Comments should explain the invariant, constraint, or reason the code works this way, not restate the mechanics line by line.
- Keep comments brief and maintain them when behavior changes. Stale comments are worse than no comments.
- Never remove existing comments.

## Keep This File Short

- Repo-level `AGENTS.md` should capture stable concepts, invariants, architecture, and navigation hints.
- Do not put implementation details here unless they are critical invariants.
- UI flows, route shapes, query field lists, thresholds, and one-off behavior usually belong in code, comments, tests, or a more scoped `AGENTS.md`.
- If a note would become misleading after a normal feature change, it probably does not belong in the repo-wide file.
- If you learn something durable and non-obvious that will save future readers time, add it concisely.

# lyra

> [!WARNING]  
> lyra is very experimental and early in development. Expect things to not work, be buggy, and change often. If you're not comfortable with that, it's probably best to wait until it's a bit more mature. You should probably just come back in a few months.

lyra is a self-hosted media server that lets you browse and watch your media with minimal fuss.

## features

- [x] Automatically skip intros
- [x] Synced watch sessions
- [x] Timeline previews
- [x] Watch state tracking
- [x] Plex watch state import
- [x] A lot of bugs

## running

lyra only runs under docker for now. At some point that will change and this section will be better, but for now, here is a sample docker compose file:

```yaml
services:
  lyra:
    container_name: lyra
    image: docker.io/sylver/lyra
    restart: unless-stopped
    environment:
      - PUID=1000
      - GUID=1000
    ports:
      - 8000:8000
    volumes:
      - /mnt/media/series:/mnt/media/series:ro
      - /mnt/media/movies:/mnt/media/movies:ro
      - ./.lyra:/config
```

There is no hardware transcoding for the time being, it adds a lot of complexity that I want to avoid while getting the basics working.

## development

lyra uses rust + async-graphql + sqlx + sea-orm for the server and bun + react + apollo for the client.

For dev you'll need rust, bun, docker, and the custom ffmpeg binaries. The easiest way to get the ffmpeg binaries is:

```sh
./scripts/export-ffmpeg.sh
```

That copies `lyra-ffmpeg` and `lyra-ffprobe` into `./bin`, which is where debug builds look for them by default.

To start the server, from the repo root do:

```sh
cargo run
```

Then for the client, in `client/`:

```sh
bun install
bun run watch
```

That starts vite and proxies `/api` to the Rust server. In production it flips around and the Rust server serves the built client as an SPA for non-API routes.

The server will create `.lyra/` locally for data and run sqlite migrations on startup.

Most of the HLS packaging logic lives in `crates/lyra-packager`. The workspace is split into smaller crates mostly so media-specific pieces can be tested and worked on in isolation. Most crates will have a bin for testing and development, such as `lyra-timeline-preview` that has a bin that dumps generated preview images for a file path.

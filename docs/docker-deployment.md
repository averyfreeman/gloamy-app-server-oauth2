# Running the Gloamy Daemon in Docker

This repository's production Compose stack runs the daemon only. The web and
desktop clients are maintained separately and connect to the gateway API.

## Quick Start

Requires Docker Compose v2.24 or newer.

```bash
cp .env.example .env
```

Set at least one provider API key in `.env`, then start the daemon:

```bash
docker compose up -d --build
docker compose logs -f gloamy
```

The gateway API is published at `http://127.0.0.1:42617`. On first start, the
daemon prints a one-time pairing code. Enter it in a Gloamy client.

## Ports and Exposure

| Service | Container | Host | Notes |
|---|---:|---|---|
| `gloamy` | 42617 | `127.0.0.1:42617` | Gateway REST/SSE API |

The gateway binds to the container interface so Docker can forward the
published port. Host exposure remains limited to loopback. Do not change the
host mapping to `0.0.0.0` unless the gateway is intentionally protected behind
a trusted network boundary.

## Configuration and Operations

Compose passes `.env` to the daemon. All mutable data lives under `$HOME`, which
the image sets to `/gloamy-data` and Compose backs with the `gloamy-data` named
volume:

- `/gloamy-data/.gloamy/` — `config.toml`, memory, and secret store
- `/gloamy-data/workspace/` — agent working files

Common commands:

```bash
docker compose ps
docker compose logs -f gloamy
docker compose exec gloamy gloamy doctor
docker compose restart gloamy
docker compose down       # keep the data volume
docker compose down -v    # delete all agent state
```

The image entrypoint is `gloamy`, so any CLI subcommand can be run as a
one-shot:

```bash
docker compose run --rm gloamy status
docker run --rm gloamy:local --version
```

## Image Targets

```bash
docker build --target release -t gloamy:local .
docker build --target dev -t gloamy:dev .
```

Both stages run as uid 1000 and are non-root. The `dev` stage adds shell tools
for development; the `release` stage stays smaller.

## Clients

- [Gloamy Web UI](https://github.com/averyfreeman/gloamy-fork-ui) — deployment
  and reverse-proxy instructions for the browser client.
- [Gloamy Desktop](https://github.com/averyfreeman/gloamy-fork-desktop-ui) — a
  native client that can start or attach to a local daemon.

For safe non-loopback gateway exposure, see [network-deployment.md](network-deployment.md).
For daemon operations, see [operations-runbook.md](operations-runbook.md).

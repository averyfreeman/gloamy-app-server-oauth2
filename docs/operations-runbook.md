# Gloamy Operations Runbook

This runbook is for operators who maintain availability, security posture, and incident response.

Last verified: **November 9, 2026**.

## Scope

Use this document for day-2 operations:

- starting and supervising runtime
- health checks and diagnostics
- safe rollout and rollback
- incident triage and recovery

For first-time installation, start from [one-click-bootstrap.md](one-click-bootstrap.md).

## Runtime Modes

| Mode | Command | When to use |
|---|---|---|
| Foreground runtime | `gloamy daemon` | local debugging, short-lived sessions |
| Foreground gateway only | `gloamy gateway` | webhook/API endpoint testing and web app connectivity |
| User service | `gloamy service install && gloamy service start` | persistent operator-managed runtime |
| macOS desktop app | Open Gloamy | local Vue UI with a daemon started or attached by the app |

### macOS Desktop Daemon

The Tauri desktop app starts the bundled daemon when no local Gloamy daemon is
available. It attaches to a healthy existing daemon without taking ownership.
The Settings screen's **Keep daemon running after closing** preference defaults
to Off. Off requests graceful shutdown only for a daemon marked as desktop-managed;
On leaves that daemon available to the browser UI. An attached unmanaged daemon
is never stopped by closing the desktop app.

Pairing recovery is available from Authentication or Settings in the desktop
app. It requests a fresh, single-use code over the same-user local control socket
and completes pairing through the existing `/pair` endpoint. Existing paired
devices remain valid. The code is not written to configuration or logs, and
there is no HTTP endpoint for generating one. See
[Desktop control socket security](security/desktop-control-socket.md).

## Baseline Operator Checklist

1. Validate configuration:

```bash
gloamy status
```

2. Verify diagnostics:

```bash
gloamy doctor
gloamy channel doctor
```

3. Start runtime:

```bash
gloamy daemon
```

4. For persistent user session service:

```bash
gloamy service install
gloamy service start
gloamy service status
```

## Health and State Signals

| Signal | Command / File | Expected |
|---|---|---|
| Config validity | `gloamy doctor` | no critical errors |
| Channel connectivity | `gloamy channel doctor` | configured channels healthy |
| Runtime summary | `gloamy status` | expected provider/model/channels |
| Daemon heartbeat/state | `~/.gloamy/daemon_state.json` | file updates periodically |

## Logs and Diagnostics

### macOS arm64 (service wrapper logs)

- `~/.gloamy/logs/daemon.stdout.log`
- `~/.gloamy/logs/daemon.stderr.log`

### Linux (systemd user service)

```bash
journalctl --user -u gloamy.service -f
```

## Model Catalog Updates

The local `model-catalog.toml` is a complete replacement for the bundled catalog. Keep a copy of the last known-good file before editing it. Catalog lookup is offline; provider discovery uses the separate `gloamy models refresh` command and cache.

For a running gateway, send `POST /api/models/catalog/reload` with a valid stored paired bearer token. A `200` response publishes the validated catalog for later lookups. A `400` response means parsing or validation failed; the gateway keeps the last valid snapshot. Run `gloamy models list` locally to see the loading error, repair the file, and retry. Daemon logs record rejection without file contents. Restarting with an invalid existing file also fails explicitly, so restore a valid file before restarting.

The cutoff for curated menus, recommendations, and built-in prices advances with the date. Existing configured model IDs stay selected after they age out. Before changing the bundled catalog, verify exact IDs, release dates, aliases, rate conditions, and direct sources; omit a price when one flat input/output pair would be misleading. See [catalog schema and sources](config-reference.md#curated-model-catalog).

## Incident Triage Flow (Fast Path)

1. Snapshot system state:

```bash
gloamy status
gloamy doctor
gloamy channel doctor
```

2. Check service state:

```bash
gloamy service status
```

3. If service is unhealthy, restart cleanly:

```bash
gloamy service stop
gloamy service start
```

4. If channels still fail, verify allowlists and credentials in `~/.gloamy/config.toml`.

5. If gateway is involved, verify bind/auth settings (`[gateway]`) and local reachability.

## Safe Change Procedure

Before applying config changes:

1. backup `~/.gloamy/config.toml`
2. apply one logical change at a time
3. run `gloamy doctor`
4. restart daemon/service
5. verify with `status` + `channel doctor`

## Rollback Procedure

If a rollout regresses behavior:

1. restore previous `config.toml`
2. restart runtime (`daemon` or `service`)
3. confirm recovery via `doctor` and channel health checks
4. document incident root cause and mitigation

## Related Docs

- [one-click-bootstrap.md](one-click-bootstrap.md)
- [troubleshooting.md](troubleshooting.md)
- [config-reference.md](config-reference.md)
- [commands-reference.md](commands-reference.md)

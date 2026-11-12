# macOS Desktop App: Operations, Security, and Recovery

Last verified: **November 11, 2026**.

The Tauri desktop app reuses the existing Vue interface and starts the bundled
Gloamy daemon only when no healthy local daemon is available. A healthy existing
daemon is attached without ownership. The desktop webview connects to `127.0.0.1`
or `::1`; a specific non-loopback gateway bind is not supported by this POC.

## Daemon lifetime

**Keep daemon running after closing** is stored in desktop preferences and
defaults to Off. With Off, closing the app requests graceful shutdown only when
the daemon reports that it was started as desktop-managed. With On, that daemon
continues running for the browser UI. A daemon without the managed marker,
including a legacy daemon attached through its healthy HTTP API, is never stopped
by the desktop app.

## Pairing recovery

From Authentication or Settings, request a fresh one-time code and complete the
existing `/pair` flow. The code travels over a same-user Unix control socket, is
kept in memory, and is not written to configuration or logs. The new pairing
adds a token hash without invalidating paired devices. Pairing recovery is
unavailable when pairing is disabled, and no HTTP endpoint generates codes.

## Local control and security

The socket lives at `$TMPDIR/gloamy-<uid>/control.sock` (or the platform's
temporary-directory equivalent), with directory mode `0700` and socket mode
`0600`. Its only operations are status, pairing-code request, and graceful
shutdown. It provides no generic shell command or config write. The socket
protects against other local users; it is not a boundary against processes
running as the same user. The daemon's existing bearer-token checks remain in
place.

## Troubleshooting

- **Address occupied:** stop the service using the configured host and port, or
  edit `gateway.host` / `gateway.port` in `~/.gloamy/config.toml`. The desktop
  app will not start a duplicate daemon.
- **Pairing recovery unavailable:** check that pairing is enabled and use the
  desktop app. A browser cannot call the local control socket.
- **Existing daemon not reachable:** check that it is healthy at the configured
  loopback address. An unmanaged daemon remains running when the app closes.

For the full security contract, see
[Desktop Control Socket Security](../../security/desktop-control-socket.md).

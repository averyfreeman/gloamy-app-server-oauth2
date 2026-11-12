# Desktop Control Socket Security

Last verified: **November 11, 2026**.

The Tauri desktop host manages the existing Gloamy daemon through a local Unix
domain socket. The socket is created below the operating system temporary
directory at `gloamy-<effective-user-id>/control.sock`. Its parent directory is
restricted to mode `0700`, the socket to `0600`, and both are checked for current
user ownership before use. Unsafe paths are rejected; stale sockets are removed
only after checking their type and owner.

The protocol accepts only three typed operations: report readiness and the
loopback API URL, request a fresh one-time pairing code, and request graceful
shutdown. It does not expose shell execution or configuration mutation. The
webview receives only typed Tauri commands for status, pairing recovery, and the
keep-running preference. Its CSP limits daemon connections to loopback.

Pairing recovery is deliberately absent from HTTP. The code is held in memory,
returned only to the same user over the socket, and consumed by the existing
`POST /pair` flow. It is not written to configuration or logs. Adding a new
paired token preserves all existing token hashes. Recovery is rejected when
pairing is disabled.

The desktop app starts a daemon with fixed `daemon --desktop-managed` arguments
only when no healthy local Gloamy daemon is available. A daemon that owns the
socket reports its managed marker. On app exit, the host requests shutdown only
when that marker is true and **Keep daemon running after closing** is Off. The
preference defaults Off and is stored in the Tauri app config directory with
owner-only file permissions. An unmanaged daemon is never stopped by the app.

The control socket is a same-user boundary, not a defense against another
process running as that user. The local API retains its existing bearer-token
checks. No new network listener or pairing-code HTTP route is introduced.

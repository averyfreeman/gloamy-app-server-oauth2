# Daemon Control Socket Contract

## Transport

- Unix domain stream socket at `<system-temp>/gloamy-<effective-uid>/control.sock`.
- Parent directory mode `0700`; socket mode `0600`; verify owner and file type before connect or stale-path removal.
- One newline-delimited JSON request per connection and one newline-delimited JSON response. Reject requests larger than 4096 bytes. Do not log request or response bodies.

## Requests and responses

```json
{"op":"status"}
```

```json
{"ok":true,"ready":true,"api_base_url":"http://127.0.0.1:42617","desktop_managed":true,"gateway_version":"0.2.2"}
```

```json
{"op":"request_pairing_code"}
```

```json
{"ok":true,"code":"123456"}
```

```json
{"op":"shutdown"}
```

```json
{"ok":true}
```

Errors return `{"ok":false,"error":"<safe message>"}` without secrets. `request_pairing_code` fails when pairing is disabled. A request rotates the pending code; code generation is never reachable through HTTP.

`gateway_version` is included in status responses by current releases and is
optional when reading status, so clients continue to support older daemons
that omit it.

## Lifecycle

- The socket is created by the daemon and removed on graceful shutdown.
- A stale socket is removed only after verifying it is a socket owned by the effective user in the validated private directory and a connection fails with a stale-endpoint error.
- `status.ready` is false until the gateway publishes its actual listener address; wildcard listeners map to loopback URLs for the desktop webview.
- `desktop_managed` is set by the hidden desktop launch argument and is not inferred from the connecting client.

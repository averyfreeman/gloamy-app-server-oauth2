# Data Model: Tauri 2 Desktop POC

## Desktop preference

- `keep_daemon_running: bool`: app-local persisted preference. Defaults to `false`. Does not modify Gloamy runtime configuration.

## Daemon control status

- `ready: bool`: true only after the gateway listener has bound successfully.
- `api_base_url: Option<String>`: local URL derived from the actual bound socket address; absent until ready.
- `desktop_managed: bool`: process-start marker set only by the hidden Tauri-managed launch argument; survives closing the desktop window because it is process state.

## Pending pairing code

- `code: Option<String>`: current one-time plaintext code held only in `PairingGuard` memory.
- `paired_token_hashes: Set<String>`: existing persisted pairing state, unchanged by code issuance.
- State transition: request replaces the current pending code; successful `/pair` consumes it and adds a new token hash; unsuccessful attempts retain existing brute-force handling.

## Control requests

- `Status`: read daemon-managed state and gateway readiness/address.
- `RequestPairingCode`: fail if pairing is disabled; otherwise rotate and return a one-time code.
- `Shutdown`: acknowledge then signal the daemon’s existing supervisor shutdown path.

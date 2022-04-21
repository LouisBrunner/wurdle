# wurdle
State-less Wordle API in Rust, see `wurlde-server/api/openapi.yaml` for API details

## Building

```bash
make # only required when changing `wurlde-server/api/openapi.yaml`
cargo build
```

## Usage

```bash
SESSION_TOKEN="<SECRET_TOKEN>" cargo run
```

You can generate a `SESSION_TOKEN` using `openssl rand -base64 42` or any base64 encoded string. As long as you use the same `SESSION_TOKEN`, all sessions will be usable across reboots/multiple servers (just like JWT).

## TODO

 * Better HTTP error handling
 * Better internal error handling
 * Abstract the session management into a trait
 * Move "game" logic out of HTTP

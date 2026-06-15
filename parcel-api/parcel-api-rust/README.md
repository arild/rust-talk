# parcel-api (Rust port)

Rust port of the Kotlin/Spring Boot `parcel-api` for the conference talk
*Introduction to Rust: Efficiency Beyond the JVM*. It exists so cold-start time and
memory usage can be compared against the JVM service. Stubbed data; no auth.

## What's in / what's out

| Aspect | Decision |
| --- | --- |
| Web framework | `axum` on `tokio` |
| JSON | `serde` + `serde_json` |
| OpenAPI / Swagger | `utoipa` + `utoipa-swagger-ui` |
| Logging | `tracing` + JSON layer, plus `tower-http` request log middleware |
| Config | `figment` reading `config/{profile}.toml` + `APP_*` env vars |
| Auth | None — Spring Boot's `UserProfile` filter is not ported |
| HTTP client | None — service has no outbound calls |
| Database | None |

## Endpoints

All under context path `/parcel-api`.

- `POST /v1/parcel` — accepts `{"lastUpdated": …?, "exclude": …?}` (fields ignored),
  returns 50 stub parcels in `ParcelDataResponse` shape.
- `GET /check` — text/plain banner + memory + version.
- `GET /check/status` — `👋 parcel-api is on air`.
- `GET /v3/api-docs` — OpenAPI JSON.
- `GET /swagger-ui/` — Swagger UI.

## Stub data

50 parcels are loaded from the shared [`../parcel-data/`](../parcel-data/) folder at
startup (`StubParcelService::load`) and stored in `Arc<AppState>`. Each file is one
`ParcelResponse` in the same wire-format JSON the service emits. Spring Boot loads
the identical files — change a file there, both implementations see it next boot.

Directory resolved from `parcel_data_dir` in `config/{profile}.toml`, overridable
via `APP_PARCEL_DATA_DIR`. Defaults to `../parcel-data` (works for `cargo run`);
Docker image bakes the data in at `/parcel-data`.

List handler clones the cached `Vec<ParcelResponse>` per request — matches the
Java service which serializes the same in-memory list.

## Run

```bash
cargo run                # APP_PROFILE=dev (default)
APP_PROFILE=qa cargo run
```

Server binds to `0.0.0.0:${server_port}` (default `8080`) under `${context_path}`
(default `/parcel-api`).

## Test

```bash
cargo test
```

Tests use `tower::ServiceExt::oneshot` against the assembled `axum::Router` — no real
network listener. Coverage is intentionally narrow: a few parcel happy paths, validation,
health, swagger.

## Build for release

```bash
cargo build --release        # → target/release/parcel-api
```

Release profile uses thin LTO and stripped symbols.

## Docker

```bash
docker build -t parcel-api-rust .
docker run --rm -p 8080:8080 -e APP_PROFILE=dev parcel-api-rust
```

Multi-stage build, final stage is `gcr.io/distroless/cc-debian12`. The builder stage
installs `curl` because `utoipa-swagger-ui` shells out to it during the build.

## Environment variables

| Variable | Default | Notes |
| --- | --- | --- |
| `APP_PROFILE` | `dev` | Selects `config/<profile>.toml`. |
| `APP_CONFIG_DIR` | `config` | Directory containing the profile TOML files. |
| `RUST_LOG` | `info` | Standard `tracing-subscriber` env filter. |
| `TZ` | `Europe/Oslo` (in Docker) | Matches the JVM service. |

## Differences from the Kotlin service (intentional)

- No authentication, no `UserProfile` — handlers don't use any user context.
- Health banner is plain text (no ASCII art).
  Memory stats only work on Linux (parses `/proc/self/statm`).
- `Instant` JSON formatting is hand-rolled to match Java's `Instant.toString()`
  (omits trailing zero subseconds; emits 3/6/9 digits when present).

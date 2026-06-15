# parcel-api-node

Node.js port of the `parcel-api` benchmark service, behaviourally identical to
the Spring Boot port. The response is payload-identical to the Go / Rust /
Quarkus ports (same 100 parcels, same fields/key order/formatting); like every
port it shuffles the array per request, so the raw byte order differs run to run.

- **Runtime:** Node.js 22, [Fastify](https://fastify.dev/) 5 HTTP server.
- **Dependencies:** `fastify` only, pinned exactly with `package-lock.json`
  committed (image builds via `npm ci`). Policy: any third-party dependency must
  be a version published **≥ 1 week ago** (supply-chain safety) — see the note in
  `package.json` and the `Dockerfile`.
- **Concurrency:** the `node:cluster` module forks one worker per available core
  (`availableParallelism()`, which resolves to the container's CPU limit; override
  with `WEB_CONCURRENCY`). Workers share the listen socket and the OS load-balances
  connections across them, so the CPU-bound JSON work spreads across cores —
  unlike the single-threaded C port. Each worker is a full process with its own
  parcel data, so RSS scales roughly linearly with the worker count.

## Endpoints

| Method | Path | Response |
|---|---|---|
| `POST` | `/parcel-api/v1/parcel` | Bare JSON array of 100 parcels (~396 KB). Request body is ignored. |

## Byte parity

Each request re-parses every parcel from the in-memory JSON strings, recomputes
the derived `features`, and re-serializes — the same per-request work as the
other ports. A custom serializer (`serialize.js`) reproduces the exact wire
format: canonical field order, `null` fields omitted, `weightInKg` keeping its
decimal point (`5` → `5.0`), and no HTML escaping. Dates are already canonical
whole-second `…Z` strings in the fixtures and pass through unchanged.

## Run locally

```sh
npm ci                                          # install fastify from the lockfile
PARCEL_DATA_DIR=../parcel-data node server.js   # one worker per core
WEB_CONCURRENCY=1 PARCEL_DATA_DIR=../parcel-data node server.js   # single worker
```

## Build the image

```sh
docker build --load -t parcel-api-node -f parcel-api-node/Dockerfile ..
```

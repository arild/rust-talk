# Parcel API — many ports, one benchmark

Perf comparison of the same service across several runtimes — Spring Boot, Quarkus (JVM + native), Rust, C, Go, and Node — on a heavy endpoint: `POST /v1/parcel` returns ~50 rich parcel objects (~200 KB JSON per call), which stresses the runtime's JSON path.

Source-of-truth is `../other/posten-parcel-api/`. The ports here are stripped-down replicas: no DB, no Kafka, no real downstream HTTP, no metrics, no auth. A fixed `UserProfile` is injected by a request filter; one in-process stub loads parcel data **from a shared JSON folder at startup** and serves it. Parcel-number validation and the Swagger UI are kept faithful.

### Shared parcel data — [`parcel-data/`](parcel-data/)

100 JSON files, one per parcel (`TESTPARCEL0001000000.json` … `TESTPARCEL0001000099.json`). Both implementations read every `*.json` file from this folder on startup and **keep the contents as raw strings (or bytes) in memory**. On every request, the strings are re-deserialised into `ParcelResponse` objects and then the framework re-serialises them for the response. That puts the JSON parse cost on the request path — closer to what a real service does when it decodes data from a downstream — and it's where the JVM's GC pressure under load shows up clearly.

`POST /v1/parcel` returns the whole set (~400 KB JSON, 100 parcels). The folder is the single source of truth — change a file, both implementations see it on next boot.

Implementations resolve the directory from:

| | Source | Default |
|---|---|---|
| Spring Boot | `parcel.data.dir` property (override via `PARCEL_DATA_DIR` env var) | `../parcel-data` (works for `./gradlew bootRun`); `/parcel-data` inside the Docker image |
| Rust | `parcel_data_dir` in `config/{profile}.toml` (override via `APP_PARCEL_DATA_DIR` env var) | `../parcel-data` (works for `cargo run`); `/parcel-data` inside the Docker image |

Both Dockerfiles use `parcel-api/` as the build context so the data folder can be COPY'd in.

## Status

| Variant              | Path / Dockerfile                                       | State                                                  |
|----------------------|---------------------------------------------------------|--------------------------------------------------------|
| Spring Boot          | `parcel-api-spring-boot/`                               | ✅ done                                                |
| Quarkus (JVM)        | `parcel-api-quarkus-jvm/`                               | ✅ done — payload-identical to Spring Boot†            |
| Quarkus (native, Serial) | `parcel-api-quarkus-native/Dockerfile.native`       | ✅ done — Mandrel GraalVM AOT, Serial GC               |
| Quarkus (native, G1) | `parcel-api-quarkus-native/Dockerfile.native-g1`        | ✅ done — Oracle GraalVM AOT, G1 GC (`--gc=G1`)        |
| Rust                 | `parcel-api-rust/`                                      | ✅ done — payload-identical to Spring Boot†            |
| C                    | `parcel-api-c/`                                         | ✅ done — payload-identical to Rust†                   |
| Go                   | `parcel-api-go/`                                        | ✅ done — payload-identical†                           |
| Node.js              | `parcel-api-node/`                                      | ✅ done — payload-identical†; Fastify + cluster (multi-core) |

> † **"Payload-identical"** = every port returns the same 100 parcels, field-for-field
> identical content (same fields, key order, and number formatting), 395 706 bytes
> total. The only difference is array order: **every port deliberately shuffles the
> array on each request** (so the serializer can't memoize a constant response), so
> even two responses from one port differ. Sorted by parcel number all seven hash
> identically — order changes neither byte count nor per-request work, so the
> benchmark stays apples-to-apples.

> **Quarkus is two directories on purpose.** `parcel-api-quarkus-jvm/` and
> `parcel-api-quarkus-native/` carry the **same source** — the JVM-vs-native
> comparison only holds if the code is identical. Mirror any `src/` change across
> both. The only intended differences are the Dockerfile and the native-only
> `NativeReflectionConfig.kt`.

---

## Build & run

All variants serve on port `8080` under context path `/parcel-api`. Run more than one at a time by remapping the host port (e.g. `-p 8081:8080`).

Paths below are relative to this `parcel-api/` directory.

### Spring Boot — `parcel-api-spring-boot/`

Run locally:

```bash
cd parcel-api-spring-boot
./gradlew bootRun
```

Build the Docker image (Dockerfile expects the jar in `build/libs/`, so build first):

```bash
cd parcel-api-spring-boot
./gradlew build -x test
docker build --load -t parcel-api-spring-boot -f Dockerfile ..
docker run --rm -p 8080:8080 parcel-api-spring-boot
```

### Rust — `parcel-api-rust/`

Run locally:

```bash
cd parcel-api-rust
cargo run --release
```

Build the Docker image (multi-stage; `cargo build` runs inside the builder, no host pre-build needed):

```bash
cd parcel-api-rust
docker build --load -t parcel-api-rust -f Dockerfile ..
docker run --rm -p 8080:8080 parcel-api-rust
```

### Quarkus (JVM) — `parcel-api-quarkus-jvm/`

Run locally (dev mode with live reload):

```bash
cd parcel-api-quarkus-jvm
./gradlew quarkusDev
```

Build the Docker image (Quarkus fast-jar layout under `build/quarkus-app/`; build context is the parent so the shared `parcel-data/` is in scope):

```bash
cd parcel-api-quarkus-jvm
./gradlew build -x test
docker build --load -t parcel-api-quarkus-jvm -f Dockerfile ..
docker run --rm -p 8080:8080 parcel-api-quarkus-jvm
```

### Quarkus (native) — `parcel-api-quarkus-native/`

Same source as the JVM dir, compiled ahead-of-time to a standalone binary. The
compile runs inside the builder stage (no host pre-build); it takes minutes and
wants **≥ 6 GB for Docker** (`native-image` peaks ~2.6 GiB). Two builds, differing
only in GC:

```bash
cd parcel-api-quarkus-native

# Serial GC (Mandrel — the free/community default)
docker build --load -t parcel-api-quarkus-native -f Dockerfile.native ..
docker run --rm -p 8080:8080 parcel-api-quarkus-native

# G1 GC (Oracle GraalVM — ~3× the Serial throughput on this workload)
docker build --load -t parcel-api-quarkus-native-g1 -f Dockerfile.native-g1 ..
docker run --rm -p 8080:8080 parcel-api-quarkus-native-g1
```

See [`parcel-api-quarkus-native/README.md`](parcel-api-quarkus-native/README.md#build-the-native-image)
for the Serial-vs-G1 trade-off, toolchain-tag fallbacks, and the keep-in-sync note.

### Smoke-test

```bash
# List all parcels (~200 KB response, no auth needed — UserProfile is stubbed)
curl -X POST http://localhost:8080/parcel-api/v1/parcel \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Benchmark

Once any variant is built, run [`bench.sh`](bench.sh):

```bash
./bench.sh all                       # all variants (skips missing images with a warning)
./bench.sh spring-boot 5             # one variant, 5 cold-start runs
./bench.sh quarkus-native-g1 5       # the Oracle-GraalVM G1 native bar on its own
MEM_LIMIT=1g ./bench.sh all          # pin every container to a 1 GiB pod-style limit
```

Variants: `spring-boot-{g1,parallel}`, `quarkus-jvm-{g1,parallel}`,
`quarkus-native` (Mandrel, Serial GC), `quarkus-native-g1` (Oracle GraalVM, G1),
`rust`, `c`, `go`, `node`. The JVM images carry a GC matrix; the native binaries
bake the GC in at build time, so they're one variant each. `BUILD=1 ./bench.sh all`
builds every image first.

**`MEM_LIMIT`** sets `docker run --memory/--memory-swap` (default: unlimited).
Set it to a pod-style limit (e.g. `1g`) for a fair RSS comparison — otherwise each
GC sizes its default heap to 25% of *whatever RAM the Docker VM exposes*, which
makes peak-RSS numbers incomparable across runs on differently-sized hosts. `CPU_LIMIT`
(default 3) caps cores the same way. Latest fair run: `bench-results/2026-06-04-fixed1g/`.

Measures cold-start time (median + p95) and container RSS at three points, plus total requests served and throughput:

1. **Idle** — 3 s after the container is up, before any load.
2. **Warm** — after a short sequential warm-up phase (1 request at a time, default 5 s). JVMs in particular start to JIT the hot path here.
3. **Peak** — after a linear ramp 1 → `MAX_PARALLEL` concurrent (default 15 s) plus a steady hold at `MAX_PARALLEL` (default 60 s). This is where allocator behaviour and GC pressure show up.

Each phase tallies requests via a counter file (single-byte appends are atomic on POSIX). The final table reports total requests across the three phases, total load duration, and average req/s.

The load endpoint is the real `POST /v1/parcel` (the stub is in-process, so the call doesn't depend on anything external — and the ~400 KB response actually exercises the JSON encoder, which is the point).

Bash is fine for the orchestration since we're measuring *memory*, not latency percentiles. If you want robust p99 / throughput numbers, point [`oha`](https://github.com/hatoo/oha) or [`hey`](https://github.com/rakyll/hey) at the same endpoint instead.

### Swagger UI

| Variant                | Swagger UI                                                          | OpenAPI JSON                                                       |
|------------------------|---------------------------------------------------------------------|--------------------------------------------------------------------|
| Spring Boot            | `http://localhost:8080/parcel-api/swagger-ui/index.html`            | `http://localhost:8080/parcel-api/v3/api-docs`                     |
| Quarkus (JVM + native) | `http://localhost:8080/parcel-api/q/swagger-ui`                     | `http://localhost:8080/parcel-api/q/openapi`                       |
| Rust                   | `http://localhost:8080/parcel-api/swagger-ui/`                      | `http://localhost:8080/parcel-api/v3/api-docs`                     |

---

## What's kept from the source

- `POST /v1/parcel` from `ParcelController`, with the same request/response shapes at the top level.
- Springdoc-driven Swagger UI on the JVM ports.

## What's stubbed / dropped

- **All downstream HTTP** — one `StubParcelService` loads 50 `ParcelResponse` objects from the shared [`parcel-data/`](parcel-data/) folder at startup. No `ParcelHubClient`, `SporingService`, `BookingService`, etc.
- **Auth** — `UserProfileFilter` sets a fixed `UserProfile("demo-user-123")` on every request. No PostenID, no Bearer-token resolution.
- **Batch fetching** — `ParcelRequest` still carries `lastUpdated` / `exclude` for shape parity, but they're ignored. No `ParcelFilter`, no partitioning, no `remainingCount` calculation.
- **All other controllers** — no archive, hide, alias, webhooks, BankID, sporing-response, anti-crime, customer-service webhook, etc.
- **Kafka, Liquibase, DB, ShedLock, Resilience4j, Datadog APM, AppStatsService** — none.
- **Response DTO nesting** — top-level shape of `ParcelDataResponse` and `ParcelResponse` matches the source; the nested types (`DeliveryResponse`, `TransportResponse`, etc.) are simplified to single data classes with a `type` discriminator instead of sealed hierarchies. The source's response uses ~21 DTO files dragging in the entire `domain` package; the port collapses that to ~15 plain data classes.

## References

- [`.claude/template/posten-parcel-api/`](../.claude/template/posten-parcel-api/) — source-of-truth Spring Boot service the ports are based on.
- [`../apps-ports.md`](../apps-ports.md) — conversion notes (historical: the earlier `branded-tracking-app` ports).

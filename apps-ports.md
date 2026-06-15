# Apps — Conversion Notes

> **Historical.** These are the original conversion notes for the `branded-tracking-app` and `posten-parcel-api` ports listed below. Those source trees are **no longer in this repo** — the active benchmark is [`parcel-api/`](parcel-api/README.md). Kept for reference.

Detailed record of the Rust + Quarkus ports. Captures what's in each port, what's stubbed, what was intentionally dropped, and verification status.

---

## Summary

| Source | Port | Path | Stack |
|---|---|---|---|
| `branded-tracking-app` | Rust | `apps/branded-tracking-app/branded-tracking-app-rust/` | axum 0.8 + tokio + reqwest |
| `branded-tracking-app` | Quarkus | `apps/branded-tracking-app/branded-tracking-app-quarkus/` | Quarkus 3.30 + Kotlin 2.3 + JDK 25 |
| `posten-parcel-api` | Rust | `apps/posten-parcel-api-rust/` | axum 0.8 + sqlx 0.8 + Postgres |

All three were scaffolded by background agents with detailed briefs. None were compile-verified by the agents — sub-agent bash sandbox blocked `cargo` and `gradle` even with override. First builds need to be done locally; expect small fixes.

Two cross-cutting decisions, applied to all ports:
- **No circuit breakers.** Source uses Resilience4j; ports forward directly with timeouts only. Removes a moving part that isn't central to the talk.
- **No strict lint gating.** Source uses Diktat (Kotlin); ports skip Diktat / ktlint / detekt entirely, and Rust ports don't gate the build on `cargo clippy` or `cargo fmt`.

---

## Port 1 — `branded-tracking-app-rust`

### Stack
- **Web:** `axum` 0.8 on `tokio`
- **HTTP client:** `reqwest` with `rustls-tls` (no openssl), 3s connect / 5s read timeouts
- **JSON:** `serde` + `serde_json`
- **Date/time:** `chrono` with serde feature, custom serializer for `yyyy-MM-dd'T'HH:mm:ss.SSSX` UTC
- **OpenAPI:** `utoipa` 5 + `utoipa-swagger-ui` 9
- **Logging:** `tracing` + `tracing-subscriber` JSON layer
- **Config:** `figment` reading `config/{profile}.toml` + `APP_*` env, profile from `APP_PROFILE` (default `dev`)
- **Tests:** `wiremock` crate (Rust port of WireMock) + `tower::ServiceExt::oneshot`
- **Edition:** Rust 2024 (needs rustc ≥ 1.85)

### What's faithful
- One real endpoint: `POST /branded-tracking-app/v1/branded-tracking`
- Same request/response shapes, same outbound URL, same JSON enum mapping (6-variant input enum collapsed to MyBring's `VIEW`/`CLICK` + `logo` + `viewType`)
- Same date format on outbound payload
- Same auth semantics (missing/non-Bearer → 403, invalid → 401)
- Same 200 / 400-no-active-campaign no-op / 500 error handling
- Health endpoints, OpenAPI/Swagger UI, JSON logging, X-Request-Id passthrough

### What's stubbed
- **PostenID GraphQL:** stubbed. `Bearer demo-token` → `UserProfile { user_id: "demo-user-123" }`. Anything else → 401. Optional: set `APP_POSTENID__URL` to flip to a real GraphQL POST.
- **MyBring downstream:** real outbound HTTP via `reqwest`. URL/credentials configurable. If creds missing, logs and skips.
- **Internal Posten/Bring libs** (`postenid:profile`, `correlationid-spring6`, `errorhandling`): reimplemented bare minimum. Correlation ID = read/forward `X-Request-Id`.

### What's intentionally out
- Circuit breakers (Resilience4j)
- Datadog APM
- Strict lint gating
- Banner.txt / git-properties version (stub `version: "demo"` instead)
- Spring DI container (plain `Arc<State>` instead)

### Layout
```
apps/branded-tracking-app/branded-tracking-app-rust/
├── Cargo.toml, rust-toolchain.toml, Dockerfile, README.md
├── config/{dev,qa,production}.toml
└── src/
    ├── main.rs, lib.rs, app.rs, config.rs
    ├── auth/{middleware,postenid}.rs
    ├── controller/{branded_tracking,health,error}.rs
    ├── client/mybring.rs
    └── domain/{tracking,time}.rs
tests/{branded_tracking,health,swagger}.rs
```

### Tests written
Many small `#[tokio::test]`s. Cover: success path per `BrandedTrackingType` variant, no-active-campaign no-op, downstream error → 500, error body has null `message`, missing/invalid auth (403/401), health/status/openapi/swagger UI reachable. **Circuit-breaker tests intentionally omitted.**

### Verification
- Local Rust toolchain was 1.73 (Oct 2023) — too old for edition 2024. Updated via `rustup update stable` to 1.95.0.
- After update, `cargo test` was attempted; further triage needed locally.
- Dockerfile not yet built.

---

## Port 2 — `branded-tracking-app-quarkus`

### Stack
- **Quarkus** 3.30.3
- **Kotlin** 2.3.21, **JDK** 25
- **Build:** Gradle (Kotlin DSL, copied wrapper from source)
- **Web:** `quarkus-rest-jackson` (RESTEasy Reactive)
- **HTTP client:** `quarkus-rest-client-jackson` (MicroProfile REST Client) with `@RegisterRestClient(configKey = "mybring-api")`
- **OpenAPI:** `quarkus-smallrye-openapi`
- **Health:** `quarkus-smallrye-health`
- **Config:** `quarkus-config-yaml` (YAML keeps source's style)
- **Logging:** `quarkus-logging-json`
- **Tests:** `quarkus-junit5` + REST Assured + WireMock standalone + AssertJ
- **Mode:** JVM only (no native image)
- **Allopen:** enabled for Kotlin

### What's faithful
- Same single endpoint, same request/response shapes
- Context path `/branded-tracking-app/` preserved
- Same auth semantics, same outbound transformation
- Profile-aware configs (`%dev`, `%test`, `%qa` selectors plus dedicated `application-{profile}.yml`)
- Same exception → 500 with `{"message":null}` behaviour

### What's stubbed
- **PostenID:** `PostenIdStub.kt`. `Bearer demo-token` → `UserProfile(userId = "demo-user-123")`. Blank rejected. Otherwise 401.
- **MyBring:** real outbound via MicroProfile REST Client. Configurable URL/creds.
- **Internal Posten/Bring libs:** correlation ID = read/forward `X-Request-Id`.

### What's intentionally out
- `quarkus-smallrye-fault-tolerance` (no circuit breakers)
- Diktat / ktlint / detekt (no Kotlin linter)
- `quarkus-spring-*` shims (proper Quarkus annotations, not Spring-on-Quarkus)
- Datadog APM
- OAuth2 to MyBring
- Real PostenID GraphQL
- Native image

### Layout
```
apps/branded-tracking-app/branded-tracking-app-quarkus/
├── build.gradle.kts, settings.gradle.kts, gradle.properties
├── gradle/, gradlew, gradlew.bat
├── Dockerfile (eclipse-temurin:25-jre-alpine, Quarkus fast-jar)
├── README.md
└── src/
    ├── main/kotlin/no/posten/brandedtracking/
    │   ├── auth/{UserProfile,PostenIdStub,UserProfileFilter,RequestStore}.kt
    │   ├── client/{MyBringClient,MyBringConfig,BrandedTrackingService}.kt
    │   ├── configuration/{Time,OpenAPIConfig}.kt
    │   ├── controller/{BrandedTrackingResource,HealthResource}.kt
    │   ├── controller/exceptions/{ErrorMessage,GlobalExceptionMapper}.kt
    │   └── domain/BrandedTrackingDomain.kt
    ├── main/resources/application{,-dev,-test,-qa}.yml
    └── test/kotlin/no/posten/brandedtracking/
        ├── config/{TestBuilders,TimeMock,MyBringWireMock}.kt
        └── controller/{BrandedTrackingResourceTest,SwaggerResourceTest,HealthResourceTest}.kt
```

### Tests written
~15 small tests across 3 classes. Cover: each `BrandedTrackingType` variant success path (view, view banner, view content card, view logo), click for different user, no-active-campaign no-op (200), unhandled validation (500), no-message-leak, missing auth (401), blank token (401), Swagger UI/OpenAPI reachable, health/status reachable, `/q/health` reachable.

`MyBringWireMockResource` is a `QuarkusTestResourceLifecycleManager` — starts WireMock on port 11300, overrides `quarkus.rest-client.mybring-api.url` so REST Assured tests hit the mock.

### Verification
- Not built. Sub-agent's bash blocked `gradle`. First `./gradlew build` may surface small Quarkus-Kotlin gotchas (REST Client method-return-type quirks, `@ConfigMapping` Kotlin getter naming, allopen coverage).

---

## Port 3 — `posten-parcel-api-rust`

The big one. Source is 545 Kotlin files (~29k LOC), multi-module Gradle, Postgres + Liquibase + Kafka + 14+ downstream HTTP integrations. Full faithful port is months of work; this is a **realistic skeleton** focused on the warmup mechanism.

### Stack
- **Web:** `axum` 0.8 on `tokio`
- **HTTP client (warmup loopback):** `reqwest` with `rustls-tls` and gzip
- **Database:** `sqlx` 0.8 with `postgres`, `runtime-tokio-rustls`, `chrono`, `uuid` features
- **JSON:** `serde` + `serde_json` (camelCase)
- **Date/time:** `chrono` with `serde` feature
- **OpenAPI:** `utoipa` 5 + `utoipa-swagger-ui` 9
- **Logging:** `tracing` + `tracing-subscriber` (JSON layer optional)
- **Config:** `figment` (`config/{profile}.toml` + `APP_*` env)
- **UUID:** `uuid` 1.x with `v4` (for warmup token)
- **Edition:** Rust 2024

### Centerpiece — the warmup
Faithful replication of the Spring Boot `WarmupService.kt` mechanic. In `src/warmup/service.rs`:

1. After `axum::serve` binds, spawn the warmup task.
2. Task generates a UUID-stamped bearer token: `system-warmup-user-token-{uuid}`. Token is registered with the auth `TokenResolver` so the middleware can resolve it to `systemWarmupUser` (`UserId("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")`).
3. Task polls `GET /parcel-api/health/warmup` until the listener is up.
4. Task loops: `POST /parcel-api/v1/parcel` with `Authorization: Bearer <warmup-token>`, body `{ "exclude": [...seen so far...] }`. Accumulates returned parcel numbers into `exclude`. Exits when `remainingCount == 0` or `remainingCount` plateaus (matches the Kotlin `tailrec`).
5. Flips an `AtomicBool` (the warmup flag) on success.
6. `GET /health/warmup` returns 200 if the flag is set, 409 otherwise.

### Endpoints
Public (no auth):
- `GET /parcel-api/health/warmup` (200/409)
- `GET /parcel-api/check` (banner + memory + version)
- `GET /parcel-api/check/status` ("👋 posten-parcel-api is on air")
- `GET /parcel-api/v3/api-docs` (OpenAPI JSON)
- `GET /parcel-api/swagger-ui/...`

Bearer-protected:
- `POST /parcel-api/v1/parcel` — body `{ lastUpdated?, exclude }`, returns `ParcelDataResponse { parcels, totalCount, remainingCount, failedParcelNumbers, deleteParcelNumbers, providedLastUpdated }`. Reads customizations on every request (real DB hit), filters HIDDEN, applies aliases, applies `exclude` and `lastUpdated`.
- `GET /parcel-api/v1/parcel/{parcelNumber}` — single parcel or 404.
- `POST/DELETE /parcel-api/v1/parcel/{n}/archive` — `archived_parcel` table insert/delete.
- `POST /parcel-api/v1/parcel/{n}/hide` — `parcel_customization` upsert with state=HIDDEN.
- `PUT /parcel-api/v1/parcel/{n}/alias` — body `{ alias }`, `parcel_customization` upsert.

### Auth (stubbed)
`TokenResolver` recognises three tokens:
- The runtime-generated warmup token → `systemWarmupUser`
- `Bearer demo-user-1-token` → mock user 1 (`UserId("2d5ee951be1043078eb0c890d74ca81a")`, phone `+4759000010`)
- `Bearer demo-user-2-token` → mock user 2 (`UserId("62494d0c9f984ab480809178cbb79183")`, phone `+4759000011`)

Anything else → 401. `UserProfile` is an axum request extension; handlers extract it.

### Mock parcel data
10 parcels per user in `src/mockdata/parcels.rs`. Representative slice (NOT all 40+ types from the source's `ParcelType.kt`): parcel-locker pickup × 2 (Posten + Bring), PIB pickup, home delivery in transit, return in transit, mailbox, BankID-required, customs tax, delivered, "pakke i postkassen". Shapes mirror the source's `ParcelResponse` so JSON looks real on stage.

### Persistence
PostgreSQL via `sqlx`. Schema migrated by `sqlx::migrate!("./migrations")` at startup.

`migrations/0001_archived_parcel.sql`:
```sql
CREATE TABLE IF NOT EXISTS archived_parcel (
    unique_posten_id text NOT NULL,
    parcel_number text NOT NULL,
    consignment_number text,
    sender_name text,
    logo_url text,
    latest_event_date timestamptz,
    product_code text,
    direction text,
    weight_in_kg numeric,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (unique_posten_id, parcel_number)
);
CREATE INDEX IF NOT EXISTS idx_archived_parcel_expires_at ON archived_parcel(expires_at);
```

`migrations/0002_parcel_customization.sql`:
```sql
CREATE TABLE IF NOT EXISTS parcel_customization (
    unique_posten_id text NOT NULL,
    parcel_number text NOT NULL,
    state text NOT NULL,           -- 'HIDDEN' | 'VISIBLE'
    alias text,
    direction text,
    expires_at timestamptz NOT NULL,
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (unique_posten_id, parcel_number)
);
CREATE INDEX IF NOT EXISTS idx_parcel_customization_expires_at ON parcel_customization(expires_at);
CREATE INDEX IF NOT EXISTS idx_parcel_customization_state ON parcel_customization(state);
```

### docker-compose
Postgres 16, port 5432, init script creates the `parcel-api` database and `parcel-api` user matching the source's setup. `docker compose up -d && cargo run` should be enough locally.

### What's intentionally out
The list is long — being explicit so we don't pretend this is more than it is:
- All 14 downstream HTTP integrations (Sporing, ParcelHub, PostenID GraphQL, BankID, Push, Pakkeboks, Pickup Point, Booking, Sende, Customs Tax, Fossil Free, Parcel Sharing, Tracking Internal API, Branded Tracking, etc.) — replaced with in-memory mocks
- Kafka (the source even excludes `KafkaAutoConfiguration` in its main class)
- OAuth2 client / mybring auth server
- Circuit breakers (Resilience4j, no `failsafe` crate)
- Datadog APM
- AppStats / OpenSearch metrics export
- Liquibase (sqlx migrations instead)
- ShedLock (distributed scheduling)
- Webhook controllers (`CustomerServiceAppWebhookController`, `ParcelLockerWebhookController`)
- BankID callback controller
- Anti-crime controller, sporing-response controller, parcel-event controller
- Prometheus management server on port 8090
- Virtual threads (Rust everything is async tokio anyway)

### Layout
```
apps/posten-parcel-api-rust/
├── Cargo.toml, rust-toolchain.toml, Dockerfile, docker-compose.yml
├── .env.example, .gitignore, README.md
├── config/{dev,qa,production}.toml
├── migrations/{0001_archived_parcel,0002_parcel_customization}.sql
└── src/
    ├── main.rs, lib.rs, config.rs, app.rs, state.rs
    ├── auth/{middleware,postenid}.rs
    ├── controller/{parcel,customization,health,error}.rs
    ├── domain/{parcel,user,customization}.rs
    ├── mockdata/parcels.rs              (10 parcels per user)
    ├── persistence/{archived_parcel,parcel_customization}.rs
    ├── service/parcel_service.rs        (DB read on every /v1/parcel call)
    └── warmup/service.rs                (the centerpiece)
tests/{warmup,parcel,customization,health}_test.rs + tests/common/
```

### Tests written
- `warmup_test.rs` — boots the app, runs warmup, asserts `/health/warmup` flips to 200.
- `parcel_test.rs` — POST /v1/parcel with mock-user-1 token returns expected parcels; empty token → 401; `exclude` reduces batch.
- `customization_test.rs` — archive/hide/alias persist to DB; POST /v1/parcel respects HIDDEN.
- `health_test.rs` — /check, /check/status, OpenAPI reachable.

### Verification
- Not built. Sub-agent's bash blocked `cargo` and `docker` even with `dangerouslyDisableSandbox`.
- To bring up locally:
  ```sh
  cd apps/posten-parcel-api-rust
  docker compose up -d postgres
  DATABASE_URL=postgres://parcel-api:parcel-api@localhost:5432/parcel-api cargo test
  APP_PROFILE=dev cargo run
  ```
- Most likely first-build issues: `figment::providers::Env::split` if figment is older than 0.10.7; a `utoipa::ToSchema` derive missed on a domain type. Both are surgical fixes.

---

## Port 4 — `parcel-api-quarkus-native` (GraalVM AOT)

Added so the talk can show **both** Quarkus bars — JVM fast-jar and GraalVM-native.
The original `parcel-api-quarkus/` was split into two sibling directories, one per
build mode, matching the one-directory-per-port layout of the rust/c/go/spring-boot
ports:

- **`parcel-api-quarkus-jvm/`** — the original, renamed. Unchanged behaviour; JVM
  fast-jar via its `Dockerfile`.
- **`parcel-api-quarkus-native/`** — full copy of the source plus `Dockerfile.native`
  and the native-only `NativeReflectionConfig.kt`.

**Trade-off taken on knowingly:** the two dirs hold the *same source*, so they can
drift. The JVM-vs-native comparison is only valid while the code is identical, so
both READMEs and the top-level `parcel-api/README.md` carry a "mirror any `src/`
change across both" note. (A single-directory / two-Dockerfile layout would have
avoided the duplication, but per-port directory symmetry was preferred.) The only
intended differences between the two dirs are the Dockerfile and `NativeReflectionConfig.kt`.

### What was added (native dir)
- **`Dockerfile.native`** (Serial GC, Mandrel) — multi-stage. Stage 1 is the
  Mandrel builder image (`quay.io/quarkus/ubi9-quarkus-mandrel-builder-image:jdk-25`),
  which compiles the binary inside Docker (mirrors the rust/c/go builds — no host
  GraalVM needed). Stage 2 is `quay.io/quarkus/ubi9-quarkus-micro-image:2.0`. Both
  image tags are `--build-arg`s. Two gotchas, both fixed here, both learned the
  hard way at first build:
    - **Use `quarkusBuild` + `-Dquarkus.package.jar.enabled=false`, not the
      lifecycle `build`.** With native enabled, asking for both a fast-jar and a
      native binary fails: *"Outputting both native and JAR packages is not
      currently supported."*
    - **Runtime base must match the builder's glibc.** The UBI9 builder (glibc
      2.34) produces a binary that crashes on the UBI8 `quarkus-micro-image:2.0`
      with *"GLIBC_2.34 not found"*; the **`ubi9-`** micro image (glibc 2.34) is
      the correct pair.
- **`Dockerfile.native-g1`** (G1 GC, Oracle GraalVM) — second native build. G1 for
  native image is **Oracle-GraalVM-only** (Mandrel / CE ship Serial GC only) and is
  a *build-time* flag (`--gc=G1`). Installs Oracle GraalVM from its public tarball
  (GFTC licence — no registry login) on a UBI9 base, so the binary still matches
  the `ubi9-quarkus-micro-image` runtime. Same source as `Dockerfile.native`; only
  the builder and the `--gc=G1` flag differ. Image `parcel-api-quarkus-native-g1`.
- **`NativeReflectionConfig.kt`** (`service/domain/`) — `@RegisterForReflection`
  over every `service.domain` type. `StubParcelService` deserialises these with a
  hand-built `ObjectMapper`, so they never appear in a JAX-RS signature and
  Quarkus can't register them for native reflection on its own. Without this the
  binary builds but 500s on the first `/v1/parcel` call. Inert on the JVM, so the
  JVM dir omits it. (Verified sufficient — native serves `POST /v1/parcel` at 100%.)

### Bench wiring (`bench.sh`)
- The JVM variants were renamed `quarkus-g1` / `quarkus-parallel` →
  `quarkus-jvm-g1` / `quarkus-jvm-parallel` (image `parcel-api-quarkus` →
  `parcel-api-quarkus-jvm`) so the bar labels match the new dir names.
- Variants `quarkus-native` → `parcel-api-quarkus-native` (Serial, `Dockerfile.native`)
  and `quarkus-native-g1` → `parcel-api-quarkus-native-g1` (G1, `Dockerfile.native-g1`).
  Each wired through `ALL_VARIANTS`, `image_for`, `clean_build_image` (cache-busting
  `APP_BUILD_ID`, like rust/c/go), `artifact_path_for` (`/usr/local/bin/parcel-api`),
  and the build-time dedup bookkeeping.
- `gc_opts_for` matches `quarkus-native*` *before* the `*-g1` glob, so the native
  binaries (GC baked in at build time) don't get a spurious `JAVA_TOOL_OPTIONS`.
- **`MEM_LIMIT` env var** sets `docker run --memory/--memory-swap`, so every
  container — and thus every GC's default heap (25% of the cgroup limit) — is sized
  to the same budget. Set it to mirror a k8s pod limit for a fair RSS comparison;
  empty = unlimited (old behaviour).

### Verification — built and benched
All five non-parallel variants build and run. Results at `--cpus 3`, `MEM_LIMIT=1g`
(a realistic pod limit; bench dir `bench-results/2026-06-04-fixed1g/`):

| variant | cold start | idle RSS | peak RSS | steady req/s |
|---|---|---|---|---|
| spring-boot-g1 | 1.96 s | 184 MiB | 452 MiB | 315 |
| quarkus-jvm-g1 | 0.75 s | 86 MiB | 386 MiB | 352 |
| quarkus-native (Serial) | 0.09 s | 6.8 MiB | 73 MiB | 115 |
| quarkus-native-g1 | 0.10 s | 6.3 MiB | 276 MiB | 287 |
| rust | 0.09 s | 2.1 MiB | 13 MiB | 1013 |

Findings:
- Native cold-start ~90 ms (≈ Rust), idle RSS ~6–7 MiB — both far below the JVMs.
- **Serial GC (Mandrel) bottlenecks throughput** on this allocation-heavy endpoint
  (~140 stop-the-world collections/sec → 115 req/s). **G1 (Oracle GraalVM) fixes it**
  (287 req/s, near the JVM's 352) — the throughput regression was the GC, not AOT.
- Under a fixed 1 GiB limit, **G1-native peak RSS (276 MiB) is lighter than both
  JVMs.** The 1.2 GiB seen in an unconstrained 8 GiB run was G1 sizing its heap to
  25% of *visible RAM*; native is cgroup-aware, so a real pod limit bounds it.
- Build cost: JVM ~9 s · Mandrel native ~120 s · Oracle-G1 native ~300 s (incl. the
  335 MB GraalVM download) · Rust ~36 s.
- `native-image` peaked ~2.6 GiB during compile → Docker needs ≥ ~6 GiB or it
  OOM-kills the build.

---

## Cross-cutting issues we hit

### Sub-agent bash sandbox
All three agents reported that `cargo` / `gradle` / `docker` invocations were blocked even with `dangerouslyDisableSandbox: true`. Result: every port is unverified at the build level. Code was authored by inspection, not by compile loop. Means first builds locally will likely surface small fixes.

### Rust toolchain age
Local rustc was 1.73.0 (Oct 2023), predating Rust edition 2024 (stabilised in 1.85, Feb 2025). Transitive deps in the new ports use edition 2024, so `cargo test` failed with:
```
this version of Cargo is older than the `2024` edition
```
Fix: `rustup update stable` → 1.95.0.

### Decisions revised mid-flight
- **Circuit breakers dropped** after initial agents were briefed to include them. Forced a stop-and-restart of both branded-tracking ports.
- **Diktat dropped** for the Quarkus port. Original brief didn't mention Diktat (the source uses it); user asked us to be explicit about not adding it.

---

## Files in `apps/`
```
apps/
├── branded-tracking-app/                          (wrapper: the perf comparison)
│   ├── README.md
│   ├── bench.sh
│   ├── branded-tracking-app-spring-boot/          (source, lightly patched for local boot)
│   ├── branded-tracking-app-quarkus/              (port 2)
│   └── branded-tracking-app-rust/                 (port 1)
└── other/
    ├── posten-parcel-api/                         (source, untouched)
    ├── posten-parcel-api-rust/                    (port 3, the big one)
    └── app-news-service/                          (analysed in apps.md, NOT yet ported)
```

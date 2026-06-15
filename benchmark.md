# Benchmark setup — parcel-api resource comparison

Reference for how the `parcel-api` benchmark in this repo is run and measured.
Written so a slide deck can describe the methodology accurately to an audience.
The harness is [`parcel-api/bench.sh`](parcel-api/bench.sh); results
land in `parcel-api/bench-results/<date>-<n>/` (latest fair run:
`bench-results/2026-06-04-fixed1g/`).

## What is benchmarked

The same small HTTP service implemented across several stacks. Each serves
`GET /parcel-api/parcel`, which returns **100 parcels (~396 KB of JSON)**.
On **every request** the service re-parses all 100 parcels from in-memory JSON,
recomputes a set of derived "features", and re-serializes the response — so the
benchmark stresses the runtime's **JSON parse + serialize path and its memory /
GC behaviour**, which is the comparison we care about.

> The implementations were verified to do identical per-request work and to
> return the same 100 parcels with field-for-field identical content (395 706
> bytes total — same fields, same key order, same number formatting). The only
> difference is the order parcels appear in the array: **every port deliberately
> shuffles the array on each request** (so the serializer can't memoize a
> constant response), so even two responses from the same port differ in order.
> Sorted by parcel number, all seven hash identically. Order changes neither the
> byte count nor the per-request work. The throughput differences are real
> runtime differences, not workload differences.

Variants measured (one bar each unless noted):

| Variant | Stack |
|---|---|
| `spring-boot-g1` | Spring Boot (Kotlin) on the JVM, G1 GC |
| `quarkus-jvm-g1` | Quarkus (Kotlin) on the JVM, G1 GC |
| `quarkus-native-g1` | Quarkus compiled to a GraalVM native binary, **Oracle GraalVM, G1 GC** (`--gc=G1`) |
| `rust` | Rust (axum + serde) native binary |

Notes for the deck:
- The JVM images are also run under ParallelGC (`*-parallel` variants); not shown
  on every slide.
- **The native bar is Oracle GraalVM with G1 GC** — label it as such, not just
  "Quarkus native". The free/community default (`quarkus build --native` on
  Mandrel) uses **Serial GC** and is ~3× slower on this workload; it is omitted
  from the slides for simplicity but exists in the harness as `quarkus-native`.
  Don't let the audience assume the shown number is the out-of-the-box default.

## Where it runs

- **Each variant runs as a single Docker/OCI container** (built per stack;
  JVMs on a JRE base, native/Rust as standalone binaries on a minimal base).
- Host: a containerised Linux VM on Apple Silicon (aarch64); the engine is
  Podman with a Docker-compatible CLI. The VM has 5 vCPUs and 8 GiB, but the
  **container limits below are what each service actually sees**, not the VM.

### Container limits (identical for every variant)

| Limit | Value | `docker run` flag |
|---|---|---|
| **CPU** | **3 cores** | `--cpus 3` |
| **Memory** | **1 GiB**, swap disabled | `--memory 1g --memory-swap 1g` |

The memory limit is the important one for fairness. If **no** limit is set, each
runtime sees the whole VM's RAM and the garbage collectors size their heaps to a
percentage of *that* — making memory numbers incomparable across machines. We
therefore pin **1 GiB**, mirroring a realistic Kubernetes pod limit.

**Consequence to explain on a slide:** the JVM GCs and GraalVM native both
default their **max heap to ~25 % of the container memory limit** — i.e. ~256 MiB
at a 1 GiB limit. That is why peak-memory numbers cluster the way they do, and
why an *unconstrained* container makes a GC'd runtime look like a memory hog
(it isn't — it just expands into whatever RAM it's allowed). Rust has no GC, so
its memory is workload-bound regardless of the limit.

## How load is generated

- Load tool: **[vegeta](https://github.com/tsenart/vegeta)** (`peterevans/vegeta`
  image), running in its own container on a shared Docker bridge network so it
  reaches the service by container DNS (no host networking in the path).
- Target: `GET http://<service>:8080/parcel-api/parcel` (no request body).
- vegeta runs **closed-loop at max rate** (`-rate=0` with a fixed worker count),
  i.e. N persistent workers each fire the next request as soon as the previous
  one returns — so throughput is bounded by the service, not by a fixed send rate.

Load is applied in **three phases** against the already-warm container:

| Phase | Workers (concurrency) | Duration |
|---|---|---|
| 1 — warmup | 1 (sequential) | 5 s |
| 2 — ramp | 100 | 15 s |
| 3 — steady | 200 | 30 s |

The JVMs JIT-compile their hot path during phases 1–2; phase 3 is where
steady-state throughput, allocator behaviour and GC pressure show up. Reported
`Req/s` is the **phase-3 steady** rate.

## How it is measured

### Cold-start time
For each variant the harness does **10 runs** of: `docker run` the container →
poll `GET /parcel-api/parcel` until it returns 200 → record the elapsed
wall-clock time. Reported as **median and p95** of the 10 runs. Readiness polls the
real workload endpoint (there is no separate health endpoint), so cold start is
genuinely *time to first served request*: for the JVMs this includes lazily
class-loading and JIT-ing the serialize path on that first call, which is why their
cold start is meaningfully higher than a trivial health-ping would show. Note this
includes container-creation overhead, not just the process's own startup (e.g.
a native binary logs "started in 0.013 s" but the measured cold start is ~90 ms).

### Memory (RSS) and CPU — custom sampler
A background sampler polls the **Docker Engine stats API** over its unix socket
(`/containers/<id>/stats`) at **1 Hz** and writes a CSV row per second of:

- **RSS (MiB)** = container memory `usage − inactive_file` (resident working set,
  excluding reclaimable page cache).
- **CPU (cores)** = Δ`cpu_usage.total_usage` (ns) ÷ Δ wall-clock (ns) between
  samples — i.e. average cores consumed in that 1 s window (so `2.5` means 2.5 of
  the 3 allotted cores were busy).

From this series the table reports RSS and CPU at three points:

- **Idle** — 3 s after the container is up, before any load.
- **Warm** — after phase 1 (warmup).
- **Peak** — after phase 3 (steady, full concurrency).

### Throughput
vegeta records every request's result. The harness re-bins these into a
**per-second count of successful (HTTP 200) responses only**, across all three
phases, and reports total requests and the phase-3 steady `Req/s`. (Non-200
responses are excluded from the rate, so a struggling service can't inflate its
number with errors.)

### Artifact / runtime / image size
Three size figures per variant, all **uncompressed on-disk** (`docker image inspect
--format '{{.Size}}'` for the image; summed file bytes for the rest):

- **Image** — the full container image total: language runtime + OS base + artifact +
  `parcel-data` (0.56 MiB / 584,824 B, identical in every image). This is the whole
  thing that lands in the pod, Alpine/distroless base included.
- **Artifact** — the deployable: jar / `quarkus-app` dir / native or compiled binary /
  app JS.
- **Runtime** — the **language runtime / VM** the image ships as a separate component,
  measured directly from its directory in the image: the JRE (`/opt/java/openjdk`,
  186.9 MiB) for the JVM ports, the Node.js runtime (`/nodejs`, 115.1 MiB) for Node.
  Go, Rust, C, and quarkus-native compile the runtime **into the binary** (so it's
  already counted in `Artifact`) — there's no separate runtime to measure, shown as
  "—". The remaining Image bytes not in Artifact/Runtime/`parcel-data` are the OS base
  (Alpine for the JVM ports, distroless Debian for Node/Go/Rust/C, UBI micro for native).

## Columns in the results table

`Cold start (median / p95)` · `Idle RSS` · `Warm RSS` · `Peak RSS` ·
`Warm CPU` · `Peak CPU` · total `Requests` · steady `Req/s` ·
`Artifact (MiB)` · `Runtime (MiB)` · `Image (MiB)`.

## Caveats worth stating to the audience

- **Re-parse-per-request is a deliberate stressor.** A real service would cache
  parsed objects; this one re-decodes ~396 KB every request to put JSON +
  allocation cost on the hot path. It exaggerates the gap between GC'd runtimes
  and Rust. A caching service would narrow it.
- **Cold start includes container overhead**, so the absolute numbers are
  "time to first served request from `docker run`", not the process's internal
  init time.
- **The native bar uses Oracle GraalVM's G1 GC.** The free/community default
  (Mandrel, Serial GC) does ~3× fewer requests/s on this allocation-heavy workload
  — it's omitted from the slides for simplicity, so the native bar shown is native
  at its *tuned best*, not its default. Worth one sentence so it isn't mistaken for
  what `quarkus build --native` produces out of the box.
- **`Req/s` is closed-loop max throughput at fixed concurrency**, not a latency
  SLA. For p99/latency claims, a separate latency-focused run would be needed.

## Results

Measured at `--cpus 3`, `--memory 1g`, **all variants in a single run** from
`bench-results/2026-06-15-1/`. Every port serves a bare JSON array of parcels over
`GET /parcel-api/parcel`. Build times are **freshly measured this run** (`BUILD=1`,
clean rebuild per image). One G1 row is shown per JVM stack; the `*-parallel` JVM
variants and the Serial-GC `quarkus-native` also ran (see Notes for the deck) but
aren't tabled here.

| Variant | Build | Cold start (med / p95) | Idle RSS | Warm RSS | Peak RSS | Warm CPU | Peak CPU | Steady req/s | Artifact | Runtime | Image |
|---|---|---|---|---|---|---|---|---|---|---|---|
| spring-boot-g1 | 9 s | 2.68 / 2.86 s | 179 MiB | 210 MiB | 438 MiB | 1.29 | 2.44 | 249 | 31.4 MiB | 186.9 MiB | 247 MiB |
| quarkus-jvm-g1 | 24 s | 1.61 / 1.70 s | 129 MiB | 201 MiB | 381 MiB | 1.38 | 2.55 | 337 | 27.3 MiB | 186.9 MiB | 243 MiB |
| quarkus-native-g1 | 287 s | 0.13 / 0.15 s | 37 MiB | 69 MiB | 278 MiB | 1.05 | 2.61 | 277 | 87.0 MiB | — | 114 MiB |
| node | 3 s | 0.26 / 0.35 s | 96 MiB | 86 MiB | 162 MiB | 0.83 | 2.38 | 381 | 7.0 MiB | 115.1 MiB | 158 MiB |
| go | 8 s | 0.13 / 0.19 s | 5.7 MiB | 7.7 MiB | 24.6 MiB | 0.82 | 2.61 | 404 | 5.4 MiB | — | 9.0 MiB |
| rust | 70 s | 0.11 / 0.11 s | 2.8 MiB | 3.6 MiB | 11.6 MiB | 0.56 | 1.83 | 923 | 13.8 MiB | — | 48 MiB |
| c | 11 s | 0.11 / 0.15 s | 2.0 MiB | 2.1 MiB | 2.3 MiB | 0.49 | 0.69 | 917 | 0.3 MiB | — | 35.5 MiB |

**Where the runtime lives.** The `Runtime` column is the language runtime / VM the
image ships as a *separate* component. Only Node and the JVM have one: the JVM ports
ship a **186.9 MiB JRE** and Node a **115.1 MiB Node.js runtime** alongside a tiny app.
Go, Rust, C, and quarkus-native compile the runtime **into the binary** (the `Artifact`
column) — Go and Rust statically, C as native code over libc, quarkus-native baking the
GraalVM substrate VM into its 87 MiB binary — so there's no separate runtime ("—"). The
rest of each image is the OS base: Alpine (~28 MiB) under the JVM ports, distroless
Debian under Node/Go/Rust/C, UBI micro (~26 MiB) under native. So: JVM/Node = big shipped
runtime + small app; Go/Rust/C/native = runtime inside the binary, no separate VM.

Headlines: **startup** — native/Rust/C ~0.1 s, Go 0.13 s, Node 0.26 s, Quarkus-JVM
1.6 s, Spring Boot 2.7 s (the JVMs rose because readiness now hits the real
`GET /parcel`, not a trivial ping — see Cold-start time). **Idle memory** — C 2.0 MiB
≈ Rust 2.8 ≈ Go 5.7 ≪ native 37 ≪ Node 96 ≪ Quarkus-JVM 129 ≪ Spring Boot 179. **Peak
memory under load** — C 2.3 MiB ≪ Rust 12 ≪ Go 25 ≪ Node 162 ≪ native 278 ≪
Quarkus-JVM 381 ≪ Spring Boot 438. **Throughput** — C 917 ≈ Rust 923, both ≫ Go 404 ≈
Node 381 ≫ the JVM/native band (249–337).

**C caveat:** C ran neck-and-neck with Rust on throughput (917 vs 923) while peaking
at only **0.69 of 3 cores** — the single-threaded h2o event loop wasn't CPU-saturated,
so 917 req/s is a *floor*, not its ceiling, and its CPU-per-request isn't cleanly
comparable to the others. The
three-way talk lineup is Spring Boot / Quarkus / Rust; Go, C, and Node are extra
reference points.

**Node:** runs Fastify behind the `node:cluster` module, forking one worker per core
(3 under `--cpus 3`). That reaches **2.38 of 3 cores and 381 req/s** — ~2.8× the
single-process `node:http` version it replaced (137 req/s at 0.96 cores; preserved in
`bench-results/2026-06-12-node-c-go/`), now genuinely multi-core like Go and the JVMs.
The cost is memory: three full Node processes push idle RSS to 96 MiB and peak to
162 MiB (vs 13 / 46 single-process), and cold start to 0.26 s (each worker boots
Fastify and loads its own parcel copy). RSS scales ~linearly with `WEB_CONCURRENCY`.

Build column is the **containerized image rebuild** (deps/toolchain cached, app
recompiled): Node ≈ 3 s; Spring Boot ≈ 9 s and Quarkus-JVM ≈ 24 s (`gradlew clean
build`); Go ≈ 8 s; C ≈ 11 s warm (~30 s cold, when Docker apt-installs the h2o
toolchain before that layer is cached); Rust ≈ 70 s (cargo-chef caches the dependency
layer, so only the app crate recompiles — a *local* incremental `cargo build` is
seconds); native ≈ 5 min (`native-image` re-analyses the whole closure every time and
can't be cached). Read it as orders of magnitude.

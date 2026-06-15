# parcel-api benchmark — Node / C / Go (2026-06-12)

Build-and-bench pass for the three "native-ish" reference ports: **Node.js** (new —
first time benched), plus **C** and **Go** re-run to cross-check against
[`../2026-06-11-clean/results.md`](../2026-06-11-clean/results.md). All three were
rebuilt fresh (`BUILD=1`) and benched into this one directory.

## Conditions

- Same methodology as [`benchmark.md`](../../../benchmark.md):
  `--cpus 3`, `--memory 1g`, 10 cold-start runs, then warmup (1 conn, 5 s) →
  ramp (100, 15 s) → steady (200, 30 s) load via vegeta on a shared bridge.
- Images rebuilt immediately before the run (unique `APP_BUILD_ID` per image).
- All ports return the **same 100 parcels, field-for-field identical, 395 706 bytes**;
  they differ only in array order, because **every port deliberately shuffles the
  array on each request** (so the serializer can't memoize a constant response) —
  which changes neither the byte count nor the per-request work. See the
  payload-equivalence check below.

## Results

| Variant | Build | Cold start (med / p95) | Idle RSS | Warm RSS | Peak RSS | Warm CPU | Peak CPU | Steady req/s | Artifact | Image |
|---|---|---|---|---|---|---|---|---|---|---|
| c | 8 s | 0.101 / 0.112 s | 1.5 MiB | 2.8 MiB | 2.9 MiB | 0.42 | 0.69 | 944 | 0.3 MiB | 35.5 MiB |
| go | 10 s | 0.089 / 0.102 s | 2.7 MiB | 8.7 MiB | 31.6 MiB | 0.79 | 2.63 | 383 | 5.4 MiB | 9.1 MiB |
| node | 5 s | 0.117 / 0.235 s | 13.4 MiB | 33.7 MiB | 45.8 MiB | 0.82 | 0.96 | 137 | <0.1 MiB | 149.7 MiB |

Cold start: all three ~0.1 s (Go 0.089 s fastest; Node's p95 0.235 s is V8 warm-up
jitter). Idle memory: C 1.5 ≪ Go 2.7 ≪ Node 13.4 MiB. Peak memory under load:
C 2.9 ≪ Go 31.6 ≪ Node 45.8 MiB. Throughput: **C 944 ≫ Go 383 ≫ Node 137 req/s**.

**Node is single-threaded.** Peak CPU never passed **0.96 of 3 cores** — one event
loop doing the ~396 KB JSON re-encode per request is the bottleneck. 137 req/s is
what a default single-process `node server.js` does on this workload, not a 3-core
ceiling; `cluster`/`worker_threads` across the 3 cores would lift it, but that isn't
what running the script out of the box gives you. Its artifact is 13 KB of
dependency-free JS — the 149.7 MiB image is the distroless Node 22 runtime, not the app.

**C caveat (unchanged from the clean run):** C led throughput while peaking at only
**0.69 of 3 cores** — the single-threaded h2o accept loop wasn't CPU-saturated, so
944 req/s is a *floor*, and its CPU-per-request isn't cleanly comparable to the others.

## Cross-check vs `2026-06-11-clean`

| Metric | Variant | 2026-06-11-clean | This run |
|---|---|---|---|
| Steady req/s | c | 945 | 944 |
| | go | 420 | 383 |
| Idle RSS | c | 1.5 MiB | 1.5 MiB |
| | go | 2.3 MiB | 2.7 MiB |
| Peak RSS | c | 2.9 MiB | 2.9 MiB |
| | go | 37.3 MiB | 31.6 MiB |

C reproduces essentially exactly. Go lands within run-to-run variance (req/s ~9 %
lower, peak RSS a touch lower — GC timing). Node had no prior baseline.

## Payload-equivalence check (all 7 benched ports)

Each benched port was hit once and its `POST /v1/parcel` body compared:

| | bytes | parcels | array order |
|---|---|---|---|
| spring-boot, quarkus-jvm, native-g1, rust, go, c, node | 395 706 (all) | 100 (all) | **7 distinct orders** |

Sorted by `parcelNumber` (object key order preserved), **all seven hash identically** —
same set, field-for-field identical content, same key order and number formatting. The
only wire difference is the order parcels appear in the array, because **every port
deliberately shuffles the array on each request** (`shuffle` in Node, `rand.Shuffle` in
Go, `qsort`+`shuffle_indices` in C, `order.shuffle` in Rust, `.shuffled()` in the two
Kotlin ports) — so even two responses from the *same* port differ in order. This does
not affect byte count, per-request parse/serialize work, or the comparison. (This
corrects the earlier "wire-format byte-identical" wording, which held for content but
not for raw array order — the shuffle makes raw output non-deterministic in every port.)

## Build times (warm caches)

| Variant | Build |
|---|---|
| node | 5 s |
| c | 8 s |
| go | 10 s |

Dependency/toolchain layers were already cached, so read these as orders of
magnitude. C's includes apt-installing the h2o toolchain inside the image.

## Artifacts

`bench-graph.png` plus per-variant `*.csv` (1 Hz RSS/CPU samples), `*.rps.csv`
(per-second throughput), `*.phases.txt`, and the full `run.log` in this directory.

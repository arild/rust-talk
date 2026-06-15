# parcel-api benchmark — Node parallelized (Fastify + cluster, 2026-06-12)

Re-bench of the **Node** port after switching it from single-process `node:http` to
**Fastify behind the `node:cluster` module** (one worker per core — 3 under
`--cpus 3`). Goal: use all the cores like Go/Rust/the JVMs do, instead of being
pinned to one event loop like the C port.

## Conditions

- Same methodology as [`benchmark.md`](../../../benchmark.md):
  `--cpus 3`, `--memory 1g`, 10 cold-start runs, then warmup (1 conn, 5 s) →
  ramp (100, 15 s) → steady (200, 30 s) load via vegeta.
- Image rebuilt fresh (`BUILD=1`). Wire output is still payload-identical to the
  other ports (same 100 parcels, same fields/key order/formatting; array shuffled
  per request); confirmed against Rust before the run (same sorted hash).

## Results — single-process vs Fastify + cluster

| Node variant | Cold start (med / p95) | Idle RSS | Warm RSS | Peak RSS | Warm CPU | Peak CPU | Steady req/s | Artifact | Image |
|---|---|---|---|---|---|---|---|---|---|
| `node:http`, 1 process (prior) | 0.117 / 0.235 s | 13.4 MiB | 33.7 MiB | 45.8 MiB | 0.82 | 0.96 | 137 | 13 KB | 149.7 MiB |
| **Fastify + cluster, 3 workers** | 0.297 / 0.354 s | 93.1 MiB | 94.1 MiB | 168.5 MiB | 0.81 | **2.25** | **304** | 7.0 MiB | 158.3 MiB |

Clustering lifts Node from **0.96 → 2.25 of 3 cores** and **137 → 304 req/s (2.2×)** —
now genuinely multi-core, landing between Quarkus-native (282) and the JVM bars
(318–350) on throughput.

The cost is **memory and startup**: three full Node processes (each with its own copy
of the parcel data and a Fastify instance) push idle RSS from 13 → 93 MiB and peak
from 46 → 169 MiB, and cold start from 0.12 → 0.30 s (each worker boots Fastify and
loads data). RSS scales roughly linearly with worker count (`WEB_CONCURRENCY`).

Build dropped to ~2 s warm because the `npm ci` layer (fastify) is cached; a cold
build that re-runs `npm ci` is ~10–15 s.

## Notes

- Worker count = `availableParallelism()`, which resolved to **3** inside the
  `--cpus 3` container, so the fork count matched the CPU cap automatically.
  Overridable via `WEB_CONCURRENCY`.
- C remains single-threaded by design, so Node is no longer a like-for-like
  "single-core floor" comparison with C.

## Artifacts

`bench-graph.png`, `node.csv` (1 Hz RSS/CPU), `node.rps.csv` (per-second throughput),
`node.phases.txt`, and `run.log` in this directory.

# parcel-api benchmark — clean re-run (2026-06-11)

Re-run after the dead-code cleanup (removed validator, auth filter, `ParcelRequest`
binding, and the `ParcelData`/`ParcelDataResponse` wrapper). **All six ports now
return a bare JSON array of parcels** — the response shape is identical across
variants, so this comparison is apples-to-apples. (The four JVM/Rust variants were
benched first; Go and C were added in a second pass into the same run directory.)

## Conditions

- Same methodology as [`benchmark.md`](../../../benchmark.md):
  `--cpus 3`, `--memory 1g`, 10 cold-start runs, then warmup → ramp → 30 s steady
  load (200 connections) via vegeta on the same bridge network.
- Images rebuilt fresh from current code immediately before the run (`--no-cache`
  for the JVM images, unique `APP_BUILD_ID` for Rust/native/Go/C).
- Machine **quiet** during the run (load average ~4–10, no competing process pegged
  a core). Earlier runs the same day were discarded because runaway background
  processes were saturating the CPU.
- Native = Oracle GraalVM, G1 GC (`quarkus-native-g1`).

## Results

| Variant | Cold start (med / p95) | Idle RSS | Warm RSS | Peak RSS | Warm CPU | Peak CPU | Steady req/s | Artifact | Image |
|---|---|---|---|---|---|---|---|---|---|
| spring-boot-g1 | 1.896 / 1.996 s | 172.4 MiB | 213.5 MiB | 445.8 MiB | 1.32 | 2.51 | 318 | 31.4 MiB | 247.4 MiB |
| quarkus-jvm-g1 | 0.669 / 0.758 s | 79.3 MiB | 190.0 MiB | 381.7 MiB | 1.50 | 2.57 | 350 | 27.3 MiB | 243.4 MiB |
| quarkus-native-g1 | 0.103 / 0.114 s | 6.2 MiB | 59.8 MiB | 278.7 MiB | 1.11 | 2.64 | 282 | 87.0 MiB | 113.6 MiB |
| go | 0.088 / 0.103 s | 2.3 MiB | 7.4 MiB | 37.3 MiB | 0.83 | 2.43 | 420 | 4.7 MiB | 8.3 MiB |
| rust | 0.097 / 0.112 s | 2.0 MiB | 6.9 MiB | 12.2 MiB | 0.60 | 1.64 | 860 | 13.8 MiB | 48.4 MiB |
| c | 0.103 / 0.117 s | 1.5 MiB | 2.8 MiB | 2.9 MiB | 0.43 | 0.70 | 945 | 0.3 MiB | 35.5 MiB |

Headlines hold: **startup** native/Go/Rust/C ~0.1 s ≪ Quarkus-JVM 0.67 s ≪ Spring
Boot 1.9 s. **Idle memory** C 1.5 MiB ≈ Rust 2 ≈ Go 2.3 ≪ native 6 ≪ Quarkus-JVM 79
≪ Spring Boot 172. **Peak memory under load** C 2.9 MiB ≪ Rust 12 ≪ Go 37 ≪ native
279 ≪ Quarkus-JVM 382 ≪ Spring Boot 446. **Throughput** C 945 ≈ Rust 860 ≫ Go 420
≫ best JVM/native 350.

**Caveat on C:** peak CPU was only **0.70 of 3 cores** while leading on throughput —
the h2o event loop wasn't CPU-saturated, so C's 945 req/s is a *floor* (the
closed-loop load / single-thread accept appears to be the limit, not C's CPU). Its
CPU-per-request can't be cleanly derived from this run. Rust, by contrast, spread
across tokio worker threads and used 1.64 cores for its 860 req/s.

## Build times (image rebuild, warm dependency caches)

| Variant | Build |
|---|---|
| c | 32 s |
| go | 8 s |
| spring-boot | 7 s |
| quarkus-jvm | 8 s |
| rust | 61 s |
| quarkus-native-g1 | 189 s |

These are faster than the published figures (Rust 119 s, native 297 s) because the
dependency layers were already cached; read them as orders of magnitude, not exact.
Go/C were built under some background load (Spotlight indexing); C's 32 s includes
apt-installing the h2o/build toolchain inside the Docker image.

## Comparison vs published baseline (`benchmark.md`, 2026-06-05)

| Metric | Variant | Published | This run |
|---|---|---|---|
| Steady req/s | spring-boot-g1 | 306 | 318 |
| | quarkus-jvm-g1 | 370 | 350 |
| | quarkus-native-g1 | 290 | 282 |
| | rust | 963 | 860 |
| Idle RSS | spring-boot-g1 | 169 MiB | 172 MiB |
| | quarkus-jvm-g1 | 91 MiB | 79 MiB |
| | quarkus-native-g1 | 6.3 MiB | 6.2 MiB |
| | rust | 2.0 MiB | 2.0 MiB |
| Peak RSS | spring-boot-g1 | 448 MiB | 446 MiB |
| | quarkus-jvm-g1 | 376 MiB | 382 MiB |
| | quarkus-native-g1 | 276 MiB | 279 MiB |
| | rust | 12.5 MiB | 12.2 MiB |

Every metric lands within run-to-run variance of the published numbers (Rust's
throughput is the widest gap, ~11% lower, still ~2.5× the field). **Conclusion:
the dead-code cleanup did not materially change resource usage or throughput** —
as expected, since the removed wrapper was a handful of scalar fields against
~100 parcels (~396 KB) of payload.

## Artifacts

`bench-graph.png` plus per-variant `*.csv` (1 Hz RSS/CPU samples), `*.rps.csv`
(per-second throughput), and `*.phases.txt` in this directory.

# Cloud savings estimate — leaner runtimes for Posten App Experience

Estimate of the Kubernetes / Azure (AKS) cost impact of migrating the team's
Spring-Boot-on-JVM services to a leaner runtime (Quarkus-JVM, Quarkus-native, or
Rust), based on the resource deltas measured in the `parcel-api` benchmark
([`benchmark.md`](benchmark.md)) applied to the team's **actual**
production deploy specs in `posten-app-experience/*/deploy/values-production.yaml`.

> **This is an estimate with explicit assumptions, not a quote.** Read the
> *Reduction factors* and *Caveats* sections before quoting any number. The
> headline figures are deliberately conservative.

---

## 1. Benchmark deltas (the input)

Per-instance resource usage, identical workload, `--cpus 3`, `--memory 1g`
(full table + methodology in [`benchmark.md`](benchmark.md)):

| Variant | Cold start | Idle RSS | Peak RSS (load) | Steady req/s |
|---|---|---|---|---|
| spring-boot-g1 | 1.90 s | 172 MiB | 446 MiB | 318 |
| quarkus-jvm-g1 | 0.67 s | 79 MiB | 382 MiB | 350 |
| quarkus-native-g1 | 0.10 s | 6.2 MiB | 279 MiB | 282 |
| rust | 0.10 s | 2.0 MiB | 12.2 MiB | 860 |

**Caveat carried into the estimate:** this is a *trivial stub* (re-parses ~396 KB
per request, no real domain state). The 37× memory gap (12 vs 446 MiB) is a
**framework-floor-only** number. Real services carry working data a rewrite does
*not* shrink, so the realistic per-service reduction is much smaller (see §4).

---

## 2. Current production footprint

25 deployed services, **all JVM / Spring Boot** (24 Kotlin, `postenid` is Java 25),
**51 baseline pods, no autoscaling** (fixed `replicaCount`). Values are the
effective production config (production overrides base `values.yaml`). The charts
use Posten's custom keys (`podMemoryRequests`/`podMemoryLimits`/`podCpuRequests`,
plus JVM `jvmXms`/`jvmXmx`); **no service sets a CPU limit**, and several omit
memory and/or CPU requests (the translating Helm chart is shared/remote and not in
the workspace, so unset = unknown, **not** defaulted).

| Service | Stack | Prod replicas | CPU request | Mem request | Mem limit |
|---|---|---|---|---|---|
| anti-crime-lookup-service | Spring Boot (Kotlin) | 1 | not set | not set | not set |
| app-data-feeder | Spring Boot (Kotlin) | 2 | 900m | 2200Mi | 2200Mi |
| app-news-service | Spring Boot (Kotlin) | 2 | 150m | 780Mi | 780Mi |
| app-notification | Spring Boot (Kotlin) | 2 | 100m | 1200Mi | 1200Mi |
| backoffice-app | Spring Boot (Kotlin) | 2 | 100m | not set | not set |
| bankid-service | Spring Boot (Kotlin) | 2 | 200m | 840Mi | 840Mi |
| branded-tracking-app | Spring Boot (Kotlin) | 2 | 100m | not set | not set |
| customer-service-app | Spring Boot (Kotlin) | 2 | 100m | 740Mi | 740Mi |
| delivery-instruction-app | Spring Boot (Kotlin) | 2 | 150m | 1600Mi | 1600Mi |
| mobile-configuration | Spring Boot (Kotlin) | 2 | not set | 768Mi | 768Mi |
| mobile-order | Spring Boot (Kotlin) | 2 | 150m | 1024Mi | 1536Mi |
| pakkeboks | Spring Boot (Kotlin) | 2 | 400m | 1080Mi | 1080Mi |
| parcel-locker-web | Spring Boot (Kotlin) | 2 | 100m | not set | not set |
| parcel-sharing-service | Spring Boot (Kotlin) | 2 | 250m | 880Mi | 880Mi |
| pat-glow-delivery | Spring Boot (Kotlin) | 2 | not set | not set | not set |
| posten-parcel-api | Spring Boot (Kotlin) | 3 | 1300m | 2612Mi | 2612Mi |
| posten-user-contact-api | Spring Boot (Kotlin) | 2 | 200m | 760Mi | 760Mi |
| postenid | Spring Boot (Java 25) | 3 | 900m | 1100Mi | 1100Mi |
| postenid-user | Spring Boot (Kotlin) | 2 | 250m | 1024Mi | 1024Mi |
| return | Spring Boot (Kotlin) | 2 | not set | 768Mi | 768Mi |
| rewards | Spring Boot (Kotlin) | 2 | 256m | 1800Mi | 1800Mi |
| rewards-backoffice | Spring Boot (Kotlin) | 2 | 100m | not set | not set |
| sende | Spring Boot (Kotlin) | 2 | not set | 1024Mi | 2048Mi |
| user-address | Spring Boot (Kotlin) | 2 | 150m | 711Mi | 711Mi |
| user-feedback | Spring Boot (Kotlin) | 2 | 120m | 960Mi | 960Mi |

**Totals (replica baseline = production `replicaCount`; 1 GiB = 1024 MiB):**

| Metric | Value (services that set the key) |
|---|---|
| Σ memory request × replicas | **47,732 MiB ≈ 46.6 GiB** (19 services) |
| Σ memory limit × replicas | 50,804 MiB ≈ 49.6 GiB (19 services) |
| Σ CPU request × replicas | **14.15 cores** (20 services) |
| Σ CPU limit | not computable — no CPU limits set anywhere |

Effective totals are **higher** — 6 services set no memory and 5 set no CPU
request here, so they draw the shared chart's (unknown) defaults. Treat current
reserved memory as **~50–55 GiB**.

**Right-sizing candidates (large requests, check before any rewrite):**
`posten-parcel-api` 2612Mi×3, `app-data-feeder` 2200Mi×2, `rewards` 1800Mi×2,
`delivery-instruction-app` 1600Mi×2, `mobile-order`/`sende` 1024–2048Mi×2.

Excluded (not deployed app services): `infrastructure`, `pae-toolkit` (Gradle
plugin), `postenid-client` (client lib), `event-hub-encryption` (no chart), and
vendored reference copies under `temp-ref/` / `.claude/` / `reference/`.

---

## 3. How cost is driven here

- AKS node (VM) cost is driven by **bin-packing pods by their resource requests**.
- These services request **~50 GiB memory vs ~14 cores CPU** → they pack
  **memory-bound**. Memory reservation is the cost lever; CPU is not the binding
  constraint *today* and won't change node count on its own (after a Rust
  migration that flips — see the CPU subsection in §5).
- Most set memory **request = limit** → Guaranteed QoS → the full memory is
  reserved 24/7 (not just under load). So the request total *is* the reserved cost.
- The savings lever is therefore: **how low can the memory request go** on a leaner
  runtime.

---

## 4. Reduction factors (the key assumption)

A JVM pod's memory request is dominated by **fixed runtime overhead** — heap sized
with ~2–3× GC headroom, metaspace (~80–150 MiB), thread stacks, direct buffers,
framework (~50–100 MiB) — a ~250–400 MiB floor *before* real working data. That
floor is what the leaner runtimes remove; the working data itself is unchanged.

Conservative per-service memory-request reduction factors used below (NOT the
benchmark's 34× floor-only figure):

| Target | Factor | Rationale |
|---|---|---|
| Quarkus-JVM | ~1.3–1.8× | same JVM, leaner framework + lower baseline; still GC + metaspace |
| Quarkus-native | ~2.5–4× | AOT, no metaspace/JIT, tiny baseline; G1 heap still sizes to ~25% of the memory limit |
| Rust | ~3–6× | no GC headroom, ~5–15 MiB runtime; request ≈ working-set × ~1.2 |

Data-heavy services land at the low end of each range; light request/response
transformers (like the benchmark stub) land higher.

---

## 5. Estimated savings

Based on **~50 GiB** current reserved memory. Node basis: standard AKS
`Standard_D8s_v5` (8 vCPU / 32 GiB, **~26 GiB allocatable** after kubelet/OS
reserve).

| Target | Reserved mem after | Freed | ≈ D8s_v5 nodes freed | ≈ $/yr saved (list) |
|---|---|---|---|---|
| Quarkus-JVM | ~28–38 GiB | ~12–22 GiB | ~0.5–1 | **~$2–4k** |
| Quarkus-native | ~12–20 GiB | ~30–38 GiB | ~1–1.5 | **~$4–6k** |
| **Rust** | ~8–17 GiB | **~33–42 GiB** | ~1.5–2 | **~$5–8k** |

**Headline:** moving these services to Rust could cut their reserved memory
**~70–85 % (~35–40 GiB freed)** — roughly **1.5–2 standard nodes**, on the order
of **$5–8k/year at AKS list price**. The CPU side is analyzed below — it's what
makes those freed nodes actually realizable.

### CPU savings — Rust (conservative)

Measured CPU cost per request (`Peak CPU ÷ steady req/s`, from
[`benchmark.md`](benchmark.md)):

| Variant | Peak CPU (of 3 cores) | Steady req/s | CPU per request |
|---|---|---|---|
| spring-boot-g1 | 2.51 | 318 | ~7.9 mcore·s |
| rust | 1.64 | 860 | **~1.9 mcore·s** |

On the benchmark workload Rust does **~4.1× more work per CPU-second** — at equal
traffic (318 req/s) it would burn ~0.6 cores where Spring Boot burns ~2.5.

**Conservative fleet-wide factor: ~1.5–2.5×** (NOT the measured 4.1×). The
benchmark deliberately puts allocation + GC on the hot path (§1 caveat), which
exaggerates the JVM's CPU cost. In real services much of the CPU budget goes to
I/O, DB drivers, and TLS, which a rewrite doesn't shrink — only GC cycles, JIT,
and allocation pressure go away. Same haircut logic as the memory factors in §4.

Applied to the **14.15 requested cores** (§2 — a lower bound, 5 services set no
CPU request):

| | CPU requested | Freed |
|---|---|---|
| Today (all Spring Boot) | ~14 cores | — |
| After Rust (÷ 1.5–2.5) | **~6–9.5 cores** | **~5–8 cores** |

**Why this matters even though the cluster packs memory-bound today (§3):** after
a Rust migration, reserved memory drops to ~8–17 GiB — well under one node's
~26 GiB allocatable — and **CPU becomes the binding constraint**. Left at ~14
cores, the fleet would still need ~2 nodes (~7.8 allocatable cores per D8s_v5);
at the conservative ~6–9.5 cores it fits in **~1 node** (the high end of the
range just spills over). Without the CPU reduction, the "~1.5–2 nodes freed" in
the table above would not materialize.

Priced standalone (list ≈ $4k/node-yr ÷ 8 vCPU ≈ $500/vCPU-yr), ~5–8 freed cores
≈ **~$2.5–4k/yr** — but do **not** add this to the memory headline: both savings
are realized through the same freed nodes, so quoting them separately
double-counts.

### Pricing basis
- `Standard_D8s_v5`, Linux, West Europe / Norway East, **pay-as-you-go list
  ≈ $0.46/hr ≈ $335/node/month** (≈ $4k/node/year).
- **Reserved Instances / Savings Plan (1–3 yr) typically cut 20–40 %**, and
  Posten's EA/CSP rate will differ — so realized $ is likely **below** the list
  figures above.
- Norway East runs slightly higher than West Europe; adjust ~5–15 %.

---

## 6. Reality check (read before deciding)

- **The absolute $ is modest because the footprint is small** (~50 GiB / ~14
  cores ≈ 2–3 nodes total). ~$5–8k/yr does **not** justify rewriting 25 Spring
  Boot services to Rust — that's a multi-year effort at weeks-to-months *per
  service*. On cost alone, the business case does not close.
- **Cheapest win, zero rewrite — right-size the current JVM requests first.**
  Several services reserve far more than the benchmark's real-ish Spring Boot peak
  (~450 MiB). Compare `kubectl top pods` to the requests in §2; trimming
  over-provisioned requests captures a chunk of these savings **today, for free.**
- **Quarkus-JVM = low-effort broad option** (keep Kotlin, lift the framework):
  ~25–40 % memory + much faster startup, minimal rewrite risk, same JVM ops.
- **Rust / native = selective, high-value targets** — highest-replica /
  highest-traffic services (`posten-parcel-api`, `postenid` at 3 replicas) or new
  services, where per-pod savings × replicas and the throughput/startup wins
  compound. That's the defensible "efficiency beyond the JVM" story — not "rewrite
  everything."

### Benefits the $ figure doesn't capture
- **Startup ~0.1 s vs ~2 s** → faster rollouts, viable scale-to-zero, less
  always-on headroom for traffic spikes.
- **~2.5× throughput/pod (Rust)** → fewer replicas where HA (min 2) isn't already
  the floor.
- **Higher pod density** → fewer nodes, smaller blast radius, simpler bin-packing.

---

## 7. Caveats / how to firm this up

Biggest unknowns, in order of impact:
1. **Reduction factor** — depends on each service's real working set. Data-heavy
   services save less than the ranges above; light ones save more.
2. **Unvendored shared Helm chart defaults** — 6 services set no memory here, so
   the current total is a *lower bound*; effective reserved memory is higher.
3. **Your actual Azure rate** (EA / CSP / Reserved) vs list price.
4. **Shared-cluster realization** — freed capacity only becomes $ if node count
   actually drops (cluster autoscaler / manual scale-down).

To turn this into a firm number: pull live `kubectl top pods` (actual usage vs
requests) per service, the shared chart's resource defaults, and your negotiated
VM rate.

---

*Inputs: `benchmark.md` (this repo) · `posten-app-experience/*/deploy/values-production.yaml`.
Estimate prepared 2026-06-06. Figures are approximate and conservative; validate
before quoting externally.*

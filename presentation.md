# Introduction to Rust: Efficiency Beyond the JVM

Talk outline / slide template. Fill in the body of each slide as we go.

---

## Part 1 — Introduction to Rust (≈ 25 min)

### 1. Title + intro (1 slide)
- Title, your name, team, where this talk is going.
- One-line framing: "What Rust is, why we care, and what it costs us less of than the JVM."

### 2. Why this talk (1 slide)
- Most of our backend is Spring Boot on the JVM.
- Rust is increasingly chosen by teams that pay for compute (Cloudflare, Discord, AWS).
- **AI angle:** Rust is unusually well-suited to AI coding agents — strong types and compiler errors give the agent a tight feedback loop, so it converges faster and ships fewer hidden bugs than in dynamically-typed or GC'd stacks. Worth knowing as agentic dev becomes routine. ([source](https://x.com/CtrlAltDwayne/status/2032388050584736157?s=20))
- Two questions worth asking: *what is Rust*, and *what would we get if we used it*.

### 3. What Rust is (1–2 slides)
- Systems language. Compiled, statically typed, no GC, no runtime to speak of.
- Memory safety without a garbage collector — checked at compile time.
- Zero-cost abstractions: high-level code, low-level performance.
- Ergonomics initiative — not just C++ with a borrow checker.
- Mature tooling: `cargo`, `clippy`, `rustfmt`, first-class language server.

### 4. The memory model — the headline feature (3–4 slides)
This is the section to spend time on. Audience is JVM, so contrast against GC.

- **Ownership.** Every value has exactly one owner. When the owner goes out of scope, the value is dropped. Deterministic, no GC pause.
- **Borrowing.** You can lend a value without transferring ownership. Either many readers (`&T`) or one writer (`&mut T`) — never both.
- **Lifetimes.** The compiler tracks how long references live, so dangling pointers can't compile.
- **What you get for free:** no use-after-free, no double-free, no data races, no GC pauses, predictable memory.

### 5. Code examples (4–6 slides)
Give the audience a *feel* for the language. Keep snippets short (≤15 lines), one idea per slide.

- **Hello, Rust** — `fn main()`, `println!`, types, immutability by default.
- **Pattern matching + enums** — `Option<T>`, `Result<T, E>`, exhaustive `match`. Compare to Java sealed types.
- **Ownership in 10 lines** — show a move, then show the borrow checker rejecting a use-after-move with a friendly error message.
- **Borrowing** — `&str` vs `String`, why a function takes `&str`.
- **Iterators + closures** — `.iter().filter().map().collect()`. Looks like Java streams; compiles to a tight loop with no allocations.
- **Async** — a tiny `async fn` with `tokio`, an axum handler. Shows that async is not exotic.

### 6. The ecosystem in one slide
- `cargo` for build / test / publish / docs.
- `crates.io` for libraries.
- `tokio`, `axum`, `serde`, `sqlx`, `reqwest` — what we used in the demos.
- Editor: rust-analyzer everywhere.

---

## Part 2 — Efficiency vs the JVM (≈ 25 min)

### 7. The JVM bill (1–2 slides)
- Heap + ~350 MB non-heap (metaspace, threads, code cache, DD APM, GC, direct, JVM).
- JIT warm-up cost.
- GC pauses on the p99.
- Numbers from our own services as reference.

### 8. The lineup (1 slide)
Three implementations of the same service, same endpoints, same outbound calls, same downstream stubs:

| Variant | Stack |
|---|---|
| Spring Boot | Kotlin, Spring Boot 4, Tomcat, JVM |
| Quarkus | Kotlin, Quarkus 3, RESTEasy Reactive, JVM |
| Rust | axum, tokio, reqwest |

App: branded-tracking-app — small, single endpoint, real downstream HTTP.

### 9. What we measure (1 slide)
- Container memory (RSS) at idle, after warmup.
- Container memory under load.
- Cold-start time (process start → first request served).
- Throughput (req/s at fixed concurrency).
- p50 / p95 / p99 latency.
- Image size.

### 10. Numbers — small app: branded-tracking-app (2–3 slides)
- Memory chart.
- Startup chart.
- Latency chart.
- Image-size chart.
- Read each chart out loud — don't make the audience squint.

### 11. Numbers — bigger app: posten-parcel-api (2–3 slides)
- Same charts. Bigger app amplifies the JVM tax.
- Highlight the warmup behaviour: Spring takes seconds; Rust is ready immediately.

### 12. Azure savings (1 slide)
Numbers from `savings-estimate.md` — deliberately conservative, applied to the team's actual production deploy specs (25 services, 51 pods).
- **Memory — the cost lever** (~50 GiB reserved today, pods pack memory-bound):
  - Conservative ~3–6× lower requests with Rust — *not* the benchmark's 34× floor-only gap; real working data doesn't shrink.
  - ~35–40 GiB freed ≈ 1.5–2 D8s_v5 nodes ≈ **$5–8k/yr at list price**.
- **CPU — what makes the freed nodes real**:
  - Measured ~4.1× less CPU per request (7.9 → 1.9 mcore·s); conservative fleet-wide ~1.5–2.5×.
  - ~14 → ~6–9.5 requested cores (~5–8 freed). Post-Rust, CPU becomes the binding constraint — the reduction is what lets the fleet fit ~1 node.
  - Worth ~$2.5–4k/yr standalone, but don't add it to the memory figure — same freed nodes, counting both double-counts.

### 13. What it costs (1–2 slides)
- Borrow checker learning curve. Real, but bounded — 1–2 months for a JVM dev.
- Smaller library ecosystem in some niches (Azure SDKs, enterprise Kafka, GraphQL frameworks).
- Compile times. `cargo` is fast for incremental, slow for clean release.
- Hiring pool. Smaller, but engaged.

### 14. When is it worth it? (1 slide)
- Large fleets where compute is a real cost line.
- Latency-sensitive paths where GC pauses bite.
- Memory-constrained environments (sidecars, edge, Lambdas).
- Long-lived hot paths (proxies, gateways, image pipelines).
- Probably *not* the right call for low-traffic CRUD services with rich enterprise integration needs.

### 15. Closing (1 slide)
- Rust is not a Spring Boot replacement. It's a tool for a specific class of problems we increasingly have.
- One-sentence takeaway.
- Links to the demo repo, the resource list, the ports under `parcel-api/`.

---

## Backup slides (have ready, only show if asked)
- Async runtimes (tokio vs async-std; why we picked tokio).
- `unsafe` — what it actually means; how often you need it.
- Error handling without exceptions: `Result<T, E>` and `?` operator.
- Macros — `derive` vs declarative vs procedural.
- Cargo workspaces and feature flags.
- Native images (GraalVM) for Quarkus — why we didn't pursue it.

---

## Demo plan
Two live moments, both pre-recorded as fallback:

1. **Boot all three side-by-side**, `docker stats` open, watch RSS settle.
2. **Hit each with `wrk`**, watch latency + memory under load.

Have the JSON results saved so the slides can show numbers even if the live demo misbehaves.

---

## Time budget
| Block | Minutes |
|---|---|
| Part 1 — Intro to Rust | 25 |
| Part 2 — Efficiency comparison | 25 |
| Live demo | 5 |
| Q&A | 10 |
| **Total** | **65** |

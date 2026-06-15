# Rust Talk — Efficiency Beyond the JVM

Companion repo for the talk *Introduction to Rust: Efficiency Beyond the JVM* (Posten). Several ports of the same `parcel-api` service — Spring Boot, Quarkus (JVM + native), Rust, C, Go, and Node — used to compare resource usage, startup time, and throughput on the JVM vs. native runtimes.

See [`parcel-api/README.md`](parcel-api/README.md) for how to build, run, and benchmark each variant.

## More

- [`parcel-api/README.md`](parcel-api/README.md) — build & run, bench, port-by-port background.
- [`apps-ports.md`](apps-ports.md) — conversion notes per port.
- [`presentation.md`](presentation.md) — talk outline.
- [`benchmark.md`](benchmark.md) — benchmark methodology + results (parcel-api resource comparison).
- [`resources.md`](resources.md) — reading list, plus deep dives on Rust-vs-JVM security and AI/SWE-bench.
- [`savings-estimate.md`](savings-estimate.md) — cloud-cost projection from the benchmark deltas.

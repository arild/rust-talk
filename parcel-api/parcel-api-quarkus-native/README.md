# parcel-api-quarkus-native

Quarkus port of [`parcel-api-spring-boot`](../parcel-api-spring-boot), compiled
ahead-of-time to a **GraalVM-native** binary. Used in the conference talk
*"Introduction to Rust: Efficiency Beyond the JVM"* as the native counterpart to
the JVM bar.

> Same source by design as [`../parcel-api-quarkus-jvm`](../parcel-api-quarkus-jvm).
> The JVM-vs-native comparison only holds if the code is identical, so any change
> to `src/` here must be mirrored there (and vice versa). The **only** intended
> differences between the two directories are the Dockerfile and
> [`NativeReflectionConfig.kt`](src/main/kotlin/no/posten/parcelapi/service/domain/NativeReflectionConfig.kt)
> (native-only, see below).

Behaviour mirrors the Spring Boot port byte-for-byte (modulo the `providedLastUpdated` timestamp).

## What it does

- `GET /parcel-api/parcel` — returns all 100 parcels.
- `GET /parcel-api/q/openapi` — OpenAPI document.
- `GET /parcel-api/q/swagger-ui` — Swagger UI.

Parcel JSON is read from `parcel.data.dir` (default `../parcel-data`, override with `PARCEL_DATA_DIR`). Files are loaded as raw strings at startup; every request re-parses them — same approach as the Spring Boot port so the bench measures equivalent work.

## Build the native image

The whole compile happens inside the Mandrel builder stage of
[`Dockerfile.native`](Dockerfile.native) — no host GraalVM toolchain needed,
mirroring the rust/c/go multi-stage builds. Build context is the parent so the
shared `parcel-data/` is in scope:

```bash
docker build --load -t parcel-api-quarkus-native -f parcel-api-quarkus-native/Dockerfile.native ../
docker run --rm -p 8080:8080 parcel-api-quarkus-native
```

Notes:

- **Memory:** `native-image` needs ~6 GB+. If the build is OOM-killed mid-compile,
  raise Docker Desktop's memory limit.
- **Time:** the native compile takes minutes (vs seconds for the JVM fast-jar) —
  that cost is the headline native trade-off and shows up in the bench's
  `Build (s)` column.
- **Toolchain tags** are `--build-arg`s. The app targets JDK 25 bytecode, so the
  builder JDK must be ≥ 25. If the `jdk-25` Mandrel tag isn't published for your
  platform, fall back with
  `--build-arg MANDREL_IMAGE=quay.io/quarkus/ubi9-quarkus-mandrel-builder-image:jdk-21`
  and drop `JavaLanguageVersion.of(25)` → `21` in `build.gradle.kts`.
- **Reflection:** [`NativeReflectionConfig.kt`](src/main/kotlin/no/posten/parcelapi/service/domain/NativeReflectionConfig.kt)
  registers the `service.domain` types that `StubParcelService` deserialises with
  its hand-built `ObjectMapper`. Without it the binary builds but 500s on the first
  request — those types never appear in a JAX-RS signature, so Quarkus can't
  discover them automatically. This file is native-only; the JVM sibling omits it.

## G1 variant — `Dockerfile.native-g1` (Oracle GraalVM)

The default `Dockerfile.native` uses Mandrel, which ships **only the Serial GC**.
On this allocation-heavy endpoint Serial GC runs ~140 stop-the-world collections
per second and throughput suffers (~115 req/s under load). The **G1** garbage
collector closes most of that gap (~287 req/s), but G1 for native image is
**Oracle-GraalVM-only** and is a *build-time* flag (`--gc=G1`).

`Dockerfile.native-g1` installs Oracle GraalVM from its public tarball (GFTC
licence — no registry login) on a UBI9 base and builds with `--gc=G1`. Same
source as `Dockerfile.native`; only the builder and that flag differ:

```bash
docker build --load -t parcel-api-quarkus-native-g1 -f parcel-api-quarkus-native/Dockerfile.native-g1 ../
docker run --rm -p 8080:8080 parcel-api-quarkus-native-g1
```

`GRAALVM_URL` is a `--build-arg` (defaults to `linux-aarch64`; use the
`linux-x64` tarball on Intel). Note G1 sizes its heap to **25% of the container's
memory limit**, so set a `docker run --memory` (or k8s limit) to bound RSS — left
unbounded it will happily use a multi-GB heap on a large host.

## Verification — built and benched

Both native builds work. Measured at `--cpus 3`, `--memory 1g` (see
[`../bench.sh`](../bench.sh)'s `MEM_LIMIT` and `../bench-results/`):

| | cold start | idle RSS | peak RSS | steady req/s |
|---|---|---|---|---|
| native (Serial) | ~0.09 s | 6.8 MiB | 73 MiB | 115 |
| native-g1 (Oracle) | ~0.10 s | 6.3 MiB | 276 MiB | 287 |

`NativeReflectionConfig.kt` proved sufficient — `GET /parcel` serves at 100%
in native. Gotchas hit and fixed along the way are recorded in
[`../../apps-ports.md`](../../apps-ports.md#port-4--parcel-api-quarkus-native-graalvm-aot)
(use `quarkusBuild` not `build`; UBI9 runtime base for glibc; Docker ≥ 6 GiB for
the compile).

## Run on the JVM for debugging

The project is a normal Quarkus app, so you can still run it on the JVM locally
(useful for iterating before the slow native compile):

```bash
./gradlew quarkusDev
```

## Differences from the Spring Boot port

| Concern | Spring Boot | Quarkus |
|---|---|---|
| Framework | Spring Boot 4 | Quarkus 3.30 |
| Web | Spring MVC (`@RestController`) | RESTEasy Reactive (`@Path`) |
| Auth filter | `OncePerRequestFilter` | JAX-RS `ContainerRequestFilter` (no-op-equivalent; sets a request property) |
| Validation | `@Validated` + `ConstraintViolationException` advice | Hibernate Validator + `ExceptionMapper<ConstraintViolationException>` |
| OpenAPI | `springdoc-openapi` | `quarkus-smallrye-openapi` |
| JSON nulls | `default-property-inclusion: non_null` | `quarkus.jackson.serialization-inclusion: non_null` |
| Runtime | JVM (JIT) | GraalVM native (AOT) |

## Sanity checks

```bash
curl http://localhost:8080/parcel-api/parcel
curl http://localhost:8080/parcel-api/q/openapi
```

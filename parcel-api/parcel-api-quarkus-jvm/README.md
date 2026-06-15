# parcel-api-quarkus-jvm

Quarkus port of [`parcel-api-spring-boot`](../parcel-api-spring-boot), running on the **JVM** (fast-jar). Used in the conference talk *"Introduction to Rust: Efficiency Beyond the JVM"* to compare resource usage between Spring Boot, Quarkus, and Rust.

> Its GraalVM-native sibling is [`../parcel-api-quarkus-native`](../parcel-api-quarkus-native) — **same source by design**. The comparison only holds if the code is identical, so any change to `src/` here must be mirrored there (and vice versa).

Behaviour mirrors the Spring Boot port byte-for-byte (modulo the `providedLastUpdated` timestamp).

## What it does

- `POST /parcel-api/v1/parcel` — accepts `ParcelRequest` body (`{}` works); returns all 100 parcels.
- `GET /parcel-api/check/status` — text liveness ping.
- `GET /parcel-api/check` — text health (memory + version banner).
- `GET /parcel-api/q/openapi` — OpenAPI document.
- `GET /parcel-api/q/swagger-ui` — Swagger UI.

Parcel JSON is read from `parcel.data.dir` (default `../parcel-data`, override with `PARCEL_DATA_DIR`). Files are loaded as raw strings at startup; every request re-parses them — same approach as the Spring Boot port so the bench measures equivalent work.

## Differences from the Spring Boot port

| Concern | Spring Boot | Quarkus |
|---|---|---|
| Framework | Spring Boot 4 | Quarkus 3.30 |
| Web | Spring MVC (`@RestController`) | RESTEasy Reactive (`@Path`) |
| Auth filter | `OncePerRequestFilter` | JAX-RS `ContainerRequestFilter` (no-op-equivalent; sets a request property) |
| Validation | `@Validated` + `ConstraintViolationException` advice | Hibernate Validator + `ExceptionMapper<ConstraintViolationException>` |
| OpenAPI | `springdoc-openapi` | `quarkus-smallrye-openapi` |
| JSON nulls | `default-property-inclusion: non_null` | `quarkus.jackson.serialization-inclusion: non_null` |

## Prerequisites

- JDK 25 (matches the source app's `jvmToolchain(25)`).

## Build & run (JVM)

```bash
./gradlew build -x test
java -jar build/quarkus-app/quarkus-run.jar
```

Or in dev mode (auto-reload):

```bash
./gradlew quarkusDev
```

## Docker

Build context is the parent directory `parcel-api/`:

```bash
docker build --load -t parcel-api-quarkus-jvm -f parcel-api-quarkus-jvm/Dockerfile ../
docker run --rm -p 8080:8080 parcel-api-quarkus-jvm
```

For the native build, see [`../parcel-api-quarkus-native`](../parcel-api-quarkus-native).

## Sanity checks

```bash
curl http://localhost:8080/parcel-api/check/status
curl -X POST http://localhost:8080/parcel-api/v1/parcel \
     -H 'Content-Type: application/json' -d '{}'
curl http://localhost:8080/parcel-api/q/openapi
```

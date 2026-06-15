# syntax=docker/dockerfile:1.7
#
# Build context is the parent directory (parcel-api/), so paths below are
# relative to that. Build with:
#   docker build --load -t parcel-api-c -f parcel-api-c/Dockerfile ../
# (run from parcel-api/parcel-api-c/)

FROM debian:bookworm-slim AS builder

# Build deps. ca-certificates is for the FetchContent git pull of yyjson.
# We pull h2o from apt rather than building it from source — building h2o
# from scratch dwarfs the rest of the build and isn't the variable we're
# studying here.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential cmake pkg-config git ca-certificates \
        libh2o-evloop-dev libssl-dev zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY parcel-api-c/CMakeLists.txt ./
COPY parcel-api-c/src ./src

# FetchContent pulls yyjson during configure. Configure + build are split so
# the configure layer caches independently of any source edits.
RUN cmake -B build -S . -DCMAKE_BUILD_TYPE=Release
ARG APP_BUILD_ID=stable
RUN echo "build id ${APP_BUILD_ID}" \
    && cmake --build build -j --target parcel-api \
    && cp build/parcel-api /parcel-api

# Stage the shared libs the binary actually needs (per `ldd`) under /staging
# preserving the multiarch directory layout and SONAME symlinks. libssl3 and
# libcrypto3 are bundled in distroless/cc-debian12 so we don't ship them.
RUN libdir=$(dirname $(readlink -f /lib/*-linux-gnu/libh2o-evloop.so.0.13)) \
    && mkdir -p "/staging${libdir}" \
    && cp -a "${libdir}/libh2o-evloop.so."* "/staging${libdir}/" \
    && cp -a "${libdir}/libwslay.so."* "/staging${libdir}/" \
    && cp -a "${libdir}/libz.so."* "/staging${libdir}/"

# Distroless cc gives us libc, libssl, libcrypto, libstdc++ and tzdata in a
# ~35 MiB image. We add only the three libs above, so the final image is
# dominated by the base rather than the 0.3 MiB binary.
FROM gcr.io/distroless/cc-debian12 AS runtime

COPY --from=builder /staging/ /
COPY --from=builder /parcel-api /usr/local/bin/parcel-api
COPY parcel-data /parcel-data

ENV TZ=Europe/Oslo \
    PARCEL_DATA_DIR=/parcel-data

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/parcel-api"]

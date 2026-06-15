# syntax=docker/dockerfile:1.7
#
# Build context is the parent directory (parcel-api/), so paths below are
# relative to that. Build with:
#   docker build --load -t parcel-api-go -f parcel-api-go/Dockerfile ../
# (run from parcel-api/parcel-api-go/)

FROM golang:1.26-bookworm AS builder

WORKDIR /src
COPY parcel-api-go/go.mod ./
# Pre-fetch dep cache. There are no deps yet but this keeps the layer
# pattern consistent with the Rust / JVM builders.
RUN go mod download

ARG APP_BUILD_ID=stable
COPY parcel-api-go/*.go ./
# CGO off → produce a fully-static ELF that can run under distroless/static.
# -s -w strips symbols + DWARF (~30% binary size win); -trimpath keeps the
# build reproducible regardless of host workspace layout.
RUN echo "build id ${APP_BUILD_ID}" \
    && CGO_ENABLED=0 GOOS=linux \
       go build -trimpath -ldflags="-s -w" -o /parcel-api .

FROM gcr.io/distroless/static-debian12 AS runtime

COPY --from=builder /parcel-api /usr/local/bin/parcel-api
COPY parcel-data /parcel-data

ENV TZ=Europe/Oslo \
    PARCEL_DATA_DIR=/parcel-data

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/parcel-api"]

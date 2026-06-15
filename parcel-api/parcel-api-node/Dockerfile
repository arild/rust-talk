# syntax=docker/dockerfile:1.7
#
# Build context is the parent directory (parcel-api/), so paths below are
# relative to that. Build with:
#   docker build --load -t parcel-api-node -f parcel-api-node/Dockerfile ../
# (run from parcel-api/parcel-api-node/)
#
# Dependency policy: any third-party dependency must be a version published at
# least 1 week ago (supply-chain safety) and pinned exactly, with package-lock.json
# committed so the build is reproducible via `npm ci`. Current dep: fastify.

# --- build stage: install pinned deps on a full Node image (distroless has no npm) ---
FROM node:22-bookworm-slim AS build
WORKDIR /app
COPY parcel-api-node/package.json parcel-api-node/package-lock.json ./
RUN npm ci --omit=dev --no-audit --no-fund
COPY parcel-api-node/*.js ./

# --- runtime stage: distroless Node 22, no shell, no npm ---
FROM gcr.io/distroless/nodejs22-debian12 AS runtime
WORKDIR /app

# Cache-bust hook for `bench.sh --build` (distroless has no shell, so we bust
# via an ENV layer rather than a RUN). Changing APP_BUILD_ID re-copies the source
# without re-running the cached `npm ci` layer above.
ARG APP_BUILD_ID=stable
ENV APP_BUILD_ID=${APP_BUILD_ID}

COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/package.json /app/*.js ./
COPY parcel-data /parcel-data

ENV TZ=Europe/Oslo \
    PARCEL_DATA_DIR=/parcel-data \
    NODE_ENV=production

EXPOSE 8080

CMD ["server.js"]

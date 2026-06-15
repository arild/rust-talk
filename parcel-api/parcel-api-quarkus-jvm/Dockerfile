# Build context is the parent directory (parcel-api/), so paths below are
# relative to that. Build with:
#   docker build --load -t parcel-api-quarkus-jvm -f parcel-api-quarkus-jvm/Dockerfile ../
# (run from parcel-api/parcel-api-quarkus-jvm/)
FROM eclipse-temurin:25-jre-alpine
ENV TZ="Europe/Oslo"
ENV PARCEL_DATA_DIR=/parcel-data
EXPOSE 8080

WORKDIR /app
COPY parcel-api-quarkus-jvm/build/quarkus-app/lib/      /app/lib/
COPY parcel-api-quarkus-jvm/build/quarkus-app/*.jar     /app/
COPY parcel-api-quarkus-jvm/build/quarkus-app/app/      /app/app/
COPY parcel-api-quarkus-jvm/build/quarkus-app/quarkus/  /app/quarkus/
COPY parcel-data                                        /parcel-data

ENTRYPOINT ["java", "-jar", "/app/quarkus-run.jar"]

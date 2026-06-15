# Build context is the parent directory (parcel-api/), so paths below are
# relative to that. Build with:
#   docker build --load -t parcel-api-spring-boot -f parcel-api-spring-boot/Dockerfile ../
# (run from parcel-api/parcel-api-spring-boot/)
FROM eclipse-temurin:25-jre-alpine
ADD parcel-api-spring-boot/build/libs/parcel-api-spring-boot-*.jar app.jar
ADD parcel-data /parcel-data
EXPOSE 8080
ENV TZ="Europe/Oslo"
ENV SPRING_PROFILES_ACTIVE=dev
ENV PARCEL_DATA_DIR=/parcel-data
ENTRYPOINT [ "java", "-jar", "/app.jar" ]

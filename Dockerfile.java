# syntax=docker/dockerfile:1-labs

# Stage 1: Build UI
FROM node:24-alpine AS ui-builder
ARG ANALYSIS_UI_MODULE=sdq-analysis-ui
WORKDIR /ui

# Copy dependency manifests first for a stable npm cache layer
COPY ${ANALYSIS_UI_MODULE}/package*.json ./
RUN --mount=type=cache,target=/root/.npm npm ci

# Copy the rest of the UI source and build it
COPY ${ANALYSIS_UI_MODULE}/ .
RUN npm run build

# Resolve Gradle dependencies separately so backend source changes do not
# invalidate this stage.
FROM eclipse-temurin:21-jdk AS gradle-deps
ARG ANALYSIS_API_MODULE_JAVA=sdq-analysis-api-java
WORKDIR /app

# Copy only the Gradle wrapper and project metadata first
COPY ${ANALYSIS_API_MODULE_JAVA}/gradlew ${ANALYSIS_API_MODULE_JAVA}/gradlew.bat ./
COPY ${ANALYSIS_API_MODULE_JAVA}/gradle ./gradle
COPY ${ANALYSIS_API_MODULE_JAVA}/settings.gradle ./
COPY --parents ${ANALYSIS_API_MODULE_JAVA}/./**/build.gradle ./

# Resolve dependencies using a dedicated Gradle cache mount so repeated builds reuse downloads
RUN --mount=type=cache,target=/root/.gradle/caches,sharing=locked \
    --mount=type=cache,target=/root/.gradle/wrapper,sharing=locked \
    ./gradlew dependencies --no-daemon

# Build the backend using the warmed Gradle cache and dependency metadata.
FROM gradle-deps AS backend-builder
ARG ANALYSIS_API_MODULE_JAVA=sdq-analysis-api-java
ARG APP_MODULE=spring-app

# Copy the backend source after dependency resolution; .dockerignore keeps local build and IDE dirs out
COPY ${ANALYSIS_API_MODULE_JAVA}/ .

# Copy UI build output into Spring Boot static resources
RUN rm -rf ${APP_MODULE}/src/main/resources/static
COPY --from=ui-builder /ui/dist ${APP_MODULE}/src/main/resources/static
COPY ${ANALYSIS_API_MODULE_JAVA}/entrypoint.sh /entrypoint.sh

RUN --mount=type=cache,target=/root/.gradle/caches,sharing=locked \
    --mount=type=cache,target=/root/.gradle/wrapper,sharing=locked \
    ./gradlew build :${APP_MODULE}:bootJar --no-daemon -x test
RUN test -f /app/${APP_MODULE}/build/libs/${APP_MODULE}-0.0.1-SNAPSHOT.jar

# Stage: Runtime
FROM amazoncorretto:21-alpine-jdk
ARG APP_MODULE=spring-app
LABEL authors="ratracejoe.co.uk"

# Install curl
RUN apk --no-cache add curl

COPY --from=backend-builder /app/${APP_MODULE}/build/libs/${APP_MODULE}-0.0.1-SNAPSHOT.jar app.jar

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 CMD curl -f http://localhost:8080/api/actuator/health || exit 1

COPY --from=backend-builder /entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
# Set the entrypoint
ENTRYPOINT ["/entrypoint.sh"]

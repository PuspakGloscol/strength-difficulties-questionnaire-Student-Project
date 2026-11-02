# Copilot instructions for the SDQ monorepo

## Repository shape

This repository is a multi-service monorepo for the Strengths and Difficulties Questionnaire (SDQ) project. The big picture is:

- `sdq-analysis-ui/`: React + TypeScript + Vite frontend for spreadsheet-based analysis and bulk data queries.
- `sdq-analysis-api-java/`: original Spring Boot analysis backend; also serves the built frontend static assets.
- `sdq-analysis-api-rust/`: Rust rewrite of the analysis API; it coexists with the Java service and is treated as the newer implementation path.
- `sdq-submission-ui/`: Vite React frontend for direct submission workflows.
- `sdq-submission-api-go/`: Go backend for future/ongoing submission capture.
- `sdq-database-flyway/`: database migration project used to build the shared Postgres schema.
- `local/`: Docker Compose files that are the normal way to start dependencies and the full stack.
- `justfile/`: repo-level task runner that orchestrates the common local dev workflows.

The project intentionally has two functional verticals: analysis (bulk spreadsheet ingestion + querying) and submission (capturing new records directly), both built on the same database.

## Build, test, and run commands

Use the repo `justfile` for the standard developer flow:

```bash
# Start the full application stack in Docker
just run

# Stop the app stack
just stop

# Build the Java analysis app (includes UI static assets copied in)
just build-java

# Build the Rust analysis service
just build-rust

# Build the Go submission service
just build-go
```

Common service-level commands:

```bash
# Java analysis backend
cd sdq-analysis-api-java && ./gradlew :spring-app:bootRun

# Java analysis tests
cd sdq-analysis-api-java && ./gradlew test --info

# Single Java test case
cd sdq-analysis-api-java && ./gradlew test --tests "*SomeTest*"

# Rust analysis tests
cargo test --manifest-path sdq-analysis-api-rust/app/Cargo.toml

# Single Rust test
cargo test --manifest-path sdq-analysis-api-rust/app/Cargo.toml some_test_name

# Analysis UI dev/build/lint
npm install --prefix sdq-analysis-ui
npm run dev --prefix sdq-analysis-ui
npm run build --prefix sdq-analysis-ui
npm run lint --prefix sdq-analysis-ui

# Submission UI dev/build/lint
npm install --prefix sdq-submission-ui
npm run dev --prefix sdq-submission-ui
npm run build --prefix sdq-submission-ui
npm run lint --prefix sdq-submission-ui
```

Testing dependencies are run through Docker Compose:

```bash
# Start DB/test dependencies
just docker-run-test-deps

# Run the Java service tests after dependencies are up
just test-service-java
```

## Architecture and operational conventions

- The repository is a monorepo, not a single app. Changes often cross service boundaries, so prefer understanding the bit of the stack you are touching before editing.
- Docker Compose is the default way to run the shared database and the full stack locally. This is the repo’s source of truth for integration behavior.
- The `justfile` reads values from `.env`; if a local command fails, verify the environment variables in the repo root before changing code.
- The analysis UI is built with Vite and is copied into the Java backend’s `src/main/resources/static` during the packaging flow. Do not treat built `dist` output as the source of truth for backend behavior.
- The Java backend uses Spring Boot with Java 21 and Spotless/Google Java Format; formatting is enforced as part of the build pipeline.
- The Rust and Java analysis APIs are both active implementations of the same domain, but the repo still contains both while the migration is in progress. Prefer the existing target backend for the feature you are changing rather than mixing implementations arbitrarily.
- Database changes belong in `sdq-database-flyway`; app code should rely on migration-backed schema instead of ad hoc local schema edits.
- The app is organized around bulk spreadsheet ingestion for historical analysis, with a separate submission flow for future direct data capture.

## Key conventions specific to this codebase

- Use `just` as the first choice for local orchestration. It centralizes Docker, build, and service entrypoints.
- Treat the Docker Compose stack in `local/` and the database migration project as the stable integration environment; this is what the repo uses for full-stack runs.
- Keep the Java/Rust/Go service split clear: the analysis stack is intentionally split between Java and Rust while the submission stack is Go-based.
- For frontend work, prefer the module-local `npm` commands (`sdq-analysis-ui` and `sdq-submission-ui`) instead of a root-level build script.
- For backend work, keep the service-specific build/test commands and module boundaries intact; do not assume a single framework is used across the whole repo.
- When modifying code that affects the database schema or ingestion flow, verify the migration path and container startup flow before assuming a local change is sufficient.

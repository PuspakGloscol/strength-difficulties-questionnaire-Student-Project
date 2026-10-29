set dotenv-load := true

APPLICATION_NAME := env_var("APPLICATION_NAME")
DB_MODULE := env_var("DB_MODULE")
SUBMISSION_API_MODULE_GO := env_var("SUBMISSION_API_MODULE_GO")
ANALYSIS_API_MODULE_JAVA := env_var("ANALYSIS_API_MODULE_JAVA")
ANALYSIS_API_MODULE_RUST := env_var("ANALYSIS_API_MODULE_RUST")
ANALYSIS_UI_MODULE := env_var("ANALYSIS_UI_MODULE")
SUBMISSION_UI_MODULE := env_var("SUBMISSION_UI_MODULE")
SDQ_DATABASE_NAME := env_var("SDQ_DATABASE_NAME")
SDQ_DATABASE_USERNAME := env_var("SDQ_DATABASE_USERNAME")
SDQ_DATABASE_PASSWORD := env_var("SDQ_DATABASE_PASSWORD")

# Default task
default: docker-run-app

# Run the whole application locally in Docker
run: docker-run-app
start: docker-run-app

# Stop the application
stop: docker-stop-all

# Install Analysis UI dependencies
install-analysis-ui:
    npm install --prefix {{ANALYSIS_UI_MODULE}}

# Build the Analysis user interface
build-analysis-ui: install-analysis-ui
    npm run build --prefix {{ANALYSIS_UI_MODULE}}

# Install Submission UI dependencies
install-submission-ui:
    npm install --prefix {{SUBMISSION_UI_MODULE}}

# Build the Submission user interface
build-submission-ui: install-submission-ui
    npm run build --prefix {{SUBMISSION_UI_MODULE}}

# Copy the static resources of the UI into the public backend folder.
copy-analysis-ui: build-analysis-ui
    rm -rf {{ANALYSIS_API_MODULE_JAVA}}/src/main/resources/static
    cp -R {{ANALYSIS_UI_MODULE}}/dist {{ANALYSIS_API_MODULE_JAVA}}/spring-app/src/main/resources/static

run-service-dev-rust: 
    ANALYSIS_UI_RESOURCES_DIR="{{ANALYSIS_UI_MODULE}}/dist" \
    cargo run --manifest-path sdq-analysis-api-rust/app/Cargo.toml

run-dev-rust: build-analysis-ui run-service-dev-rust

run-service-dev-go:
    cd {{SUBMISSION_API_MODULE_GO}} && \
        DATABASE_URL="postgres://{{SDQ_DATABASE_USERNAME}}:{{SDQ_DATABASE_PASSWORD}}@localhost:5432/{{SDQ_DATABASE_NAME}}?sslmode=disable" \
        SUBMISSION_UI_RESOURCES_DIR="../{{SUBMISSION_UI_MODULE}}/dist" \
         go run ./app

run-dev-go: build-submission-ui run-service-dev-go

build-analysis-api-rust:
    cargo build --manifest-path {{ANALYSIS_API_MODULE_RUST}}/app/Cargo.toml

# Run the service via gradle
run-service-dev-java:
    cd {{ANALYSIS_API_MODULE_JAVA}} && ./gradlew :spring-app:bootRun

# Build the UI and Run the service via gradle
run-dev-java: copy-analysis-ui run-service-dev-java

# Build the JAR file
build-api-java:
    cd {{ANALYSIS_API_MODULE_JAVA}} && ./gradlew :spring-app:bootJar

# Build the UI and bundle into application JAR file
build-java: copy-analysis-ui build-api-java

build-rust:
    cargo build --manifest-path sdq-analysis-api-rust/app/Cargo.toml

build-go:
    cd {{SUBMISSION_API_MODULE_GO}} && go build -o dist/sdq-submission-api-go ./app

# Runs the user interface in hot reloading mode.
run-analysis-ui-dev: install-analysis-ui
    npm run dev --prefix {{ANALYSIS_UI_MODULE}}

# Run the backend unit tests
test-service-java:
    cd {{ANALYSIS_API_MODULE_JAVA}} && ./gradlew test --info

test-service-rust:
    cargo test --manifest-path sdq-analysis-api-rust/app/Cargo.toml

# Run the dependencies required by unit tests
# Always clean them out first
docker-run-test-deps:
    docker compose -f local/docker-compose.test.yaml down --volumes
    docker compose -f local/docker-compose.test.yaml up --build --wait

# Run the unit tests, which depends on us running containers
run-tests: docker-run-test-deps test-service-java

# Run the app images as they are
docker-run-app-no-build:
    docker compose -f local/docker-compose.yaml --profile api-rust up -d --wait

# Run the entire system up within Docker
docker-run-app:
    docker compose -f local/docker-compose.yaml --profile api-rust --profile api-java --profile api-go up --build -d --wait

docker-run-go:
    docker compose -f local/docker-compose.yaml --profile api-go up --build --wait

# Run the app dependencies in docker, but not the app itself
# Use run-service-dev-java for that
docker-run-deps:
    docker compose -f local/docker-compose.yaml up --build -d --wait

# Stop the application stack
docker-stop:
    docker compose -f local/docker-compose.yaml --profile api-java --profile api-rust --profile api-go down

# Stop the test dependencies
docker-stop-test:
    docker compose -f local/docker-compose.test.yaml --profile api-java --profile api-rust down

# Stop any docker containers relating to this application
docker-stop-all: docker-stop docker-stop-test

docker-build-rust-api:
    docker build -t {{ANALYSIS_API_MODULE_RUST}} -f Dockerfile.rust .

docker-build-go-api:
    docker build -t {{SUBMISSION_API_MODULE_GO}} -f Dockerfile.golang .

# Build the Docker image for the application
docker-build-java-api:
    docker build -t {{ANALYSIS_API_MODULE_JAVA}} -f Dockerfile.java .

# Build the Docker image for the database migration
docker-build-db-migration:
    docker build -t {{DB_MODULE}} {{DB_MODULE}}/.

build: build-analysis-ui \
    build-submission-ui \
    build-java \
    build-rust \
    build-go \
    docker-build-db-migration \
    docker-build-rust-api \
    docker-build-java-api \
    docker-build-go-api

# Run the migration on its own
database-migrate:
    docker compose -f local/docker-compose.yaml up --wait --build

# take down the docker stack, but also remove volumes
docker-clean:
    docker compose -f local/docker-compose.yaml down --volumes
    docker compose -f local/docker-compose.test.yaml down --volumes

# Connect a shell to the database.
database-connect:
    echo "Connecting to database"
    docker exec -it {{APPLICATION_NAME}}-db psql -d {{SDQ_DATABASE_NAME}} -U {{SDQ_DATABASE_USERNAME}}

test-database-connect:
    echo "Connecting to test database"
    docker exec -it {{APPLICATION_NAME}}-test-db psql -d {{SDQ_DATABASE_NAME}} -U test

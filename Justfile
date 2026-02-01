
# prints the available options
help:
  @just --list --unsorted


build:
    cargo build

test:
    cargo nextest run

# Build the docker image
docker-build:
    cargo sqlx prepare --workspace -- --all-targets
    docker build --tag zero2prod .

docker-run:
    docker run -p 8000:8000 zero2prod | jq

# Recreates the DB
setup-db:
    docker stop postgres || true
    docker rm postgres || true
    scripts/init_db.sh

healthcheck:
    curl http://127.0.0.1:8000/health_check

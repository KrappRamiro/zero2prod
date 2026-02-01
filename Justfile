
# prints the available options
help:
  @just --list --unsorted


build:
    cargo build

test:
    cargo nextest run

migrate:
    sqlx migrate run

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

# Updates the Digital Ocean deployment, based on the values of spec.yaml
# 232887b5-a9b3-4c8b-8385-16c487c18997 is the ID of our app, got it from doing doctl apps list
do-update-spec:
    doctl apps update 232887b5-a9b3-4c8b-8385-16c487c18997 --spec=spec.yaml

do-add-ip-to-database-firewall:
    doctl databases firewalls append f4f08dad-9615-4964-8f61-df9c2602c988 --rule ip_addr:$(curl -s https://api.ipify.org)

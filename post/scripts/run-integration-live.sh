#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

docker compose -f docker-compose.yml up -d postgres redis nats rustfs elasticsearch

echo "Waiting for external services..."
for _ in {1..60}; do
  if docker inspect --format '{{.State.Health.Status}}' post-postgres >/dev/null 2>&1 \
    && docker inspect --format '{{.State.Health.Status}}' post-redis >/dev/null 2>&1 \
    && docker inspect --format '{{.State.Health.Status}}' post-nats >/dev/null 2>&1 \
    && docker inspect --format '{{.State.Health.Status}}' post-rustfs >/dev/null 2>&1 \
    && docker inspect --format '{{.State.Health.Status}}' post-elasticsearch >/dev/null 2>&1 \
    && [ "$(docker inspect --format '{{.State.Health.Status}}' post-postgres)" = "healthy" ] \
    && [ "$(docker inspect --format '{{.State.Health.Status}}' post-redis)" = "healthy" ] \
    && [ "$(docker inspect --format '{{.State.Health.Status}}' post-nats)" = "healthy" ] \
    && [ "$(docker inspect --format '{{.State.Health.Status}}' post-rustfs)" = "healthy" ] \
    && [ "$(docker inspect --format '{{.State.Health.Status}}' post-elasticsearch)" = "healthy" ]; then
    break
  fi
  sleep 2
done

docker exec -i post-postgres psql -U post -d post < migrations/202606120002_integration_outbox.sql

cargo test --test integration_live -- --ignored --nocapture

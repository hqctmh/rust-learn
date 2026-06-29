#!/usr/bin/env sh
set -eu

profile="${1:-heap.pb.gz}"
addr="${2:-127.0.0.1:8080}"
binary="${3:-}"

if [ ! -f "$profile" ]; then
  echo "profile not found: $profile" >&2
  echo "generate it first: cargo run --bin jemalloc_profile_demo -- heap.pb.gz" >&2
  exit 1
fi

if [ -n "$binary" ]; then
  exec go tool pprof -http="$addr" "$binary" "$profile"
fi

exec go tool pprof -http="$addr" "$profile"

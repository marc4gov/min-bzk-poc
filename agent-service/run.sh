#!/usr/bin/env bash
# Start agent-service met env uit `agent-service/.env` (zie main.rs + dotenvy).
set -euo pipefail
cd "$(dirname "$0")"
exec cargo run

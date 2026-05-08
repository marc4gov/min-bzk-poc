#!/usr/bin/env bash
# Start de lokale privacy-filter-sidecar (openai/privacy-filter op Hugging Face).
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"
if [[ ! -d .venv ]] && command -v python3 >/dev/null; then
  echo "TIP: maak een venv: python3 -m venv .venv && . .venv/bin/activate && pip install -r requirements-privacy-filter.txt"
fi
exec uvicorn privacy_filter_server:app --host "${PII_SIDECAR_HOST:-127.0.0.1}" --port "${PII_SIDECAR_PORT:-8091}"

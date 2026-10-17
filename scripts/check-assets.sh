#!/usr/bin/env bash
# Blindaje S4 (docs/anim-liviano.md): ningún MP4/MOV/WebM mayor a 5MiB en
# assets/, docs/ ni crates/. Solo GIF ≤5MiB o receta `.json`.
# Uso: `bash scripts/check-assets.sh` antes de pushear.
set -euo pipefail

tope_bytes=$((5 * 1024 * 1024))
mal=0

while IFS= read -r pesado; do
  [ -z "$pesado" ] && continue
  tamano=$(stat -c%s "$pesado" 2>/dev/null || stat -f%z "$pesado")
  if [ "$tamano" -gt "$tope_bytes" ]; then
    echo "check-assets: PESADO $pesado ($tamano bytes > 5MiB) — usá GIF ≤5MiB o receta .json (docs/anim-liviano.md)" >&2
    mal=1
  fi
done < <(find assets docs crates -type f \( -iname '*.mp4' -o -iname '*.mov' -o -iname '*.webm' \) 2>/dev/null)

if [ "$mal" -ne 0 ]; then
  exit 1
fi
echo "check-assets: limpio (sin mp4/mov/webm >5MiB en assets/docs/crates)"

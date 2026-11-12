#!/usr/bin/env bash
set -euo pipefail
# Lancer depuis desktop, quel que soit le dossier de version.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
# Racine commune LXO via variable, sinon détecter V8/V7.1/.../recycling-system-v*/desktop.
if [[ -n "${LXO_HOME:-}" ]]; then
  LXO_ROOT="$LXO_HOME"
else
  DIR="$SCRIPT_DIR"
  LXO_ROOT=""
  while [[ "$DIR" != / ]]; do
    if [[ "$(basename "$DIR")" == LXO ]]; then LXO_ROOT="$DIR";break;fi
    DIR="$(dirname "$DIR")"
  done
  if [[ -z "$LXO_ROOT" ]]; then LXO_ROOT="$HOME/.cache/recycling-system-industrial";fi
fi
mkdir -p "$LXO_ROOT/cache-build/cargo-target" "$LXO_ROOT/cache-build/npm-cache"
export CARGO_TARGET_DIR="$LXO_ROOT/cache-build/cargo-target"
export npm_config_cache="$LXO_ROOT/cache-build/npm-cache"
printf 'Sources : %s\nCargo partagé : %s\nnpm partagé : %s\n' "$SCRIPT_DIR" "$CARGO_TARGET_DIR" "$npm_config_cache"
if [[ ! -d node_modules ]]; then
  if [[ -f package-lock.json ]]; then npm ci --prefer-offline;else npm install --prefer-offline;fi
fi
# Synchroniser la clé publique uniquement si nécessaire, sans toucher à la clé privée.
if [[ -f "$HOME/.local/share/recycling-license-manager/public-key.hex" ]]; then
  KEY_SRC="$HOME/.local/share/recycling-license-manager/public-key.hex"
  KEY_DST="src-tauri/src/license_public_key.hex"
  python3 - "$KEY_SRC" "$KEY_DST" <<'PYVALID'
from pathlib import Path
import sys
src, dst = (Path(x) for x in sys.argv[1:])
k = src.read_text().strip()
if len(k) != 64 or k == '0'*64:
    raise SystemExit('Chave pública Ed25519 inválida')
try:
    bytes.fromhex(k)
except ValueError as e:
    raise SystemExit(f'Chave pública inválida: {e}')
dst.write_text(k+'\n')
print('Chave pública Ed25519 sincronizada')
PYVALID
else
  echo 'Aviso: chave pública não encontrada; verifique src-tauri/src/license_public_key.hex.'
fi
npm run tauri:dev

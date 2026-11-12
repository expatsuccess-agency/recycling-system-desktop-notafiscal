#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
echo 'Building V8-NotaFiscal for Linux (AppImage only)'
npm run build:web
npm run tauri:build -- --bundles appimage

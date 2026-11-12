#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
if [[ "$(id -u)" == 0 ]]; then echo 'ERREUR : compiler comme utilisateur normal, pas root.' >&2;exit 1;fi
for cmd in node npm cargo rustc;do command -v "$cmd" >/dev/null || { echo "Outil absent: $cmd" >&2;exit 1;};done
if ! command -v pkg-config >/dev/null;then echo 'Installer pkg-config et les dépendances Tauri (voir README).' >&2;exit 1;fi
pkg-config --exists webkit2gtk-4.1 || { echo 'libwebkit2gtk-4.1-dev manquant.' >&2;exit 1;}
if [[ -f package-lock.json ]];then npm ci;else npm install;fi
npm test
npm run build:web
npm run tauri:build -- --bundles appimage
mkdir -p linux-build
find src-tauri/target/release/bundle/appimage -maxdepth 1 -name '*.AppImage' -exec cp -v '{}' linux-build/ \;
find linux-build -type f -name '*.AppImage' -exec sha256sum '{}' \; | tee linux-build/SHA256SUMS
if ! find linux-build -name '*.AppImage' | grep -q .;then echo 'ERREUR: aucun AppImage généré' >&2;exit 1;fi
echo 'Terminé : dossier linux-build/'

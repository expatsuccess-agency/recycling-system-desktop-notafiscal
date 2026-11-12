#!/usr/bin/env bash
set -Eeuo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export LXO_DIR="$HERE"
exec bash "$HERE/build-pesa-recicla-v27-windows.sh"

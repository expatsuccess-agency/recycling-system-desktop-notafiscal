#!/usr/bin/env bash
set -Eeuo pipefail

PROJECT="/media/user/skllnxs.cloud/Projets/LXO/V8-NotaFiscal/recycling-system-v8-notafiscal/desktop"
CACHE="/media/user/skllnxs.cloud/Projets/LXO/cache-build/cargo-target"

cd "$PROJECT"

if [[ "$(id -u)" -eq 0 ]]; then
    echo "ERREUR : ne pas exécuter en root."
    exit 1
fi

echo "=========================================="
echo " RECYCLING NOTAFISCAL — CORRECTION VENDAS"
echo "=========================================="

python3 - <<'PY'
from pathlib import Path
from datetime import datetime
import re
import shutil

ui = Path("src/renderer/index.tsx").read_text()

# Vérifier que le libellé supprimé n'est plus
# présent dans le composant principal.
if "Empresa compradora" in ui:
    raise SystemExit(
        "ARRÊT : Empresa compradora existe encore "
        "dans index.tsx. Correction de l'interface requise."
    )

files = list(Path("tests").glob("*.test.mjs"))
modified = 0

pattern = re.compile(
    r'assert\.match\(\s*(\w+)\s*,'
    r'\s*/Empresa compradora/\s*\)'
)

for path in files:
    source = path.read_text()
    updated, count = pattern.subn(
        r'assert.doesNotMatch(\1,/Empresa compradora/)',
        source
    )

    if count:
        backup = path.with_name(
            path.name + ".backup-" +
            datetime.now().strftime("%Y%m%d-%H%M%S")
        )
        shutil.copy2(path, backup)
        path.write_text(updated)
        modified += count
        print("Corrigé :", path, "-", count, "assertion(s)")
        print("Sauvegarde :", backup)

if modified == 0:
    print("Aucune assertion obsolète trouvée.")
else:
    print("Total corrigé :", modified)
PY

echo ""
echo "=== 1. TEST V71 ==="

LOG="$(mktemp)"
trap 'rm -f "$LOG"' EXIT

if node --test tests/v71-improvements.test.mjs \
    >"$LOG" 2>&1; then
    echo "TEST V71 : OK"
else
    echo "TEST V71 : ECHEC"
    grep -E \
      'not ok|✖|AssertionError|expected:|operator:|test at' \
      "$LOG" | head -25 || true
    exit 1
fi

echo ""
echo "=== 2. TOUS LES TESTS ==="

if npm test >"$LOG" 2>&1; then
    echo "NPM TEST : OK"
    tail -12 "$LOG"
else
    echo "NPM TEST : ECHEC"
    grep -E \
      'not ok|✖|AssertionError|expected:|operator:|test at|fail ' \
      "$LOG" | head -40 || true
    exit 1
fi

echo ""
echo "=== 3. BUILD FRONTEND ==="
npm run build

echo ""
echo "=== 4. COMPILATION RUST ==="
export CARGO_TARGET_DIR="$CACHE"
cargo check --manifest-path src-tauri/Cargo.toml

echo ""
echo "=== 5. GIT ==="
git --no-pager diff --stat
git status --short

echo ""
echo "=========================================="
echo " VENDAS — VALIDATION TERMINÉE"
echo "=========================================="

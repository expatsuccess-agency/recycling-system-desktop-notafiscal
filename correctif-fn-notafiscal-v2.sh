#!/usr/bin/env bash
set -Eeuo pipefail

# Recycling System V8 NotaFiscal : réparation des touches Fn
# Usage : bash correctif-fn-notafiscal-v2.sh [archive.zip] [--no-build]

V8='/media/user/skllnxs.cloud/Projets/LXO/V8/recycling-system-v8/desktop'
NF='/media/user/skllnxs.cloud/Projets/LXO/V8-NotaFiscal/recycling-system-v8-notafiscal/desktop'
ZIP_NAME='RecyclingSystem_V8_NotaFiscal_8.2_Correction_Fn_Sources.zip'
INNER='recycling-system-v8-notafiscal/desktop'
BUILD=1
ARCHIVE=''
TMP=''

fail() { printf '\nERREUR : %s\n' "$*" >&2; exit 1; }
cleanup() { [[ -z "$TMP" ]] || rm -rf -- "$TMP"; }
trap cleanup EXIT
trap 'printf "\nÉchec à la ligne %s. Les sources originales ont été sauvegardées avant copie.\n" "$LINENO" >&2' ERR

for arg in "$@"; do
  case "$arg" in
    --no-build) BUILD=0 ;;
    --build) BUILD=1 ;;
    -h|--help) echo "Usage: bash $0 [chemin/vers/$ZIP_NAME] [--no-build]"; exit 0 ;;
    *.zip) [[ -z "$ARCHIVE" ]] || fail 'Une seule archive ZIP autorisée'; ARCHIVE="$arg" ;;
    *) fail "Argument inconnu : $arg" ;;
  esac
done

for cmd in unzip node grep mktemp cp; do
  command -v "$cmd" >/dev/null 2>&1 || fail "Commande manquante : $cmd"
done
[[ -d "$NF" && -f "$NF/package.json" && -f "$NF/src/renderer/index.tsx" ]] || fail "Sources NotaFiscal absentes : $NF"

# Recherche non récursive dans les emplacements habituels de téléchargement.
if [[ -z "$ARCHIVE" ]]; then
  for d in "$PWD" "$HOME/Downloads" "$HOME/Téléchargements" "$HOME/Transferências" "$HOME/Desktop" "$HOME/Bureau" "$(dirname "$NF")"; do
    if [[ -f "$d/$ZIP_NAME" ]]; then ARCHIVE="$d/$ZIP_NAME"; break; fi
  done
fi

# Recherche élargie si la localisation exacte du navigateur est inconnue.
if [[ -z "$ARCHIVE" ]]; then
  for d in "$HOME/Downloads" "$HOME/Téléchargements" "$HOME/Transferências" "$HOME/Desktop" "$HOME/Bureau"; do
    [[ -d "$d" ]] || continue
    candidate="$(find "$d" -maxdepth 3 -type f -name "$ZIP_NAME" -print -quit 2>/dev/null || true)"
    if [[ -n "$candidate" ]]; then ARCHIVE="$candidate"; break; fi
  done
fi

if [[ -z "$ARCHIVE" || ! -f "$ARCHIVE" ]]; then
  printf '\nArchive introuvable automatiquement : %s\n' "$ZIP_NAME" >&2
  printf 'Relance avec son chemin exact, par exemple :\n' >&2
  printf 'bash "%s" "$HOME/Downloads/%s"\n' "$0" "$ZIP_NAME" >&2
  exit 1
fi

ARCHIVE="$(realpath "$ARCHIVE")"
printf '\n=== Correctif Fn V8-NotaFiscal ===\nArchive : %s\nProjet : %s\n' "$ARCHIVE" "$NF"
unzip -tq "$ARCHIVE" >/dev/null || fail 'Archive ZIP corrompue'

# Le précédent script échouait sur `unzip -Z1 | grep -q` en mode pipefail.
# Lire directement les deux fichiers internes évite l'erreur SIGPIPE/grep.
TMP="$(mktemp -d)"
UI_FILE="$INNER/src/renderer/index.tsx"
TEST_FILE="$INNER/tests/v82-shortcuts-notafiscal.test.mjs"
unzip -p "$ARCHIVE" "$UI_FILE" > "$TMP/index.tsx" || fail "Fichier introuvable dans le ZIP : $UI_FILE"
unzip -p "$ARCHIVE" "$TEST_FILE" > "$TMP/test.mjs" || fail "Fichier introuvable dans le ZIP : $TEST_FILE"
[[ -s "$TMP/index.tsx" && -s "$TMP/test.mjs" ]] || fail 'Fichiers du correctif vides'

node --check "$TMP/test.mjs" || fail 'Syntaxe du test invalide'
for pattern in "F1:'weigh',F3:'vendas',F7:'finance',F9:'config'" "ev.key==='F5'||ev.key==='F11'" 'Caixas Anteriores <kbd>F11</kbd>'; do
  grep -Fq "$pattern" "$TMP/index.tsx" || fail "Mapping attendu absent : $pattern"
done

STAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP="$NF/.sauvegardes-correctif-fn/$STAMP"
mkdir -p "$BACKUP/src/renderer" "$BACKUP/tests" "$NF/tests"
cp -a "$NF/src/renderer/index.tsx" "$BACKUP/src/renderer/index.tsx"
if [[ -f "$NF/tests/v82-shortcuts-notafiscal.test.mjs" ]]; then
  cp -a "$NF/tests/v82-shortcuts-notafiscal.test.mjs" "$BACKUP/tests/"
fi
printf '\n[1/4] Sauvegarde : %s\n' "$BACKUP"

# Seulement deux fichiers du projet (pas de copie générale, pas de .git, pas de base).
cp "$TMP/index.tsx" "$NF/src/renderer/index.tsx"
cp "$TMP/test.mjs" "$NF/tests/v82-shortcuts-notafiscal.test.mjs"
printf '[2/4] Correctif installé dans les sources NotaFiscal\n'

cd "$NF"
node --test tests/v82-shortcuts-notafiscal.test.mjs
printf '[3/4] Test Fn réussi\n'

if (( BUILD )); then
  command -v npm >/dev/null 2>&1 || fail 'npm absent : correctif installé, compilation non effectuée'
  printf '\n[4/4] Vérification et compilation Linux/Tauri\n'
  if [[ ! -x node_modules/.bin/vite || ! -x node_modules/.bin/tauri ]]; then
    npm ci
  fi
  npm run build:web
  npm run tauri:build
else
  printf '[4/4] Build ignoré (--no-build)\n'
fi

printf '\nTERMINE : F1 Pesagem | F3 Controle | F5 Caixa Atual | F7 Financeira | F9 Configuração | F11 Caixas Anteriores\n'
printf 'Sauvegarde : %s\nV8 standard inchangée : %s\n' "$BACKUP" "$V8"
printf 'Bundles : %s/src-tauri/target/release/bundle/\n' "$NF"


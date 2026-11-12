#!/usr/bin/env bash
set -Eeuo pipefail

# Pesa Recicla V2.7 - Build de l'installateur Windows NSIS depuis Parrot Linux.
# Usage : bash build-pesa-recicla-v27-windows.sh
# Options : LXO_DIR=/autre/chemin bash build-pesa-recicla-v27-windows.sh
#           LXO_PFX=/chemin/certificat.pfx bash build-pesa-recicla-v27-windows.sh

LXO_DIR="${LXO_DIR:-/media/user/skllnxs.cloud/Projets/LXO/V2.7/lxo-starter/desktop}"
TARGET="x86_64-pc-windows-msvc"
OUTPUT_DIR="$LXO_DIR/windows-build"
LOG="$HOME/pesa-recicla-v27-build-$(date +%Y%m%d-%H%M%S).log"

say() { printf '\n\033[1;36m[LXO] %s\033[0m\n' "$*"; }
fail() { printf '\n\033[1;31m[ERREUR] %s\033[0m\n' "$*" >&2; exit 1; }
on_error() { printf '\n[ÉCHEC] Ligne %s. Journal : %s\n' "$1" "$LOG" >&2; }
trap 'on_error "$LINENO"' ERR

[[ $EUID -ne 0 ]] || fail "Ne lance pas ce script en root. Utilise le compte user (sudo sera demandé pour apt)."
[[ -d "$LXO_DIR" ]] || fail "Dossier introuvable : $LXO_DIR"
[[ -f "$LXO_DIR/package.json" ]] || fail "package.json absent : $LXO_DIR"
[[ -f "$LXO_DIR/src-tauri/tauri.conf.json" ]] || fail "src-tauri/tauri.conf.json absent. Initialise et migre réellement LXO vers Tauri avant le build."
[[ -f "$LXO_DIR/src-tauri/Cargo.toml" ]] || fail "Cargo.toml absent : backend Tauri non initialisé."
[[ -f "$LXO_DIR/src-tauri/src/lib.rs" || -f "$LXO_DIR/src-tauri/src/main.rs" ]] || fail "Code Rust Tauri manquant."

# IMPORTANT : ne pas compiler une coquille Tauri alors que React appelle encore l'API Electron.
if grep -RqlE 'window[.]lxo|window[.]electron|ipcRenderer' "$LXO_DIR/src/renderer" "$LXO_DIR/src" \
    --include='*.tsx' --include='*.ts' --include='*.jsx' --include='*.js' 2>/dev/null; then
  fail "Des appels Electron (window.lxo / IPC) subsistent dans src/. Il faut migrer les opérations vers les commandes Tauri avant de produire un EXE fonctionnel."
fi
if grep -Eq '"(electron|electron-builder|better-sqlite3|@electron/rebuild)"[[:space:]]*:' "$LXO_DIR/package.json"; then
  fail "Des dépendances Electron persistent dans package.json (source possible d'EALLOWGIT). Migre/nettoie le manifest avant le packaging."
fi

mkdir -p "$OUTPUT_DIR"
exec > >(tee -a "$LOG") 2>&1
say "Projet : $LXO_DIR"
say "Journal : $LOG"
cd "$LXO_DIR"

say '1/7 - Installer les outils Linux nécessaires (nécessite sudo)'
sudo apt-get update
sudo apt-get install -y \
  build-essential curl ca-certificates file pkg-config \
  clang lld llvm nsis git python3 cmake \
  libssl-dev libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libxdo-dev

say '2/7 - Vérifier Node.js et npm'
command -v node >/dev/null || fail 'Node.js absent : installer Node.js 22/24 LTS.'
command -v npm >/dev/null || fail 'npm absent.'
node --version
npm --version

say '3/7 - Installer Rust et la cible Windows'
if ! command -v rustup >/dev/null; then
  say "Installation de rustup (script officiel téléchargé en HTTPS)"
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs -o /tmp/lxo-rustup-init.sh
  sh /tmp/lxo-rustup-init.sh -y --profile minimal
fi
# shellcheck disable=SC1090
[[ ! -f "$HOME/.cargo/env" ]] || source "$HOME/.cargo/env"
command -v cargo >/dev/null || fail 'Cargo introuvable après installation de Rust.'
rustup target add "$TARGET"
if ! command -v cargo-xwin >/dev/null; then
  cargo install --locked cargo-xwin
fi
command -v makensis >/dev/null || fail 'NSIS (makensis) introuvable.'
command -v llvm-rc >/dev/null || fail 'llvm-rc introuvable : vérifier l’installation du paquet llvm.'

say '4/7 - Installer les dépendances JavaScript'
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
[[ -x node_modules/.bin/tauri ]] || fail 'CLI Tauri absente. Installer @tauri-apps/cli@^2 dans devDependencies.'

say '5/7 - Contrôler les scripts frontend et la configuration Tauri'
node - <<'NODE'
const fs=require('fs');
const pkg=JSON.parse(fs.readFileSync('package.json','utf8'));
const c=JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8'));
if(!pkg.scripts?.['build:web']) throw Error('Script npm build:web absent : définir une compilation Vite (sans electron-builder).');
if(!c.build?.frontendDist) throw Error('frontendDist absent dans tauri.conf.json.');
if(c.bundle?.targets && !(['nsis','all'].includes(c.bundle.targets) || (Array.isArray(c.bundle.targets)&&c.bundle.targets.includes('nsis')))) {
  throw Error('Configurer bundle.targets sur ["nsis"] dans tauri.conf.json.');
}
console.log('Frontend:', c.build.frontendDist, 'Version:',c.version);
NODE
npm run build:web

say '6/7 - Construire l’application Windows et son installateur NSIS'
export XWIN_CACHE_DIR="${XWIN_CACHE_DIR:-$HOME/.cache/pesa-recicla-xwin}"
./node_modules/.bin/tauri build --runner cargo-xwin --target "$TARGET" --bundles nsis

say '7/7 - Rassembler les installateurs et vérifier les fichiers'
NSIS_DIR="$LXO_DIR/src-tauri/target/$TARGET/release/bundle/nsis"
[[ -d "$NSIS_DIR" ]] || fail "Aucun répertoire NSIS généré : $NSIS_DIR"
mapfile -d '' installers < <(find "$NSIS_DIR" -maxdepth 1 -type f -iname '*.exe' -print0)
(( ${#installers[@]} > 0 )) || fail "Aucun fichier .exe trouvé dans $NSIS_DIR"
for installer in "${installers[@]}"; do
  cp -f -- "$installer" "$OUTPUT_DIR/"
  file "$installer"
  sha256sum "$installer" | tee -a "$OUTPUT_DIR/SHA256SUMS.txt"
done

# Signature facultative avec certificat EXISTANT (pas de clé privée dans les sources).
if [[ -n "${LXO_PFX:-}" ]]; then
  command -v osslsigncode >/dev/null || fail 'LXO_PFX défini mais osslsigncode absent : sudo apt install osslsigncode'
  [[ -f "$LXO_PFX" ]] || fail "Certificat PFX introuvable : $LXO_PFX"
  say 'Signature avec PFX existant (mot de passe demandé par osslsigncode)'
  for installer in "${installers[@]}"; do
    base=$(basename "$installer")
    signed="$OUTPUT_DIR/${base%.exe}-signed.exe"
    osslsigncode sign -pkcs12 "$LXO_PFX" -askpass -h sha256 \
      -in "$installer" -out "$signed"
    osslsigncode verify -in "$signed" || say 'Attention : vérification de chaîne de confiance non concluante (certificat auto-signé).'
  done
fi

say "Build terminé. Installateurs disponibles dans : $OUTPUT_DIR"
say 'Note : vérifier le fonctionnement sur Windows 10/11. Cross-build et signature auto-signée ne garantissent pas une installation sans avertissement SmartScreen.'

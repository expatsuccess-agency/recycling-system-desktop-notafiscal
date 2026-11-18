#!/usr/bin/env bash
set -Eeuo pipefail

LXO="/media/user/skllnxs.cloud/Projets/LXO"
MANAGER="$LXO/license_manager.py"
VENV="$LXO/.venv-license"
PYTHON="$VENV/bin/python"
KEY_DIR="${RECYCLING_ISSUER_HOME:-$HOME/.local/share/recycling-license-manager}"
OUTPUT_DIR="$LXO/licences-emises"

die() { printf '\nERREUR : %s\n' "$*" >&2; exit 1; }

printf '\n===========================================\n'
printf ' RECYCLING SYSTEM INDUSTRIAL\n'
printf ' ÉMISSION DE LICENCE Ed25519\n'
printf '===========================================\n\n'

[[ -f "$MANAGER" ]] || die "License Manager introuvable : $MANAGER"
[[ -f "$KEY_DIR/private-key.pem" ]] || die "Clé privée introuvable : $KEY_DIR/private-key.pem (ne pas régénérer les clés existantes)"
[[ -f "$KEY_DIR/public-key.hex" ]] || die "Clé publique introuvable : $KEY_DIR/public-key.hex"

if [[ ! -x "$PYTHON" ]]; then
  echo 'Création de l’environnement Python...'
  python3 -m venv "$VENV" || die "Impossible de créer le venv. Installer python3-venv si nécessaire."
fi

if ! "$PYTHON" -c 'import cryptography' >/dev/null 2>&1; then
  echo 'Installation de cryptography...'
  "$PYTHON" -m pip install cryptography || die "Installation de cryptography impossible"
fi

printf '\n'
read -r -p 'Code machine (rs4- suivi de 64 caractères hexadécimaux) : ' MACHINE
read -r -p 'Nom du client / entreprise : ' CUSTOMER
read -r -p 'Durée en jours [365] : ' DAYS
DAYS="${DAYS:-365}"

[[ "$MACHINE" =~ ^rs4-[[:xdigit:]]{64}$ ]] || die 'Code machine invalide (format attendu : rs4- + 64 caractères hexadécimaux)'
[[ -n "${CUSTOMER//[[:space:]]/}" ]] || die 'Nom du client obligatoire'
[[ "$DAYS" =~ ^[0-9]+$ ]] && (( 10#$DAYS >= 1 )) || die 'La durée doit être un nombre entier positif'

mkdir -p "$OUTPUT_DIR"
OUTPUT="$OUTPUT_DIR/licence-$(date +%Y%m%d-%H%M%S)-$$.lic"

printf '\nClient : %s\nDurée : %s jours\nDestination : %s\n' "$CUSTOMER" "$DAYS" "$OUTPUT"
read -r -p 'Confirmer l’émission ? (o/N) : ' CONFIRM
[[ "$CONFIRM" == 'o' || "$CONFIRM" == 'O' ]] || { echo 'Émission annulée.'; exit 0; }

"$PYTHON" "$MANAGER" issue \
  --machine "$MACHINE" \
  --customer "$CUSTOMER" \
  --days "$DAYS" \
  --output "$OUTPUT"

[[ -s "$OUTPUT" ]] || die 'Le fichier licence n’a pas été créé'

echo 'Vérification de la signature...'
"$PYTHON" "$MANAGER" verify "$OUTPUT" || die 'Échec de la vérification de licence'

printf '\nLICENCE CRÉÉE ET VÉRIFIÉE :\n%s\n' "$OUTPUT"
printf 'Transmettre uniquement le fichier .lic, jamais private-key.pem.\n'

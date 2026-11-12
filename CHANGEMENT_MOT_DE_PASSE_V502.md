# Recycling System Industrial v5.0.2 — Règle des mots de passe et œilleton

Version inchangée : **5.0.2**.

- Nouveaux mots de passe des administrateurs et employés : **8 chiffres exactement** (0–9). Validation côté Rust et côté interface.
- Un bouton avec œilleton permet d’afficher ou masquer **indépendamment** les mots de passe dans la connexion, le changement obligatoire, les réglages du compte, la création et la réinitialisation des utilisateurs.
- Le mot de passe existant est toujours accepté pour se connecter, même s’il ne suit pas la nouvelle règle. Lors d’un changement ou d’une réinitialisation, le **nouveau** mot de passe doit respecter les 8 chiffres. L’identifiant initial `admin` / `admin` reste soumis au changement obligatoire.
- Un code de huit chiffres offre une protection **plus faible** qu’un mot de passe long. Éviter de réutiliser les mêmes codes sur plusieurs comptes et protéger physiquement le poste.

## Installation

Sauvegarder la base SQLite et `license_public_key.hex`, puis remplacer les sources avec ce paquet.

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V5.0.2/recycling-system-v5/desktop
npm install
npm test
npm run tauri:dev
```

Ne pas régénérer la paire de clés Ed25519 ni écraser les données SQLite.

## Vérification

Se connecter avec les identifiants existants, modifier le mot de passe avec exactement huit chiffres, cliquer sur l’œilleton de chacun des cinq champs et tester la création / réinitialisation de l’utilisateur.

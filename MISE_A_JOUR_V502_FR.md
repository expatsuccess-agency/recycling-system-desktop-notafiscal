# Recycling System Industrial V5.0.2 — Interface et raccourcis

## Nouveautés

- Gestão Financeira (F8) est visible dans le menu latéral gauche, en dessous de Configuração. L’accès est réservé au profil admin.
- Le bouton Gestão Financeira a été retiré du bandeau « Sistema de compra e venda de recicláveis ».
- La page financière occupe désormais toute la largeur de la zone de travail, sous le bandeau (correction de la structure JSX).
- Le menu Clientes et le raccourci F4 ont été retirés de la navigation. Le module de données clients n’est pas supprimé de SQLite.
- Dans Pesagem, Entrée dans Peso ajoute la ligne valide, vide Produto et Peso, puis donne le focus à Produto. La modification du preço/kg reste possible si elle est autorisée pour l’article en configuration.
- Dans Forma de pagamento, Dinheiro reste le choix par défaut et Entrée confirme l’opération sélectionnée. La fenêtre se ferme après réussite ; en cas d’échec elle reste ouverte et affiche une erreur. Une seule confirmation est permise pendant l’attente du backend.
- Le profil Funcionário reste limité à Pesagem. La vérification des droits backend Rust demeure inchangée.

## Installation sur Parrot Linux

Décompresser dans un nouveau dossier pour conserver les sources précédentes :

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V5.0.2/recycling-system-v5/desktop
npm install
npm test
npm run tauri:dev
```

Pour créer un AppImage, après validation :

```bash
bash build-appimage-linux.sh
```

### Licence et données

Avant démarrage, recopier la clé publique `src-tauri/src/license_public_key.hex` de l’installation V5 précédente. Ne pas générer une autre paire de clés Ed25519 ; la clé doit correspondre aux licences déjà émises. Sauvegarder SQLite et fermer l’ancienne application avant toute migration. La structure de SQLite n’est pas modifiée par ce correctif.

### État de validation

32 tests JS réussis ; syntaxe TSX contrôlée. La compilation frontend complète n’a pas été réalisée dans l’environnement de préparation faute de dépendances npm disponibles hors ligne. La compilation et l’essai sous Parrot restent nécessaires avant distribution.

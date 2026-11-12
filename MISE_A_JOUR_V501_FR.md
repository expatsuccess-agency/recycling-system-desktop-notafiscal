# Recycling System Industrial V5.0.1 — mise à jour interface

## Modifications

- **Gestão Financeira (F8)** est maintenant directement sous « Sistema de compra e venda de recicláveis » dans l'en-tête principal, visible uniquement par l'administrateur. Il n'est plus dans le menu latéral.
- **Pesagem / Produto :** avec les premières lettres saisies, Entrée sélectionne le produit surligné et place le curseur dans « Peso (kg) ». Les flèches haut/bas sélectionnent une suggestion.
- **Paiement :** « Dinheiro » est présélectionné et reçoit le focus à l'ouverture. Entrée sélectionne la méthode actuellement focalisée. **Entrée ne déclenche pas l'enregistrement de la vente** ; seul le bouton distinct « Confirmar compra/venda » enregistre l'opération.
- **Configuração / Produtos :** grille de champs plus compacte et dimensions uniformisées pour Código, Descrição, Preço/kg et autres entrées, avec adaptation à la largeur de la fenêtre.
- Correction du problème de compilation Rust `OsRng` : ajout de `rand_core` avec `getrandom`.

## Installation sur Parrot Linux

1. Fermer V5.0, sauvegarder les données SQLite (et conserver votre fichier de licence et votre clé publique de confiance).
2. Extraire cette archive dans `/media/user/skllnxs.cloud/Projets/LXO/V5.0.1/`.
3. Vérifier que la même clé publique Ed25519 que V5.0 est présente dans `src-tauri/src/license_public_key.hex` ; ne pas générer une autre paire de clés.
4. Lancer avec l'utilisateur normal, sans sudo :

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V5.0.1/recycling-system-v5/desktop
npm install
npm test
npm run tauri:dev
```

Pour produire l'AppImage une fois la version testée :

```bash
bash build-appimage-linux.sh
```

**État des tests :** 32 tests Node.js réussis (dont 4 nouveaux). La compilation complète Tauri/Rust et le parcours réel des raccourcis clavier ne sont pas validés ici.

# Recycling System V4.0 Linux (Tauri)

Ce composant contient uniquement le vérificateur Ed25519 et la clé publique de l'éditeur. Il n'émet jamais de licences.

## Initialisation

Le fichier `src-tauri/src/license_public_key.hex` contient un placeholder. **La compilation échoue tant que la vraie clé publique n'a pas été installée**. Cette clé provient de Recycling License Manager.

Exemple après initialisation de l'outil éditeur :

```bash
cp "$HOME/.local/share/recycling-license-manager/public-key.hex" src-tauri/src/license_public_key.hex
npm install
npm run tauri:dev
```

Au premier lancement, copier le code machine depuis l'écran d'activation. Importer ensuite le `.lic` signé par l'éditeur.

L'identification actuelle s'appuie sur `/etc/machine-id` avec un hachage contextualisé SHA-256, et non sur une protection TPM. Les machines Linux clonées peuvent partager le même code. La licence est vérifiée au démarrage de l'interface et avant chaque commande métier dans Rust. Base locale conservée même si licence absente ou expirée.

Pour créer un AppImage : `bash build-appimage-linux.sh`. Tester d'abord `cargo check` dans `src-tauri` sur Linux et `npm run tauri:dev`.

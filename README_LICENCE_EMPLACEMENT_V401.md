# Recycling System Industrial 4.0.1 — emplacement du fichier .lic

## Deux modes de licence

Sur l'écran **Ativação da licença**, choisir :

1. **Importar e guardar no aplicativo (recomendado)** — sélectionner `recycling.lic`. Après vérification Ed25519, identité machine et expiration, l'application conserve une copie dans son répertoire de données Tauri, sous `license/recycling.lic`. Le fichier source peut ensuite être déplacé.
2. **Usar um arquivo no local escolhido** — cliquer sur **Procurar arquivo .lic no computador…**, sélectionner le fichier existant (disque local, répertoire partagé ou clé USB). L'application conserve seulement le chemin absolu dans `license-location.json` et revérifie le `.lic` à chaque ouverture et avant les opérations métier. Le fichier doit rester accessible. S'il est supprimé/déplacé ou si la clé USB est débranchée, l'activation échouera jusqu'à sa remise à l'emplacement d'origine.

Les chemins sont déterminés par Tauri (`app_data_dir`) et dépendent de l'utilisateur et de l'identifiant applicatif. Aucun chemin absolu n'est intégré à l'exécutable.

## Mise à jour

Sur Linux, extraire l'archive dans un dossier de sources distinct, puis :

```bash
cd recycling-system-v4/desktop
npm install
npm test
npm run tauri:dev
```

Pour produire un AppImage après les tests :

```bash
bash build-appimage-linux.sh
```

Le module `@tauri-apps/plugin-dialog` a été ajouté pour le sélecteur natif, avec son plugin Rust et la permission `dialog:allow-open`.

## Sécurité

- Aucune clé privée n'est intégrée à l'application cliente.
- La signature Ed25519, l'identifiant machine et la date d'expiration sont contrôlés par Rust avant d'accepter le fichier.
- Le chemin du fichier externe est vérifié avant d'être mémorisé. Les données SQLite ne sont pas effacées lorsque la licence devient inaccessible.
- Le mode copie interne est préférable pour ne pas dépendre d'un support USB ou d'un montage réseau.
- La vérification des tests npm ne remplace pas un `cargo check` ou un essai de l'interface sur Linux.

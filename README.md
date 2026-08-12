# KeyHold

KeyHold est un petit utilitaire desktop qui maintient une ou plusieurs touches du clavier enfoncées à votre place. Sa fenêtre reste discrète au-dessus des autres applications et montre immédiatement si les touches sont actives.

> État actuel : première version ciblée et compilée pour **Windows 10/11**. La couche clavier est isolée pour permettre une adaptation ultérieure à macOS et Linux.

## Fonctionnalités

- sélection d'une touche, d'une combinaison ou de plusieurs touches directement au clavier ;
- maintien simultané réel des touches au niveau du système ;
- arrêt manuel ou automatique après une durée définie ;
- indicateur visuel vert lorsque la touche est maintenue ;
- passage automatique en mini-widget dans le coin supérieur droit pendant le maintien ;
- mini-widget limité à la touche active et son animation pulsée, cliquable pour rouvrir les contrôles ;
- mini-widget adaptatif qui réduit les touches et s'élargit progressivement pour rester lisible ;
- fenêtre principale compacte de 336 × 428 px, fixe et toujours au premier plan ;
- réglages mémorisés localement ;
- menu dans la barre système pour afficher, relâcher ou quitter ;
- raccourci global d'urgence `Ctrl + Shift + F12` ;
- relâchement garanti par Rust même lorsque la fenêtre est masquée ;
- recherche et installation automatiques des nouvelles versions au lancement ;
- vérification cryptographique de chaque mise à jour avant son installation.

## Utilisation

1. Ouvrez KeyHold.
2. Cliquez sur les touches affichées au centre, maintenez la touche ou la combinaison souhaitée, puis relâchez toutes les touches.
3. Activez **Arrêt automatique** si nécessaire et choisissez la durée.
4. Cliquez sur **Maintenir la touche** ou **Maintenir les touches**.
5. Revenez dans l'application cible : la sélection reste enfoncée jusqu'à l'arrêt manuel ou la fin du timer.

Pour interrompre immédiatement le maintien depuis n'importe quelle application, utilisez `Ctrl + Shift + F12`.

## Installation Windows

Les builds validés par GitHub Actions produisent un installateur NSIS. Téléchargez le `.exe` ou son archive `.zip` depuis la section **Releases**. L'archive ZIP peut être plus simple à récupérer si le navigateur bloque directement l'exécutable.

Windows peut afficher un avertissement SmartScreen tant que l'application n'est pas signée. Certaines applications exécutées en administrateur nécessitent que KeyHold soit également lancé en administrateur. Les logiciels protégés par un système anti-triche peuvent volontairement ignorer les entrées synthétiques.

La signature interne des mises à jour garantit leur provenance et leur intégrité. Elle est indépendante de la signature de code Windows utilisée par SmartScreen.

## Développement

### Prérequis

- Node.js 24 ou version LTS compatible ;
- Rust stable ;
- prérequis Tauri 2 pour Windows : Microsoft C++ Build Tools et WebView2 ;
- pnpm 11.

### Commandes

```bash
pnpm install
pnpm tauri:dev
```

Vérifications disponibles :

```bash
pnpm typecheck
pnpm generate
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri:build --bundles nsis
```

## Architecture

- **Nuxt 4 / Vue 3 / TypeScript** : interface, sélection de la touche, affichage du timer et persistance locale ;
- **Tauri 2** : fenêtre native compacte, commandes sécurisées, icône système et raccourci global ;
- **Rust + Enigo** : événements clavier `Press` / `Release` et timer de sécurité indépendant de l'interface.

## Portabilité

Tauri, Nuxt et Enigo prennent en charge Windows, macOS et Linux, mais une application de simulation clavier doit être validée séparément sur chaque système. macOS demandera notamment une autorisation d'accessibilité, tandis que Linux varie entre X11 et Wayland. Le premier jalon reste donc Windows ; les autres plateformes seront ajoutées après tests natifs.

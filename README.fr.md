<div align="center">

<img width="50%" alt="Bahia" src="https://github.com/user-attachments/assets/8c715a53-a0df-4780-9dd4-09ce3ccd9043" />

  **Un dock léger, inspiré de macOS, pour Windows 10 et 11.**

  Bahia remplace la barre des tâches de Windows par un dock personnalisable et
  une barre de menus facultative à la manière de macOS.

  [English](README.md) · [Español](README.es.md) · **Français**
</div>

> Bahia is a fork of [Muelle](https://github.com/crafter-station/Muelle) (MIT). / Bahía es un fork de Muelle (MIT).

<img width="1500" height="436" alt="Le dock et la barre de menus de Bahia" src="https://github.com/user-attachments/assets/3c91b488-214d-453d-a8c2-a9e0dc7f245e" />

<img width="1000" height="259" alt="Agrandissement des icônes du dock" src="https://github.com/user-attachments/assets/e5b2ce63-cec1-4d1d-8ef1-a4fa1c668c40" />


Agrandissement fluide, aperçus des fenêtres en direct et effet génie facultatif à la réduction.

| Mascotte Clawd | Effet génie à la réduction |
| :---: | :---: |
| ![GIF montrant la mascotte au-dessus du dock et de la barre de menus](https://github.com/user-attachments/assets/580b235e-6141-493a-a333-8f2b96f3acca) | ![GIF montrant une fenêtre réduite dans le dock avec l'effet génie](https://github.com/user-attachments/assets/878503a8-7286-445d-826b-41c18da013bc) |
| **Aperçu de l'utilisation** | **Détail de l'utilisation** |
| ![GIF montrant le dock et la barre de menus qui se masquent et réapparaissent automatiquement](https://github.com/user-attachments/assets/60a8f7c0-20fc-429b-977f-d5b4b7d97d88) | ![GIF montrant une fenêtre réduite dans le dock avec l'effet génie](https://github.com/user-attachments/assets/fbf21cf6-c1c0-4173-9d68-ae883f0d2a45) |

## Fonctionnalités

- Dock effet verre, avec un agrandissement fluide des icônes sous le pointeur.
- Choisissez le bord de l'écran dans Réglages → Dock → *Position à l'écran* :
  gauche, bas ou droite. Le bas est la position par défaut.
- Lancer, épingler, réorganiser et gérer les apps, y compris les apps récentes
  et les fenêtres ouvertes.
- Regrouper des apps épinglées dans une seule icône du dock : déposez une app
  sur une autre, cliquez sur le groupe pour déployer ses apps en éventail, et
  sortez-en une de l'éventail pour la retirer du groupe.
- Survolez une app qui a plusieurs fenêtres pour afficher leurs aperçus en
  direct, avec leur titre ; cliquez sur un aperçu pour passer à cette fenêtre.
  Faites défiler les aperçus pour voir plus de fenêtres, ou appuyez sur Échap
  pour les fermer.
- Les fenêtres réduites se rangent dans l'icône de leur app avec l'effet
  génie ; activez Réglages → Fenêtres → *Afficher les fenêtres réduites* pour
  les garder plutôt sous forme de miniatures en direct à droite du dock.
- Cliquer sur l'app déjà au premier plan la réduit, comme dans la barre des
  tâches de Windows.
- Clic droit sur une app pour voir sa liste de raccourcis, avec icônes, comme
  dans la barre des tâches de Windows : éléments épinglés et récents, ses
  catégories et ses tâches.
- Glisser-déposer des fichiers sur les apps, les Téléchargements ou la
  Corbeille.
- Un dossier de raccourcis à droite du dock : son contenu s'ouvre en menu, avec
  un sous-menu pour chaque dossier qu'il contient, comme une barre d'outils de
  la barre des tâches de Windows 10.
- Masquage automatique facultatif.
- Niveau de batterie facultatif pour les souris et casques compatibles
  connectés. Activez Réglages → Barre de menus → *Afficher la batterie des
  accessoires* ; l'option est désactivée par défaut.
- Une barre de menus en haut de l'écran avec l'app au premier plan, l'heure, le
  réseau, le volume et la batterie, la langue de saisie, le micro et la caméra
  en cours d'utilisation, les icônes masquées de la zone de notification, ainsi
  que les boutons Rechercher, Affichage des tâches et Widgets s'ils figurent
  dans votre barre des tâches Windows.
- Un îlot arrondi dans la barre de menus pour les limites d'utilisation de
  Claude Code, Codex, OpenCode Go, Z.ai GLM Coding Plan, Gemini CLI et
  Antigravity. Z.ai affiche ses quotas sur 5 heures et sur une semaine ; les
  quotas Gemini Pro et Flash s'affichent séparément quand ils sont disponibles.
- Une app Réglages séparée, aux changements appliqués en direct, en espagnol,
  anglais et français, avec une animation de réduction façon génie
  configurable.
- Un processus de surveillance rétablit la barre des tâches de Windows si le
  dock se ferme de façon inattendue.

L'effet génie est **activé par défaut** et se désactive dans Réglages →
Fenêtres. Tant qu'il est activé, Bahia désactive temporairement les
transitions de Windows pour les fenêtres qu'il suit, afin de pouvoir jouer son
animation de réduction ; ces transitions sont rétablies à la fermeture de
Bahia.

## Installer

**[Télécharger Bahia-Setup.exe](https://github.com/kirpulrbx-max/bahia/releases/latest/download/Bahia-Setup.exe)**
(Windows 10 version 1903 ou ultérieure, ou Windows 11 ; 64 bits) ou choisissez
une version dans les [Releases](https://github.com/kirpulrbx-max/bahia/releases).

1. Lancez `Bahia-Setup.exe`. Il s'installe pour votre seul compte : sans
   droits d'administrateur, sans assistant, en deux secondes environ.
2. Bahia est récent : SmartScreen peut ne pas encore le reconnaître et
   indiquer qu'il a protégé votre ordinateur. Choisissez **Informations
   complémentaires → Exécuter quand même**. Voir la
   [politique de signature du code](https://github.com/kirpulrbx-max/bahia/code-signing-policy).
3. Ouvrez **Bahia** depuis le menu Démarrer, puis choisissez **Dock** pour
   personnaliser le dock et la barre de menus.

## Essayer sous Windows

### Prérequis

- Windows 10 (version 1903 ou ultérieure) ou Windows 11. Sous Windows 10, le
  dock et la barre de menus sont opaques plutôt qu'en verre : Windows 10 n'offre
  pas de flou d'arrière-plan aux applications de bureau.
- Rust avec la chaîne d'outils MSVC et les Visual Studio C++ Build Tools
- Node.js et pnpm

### Lancer le dock

Depuis la racine du dépôt :

```powershell
cargo run -p bahia
```

Le dock masque la barre des tâches tant qu'il tourne. Appuyez sur **Ctrl+C**
dans le terminal pour le quitter ; Bahia devrait rétablir la barre des tâches
automatiquement. S'il faut la rétablir à la main :

```powershell
cargo run -p bahia -- --restore
```

### Lancer Réglages

Laissez le dock tourner, ouvrez un second terminal et lancez :

```powershell
cd apps\settings
pnpm install
pnpm tauri dev
```

La fenêtre native de Réglages s'ouvre. Les changements de l'interface se
rechargent à chaud ; après une modification du backend Rust, relancez
`tauri dev`. Les changements de réglages s'appliquent au dock en cours
d'exécution.

## Compiler et tester

Depuis la racine du dépôt :

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
```

Pour compiler l'interface de Réglages :

```powershell
cd apps\settings
pnpm build
```

## Organisation du projet

| Chemin | Rôle |
|---|---|
| `apps/dock` | Dock Windows natif |
| `apps/settings` | App de bureau Tauri pour les réglages |
| `crates/bahia-core` | Logique de disposition et d'animation, indépendante de la plateforme |
| `crates/bahia-shell` | Intégration au shell Windows |
| `crates/bahia-config` | Réglages partagés et persistance |
| `bahia` | Site web Astro |

## Confidentialité

Bahia ne collecte aucune donnée, n'a pas de télémétrie et n'envoie jamais rien
au projet. Il fait deux sortes de requêtes réseau, toutes deux facultatives :

- **Mises à jour :** Réglages demande à GitHub Releases s'il existe une
  nouvelle version à son ouverture, et une fois par jour, quelques minutes
  après l'allumage de l'ordinateur, Bahia fait de même en arrière-plan et
  installe ce qu'il trouve (le dock lui-même n'utilise jamais le réseau).
  Désactivez l'installation dans Réglages → Système → *Installer
  automatiquement les mises à jour*, ou le tout avec *Rechercher
  automatiquement les mises à jour*.
- **Utilisation des fournisseurs d'IA :** tant qu'un client pris en charge
  tourne, Bahia lit sa connexion locale existante et demande à ce fournisseur
  les limites disponibles de l'abonnement toutes les cinq minutes tant que
  l'îlot est visible, ou quand vous appuyez sur Actualiser. Pour Gemini CLI et
  Antigravity, les quotas Gemini Pro et Flash s'affichent séparément quand ils
  sont disponibles ; Antigravity est interrogé via son service local. Les
  identifiants restent en mémoire et ne sont jamais enregistrés par Bahia.
  Pour Z.ai Coding Plan, Bahia lit la clé dans le fichier local `auth.json`
  d'OpenCode ou dans `ZAI_API_KEY` / `ZHIPU_API_KEY`, puis l'envoie au service
  de quotas de Z.ai. Désactivez les requêtes dans Réglages → Barre de menus →
  *Afficher l'utilisation des fournisseurs d'IA*.

Détails : [github.com/kirpulrbx-max/bahia/privacy](https://github.com/kirpulrbx-max/bahia/privacy).

# concatener

`concatener` est un outil en ligne de commande ultra-rapide écrit en Rust qui vous permet de concaténer plusieurs fichiers en un seul fichier de sortie.

## Fonctionnalités

`concatener` est conçu pour être efficace et flexible :

- Concaténer plusieurs fichiers avec des arguments séparés par des espaces
- Support des motifs génériques (*.txt, *.log, etc.)
- Support des répertoires - concaténer tous les fichiers d'un répertoire
- Support récursif des répertoires avec l'option -r/--recursive
- Détection automatique d'encodage - supporte UTF-8, série ISO-8859, Windows-1252, et plus
- Spécification de fichier de sortie personnalisée avec l'option -o/--output
- Compatibilité multi-plateforme (Linux, Windows, macOS)
- Construit avec l'édition Rust 2024 pour des performances optimales

## Dépendances

Ce projet utilise les dépendances suivantes (définies dans `Cargo.toml`) :

- `clap` : Analyse des arguments en ligne de commande avec les macros derive
- `glob` : Correspondance de motifs génériques pour la sélection de fichiers
- `anyhow` : Gestion des erreurs et du contexte
- `encoding_rs` : Détection et conversion automatique d'encodage de caractères

## Installation

### Prérequis

Assurez-vous d'avoir Rust et Cargo installés sur votre système. Vous pouvez les installer en suivant les instructions sur le site web officiel de Rust : [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compilation pour Linux (depuis Linux)
1. Clonez ce dépôt :
    ```sh
    git clone https://github.com/cederig/concatener.git
    cd concatener
    ```
2. Compilez le projet :
    ```sh
    cargo build --release
    ```
    L'exécutable sera situé dans `target/release/concatener`.

### Compilation pour Windows (depuis Linux/macOS)

Pour compiler ce projet pour Windows depuis un autre système d'exploitation (comme Linux ou macOS), vous aurez besoin de la cible Rust pour Windows.

1. Ajoutez la cible Windows à votre installation Rust :
    ```sh
    rustup target add x86_64-pc-windows-gnu
    ```

2. Compilez le projet pour la cible Windows :
    ```sh
    cargo build --release --target=x86_64-pc-windows-gnu
    ```

L'exécutable Windows sera situé dans `target/x86_64-pc-windows-gnu/release/concatener.exe`.

### Compilation pour macOS (depuis Linux/macOS)

Pour compiler ce projet pour macOS depuis un autre système d'exploitation (comme Linux ou macOS), vous aurez besoin de la cible Rust pour macOS.

1. Ajoutez la cible macOS à votre installation Rust (choisissez la bonne architecture) :
   * Pour Macs Intel (x86_64) :
        ```sh
        rustup target add x86_64-apple-darwin
        ```
   * Pour Macs Apple Silicon (aarch64) :
        ```sh
        rustup target add aarch64-apple-darwin
        ```

2. Compilez le projet pour la cible macOS (choisissez la bonne architecture) :
   * Pour Macs Intel :
        ```sh
        cargo build --release --target=x86_64-apple-darwin
        ```
   * Pour Macs Apple Silicon :
        ```sh
        cargo build --release --target=aarch64-apple-darwin
        ```

L'exécutable macOS sera situé dans `target/<votre_cible_macos>/release/concatener`.

## Utilisation

La syntaxe de base est la suivante :

```sh
./concatener [OPTIONS] <ENTRÉES>...
```

### Options

- `-o, --output <FICHIER>` : Chemin du fichier de sortie (Obligatoire)
- `-r, --recursive` : Rechercher récursivement les fichiers dans les répertoires (Optionnel)
- `<ENTRÉES>...` : Fichiers d'entrée, répertoires, ou motifs à concaténer (Obligatoire)

## Exemples

### Concaténer des fichiers spécifiques
```sh
./concatener -o combiné.txt fichier1.txt fichier2.txt fichier3.txt
```

### Concaténer des fichiers en utilisant un motif générique
```sh
./concatener -o tous_logs.txt "*.log"
```

### Concaténer tous les fichiers d'un répertoire
```sh
./concatener -o contenu_repertoire.txt /chemin/vers/repertoire
```

### Utilisation mixte avec fichiers et motifs
```sh
./concatener -o mixte.txt document.txt "*.md" /chemin/vers/configs/
```

### Concaténer tous les fichiers texte du répertoire actuel
```sh
./concatener -o tout_texte.txt "*.txt"
```

### Concaténer récursivement tous les fichiers d'un répertoire et sous-répertoires
```sh
./concatener -r -o tous_fichiers.txt /chemin/vers/repertoire
```

### Concaténer récursivement des fichiers en utilisant des motifs génériques
```sh
./concatener -r -o tous_fichiers_rs.txt "*.rs"
./concatener -r -o tous_fichiers_txt.txt "src/*.txt"
```

**Important** : Lorsque vous utilisez des motifs génériques avec l'option `-r`, utilisez toujours des guillemets pour empêcher le shell de développer le motif avant de le passer au programme :

- ✅ **Correct** : `"*.json"` - Le programme reçoit le motif et recherche récursivement
- ❌ **Incorrect** : `*.json` - Le shell développe le motif, donc seuls les fichiers du répertoire actuel sont trouvés

### Concaténer des fichiers de plusieurs répertoires récursivement
```sh
./concatener -r -o fichiers_projet.txt src/ docs/ tests/
```

### Détection d'Encodage

`concatener` détecte automatiquement et gère divers encodages de fichiers texte :

- **UTF-8** (avec et sans BOM)
- **UTF-16LE** (Little-endian, commun sur Windows)
- **UTF-16BE** (Big-endian, avec support BOM)
- **Windows-1252** (encodage commun Windows)
- **Série ISO-8859** (encodages européens)
- **Encodages asiatiques** (GBK, BIG5, SHIFT_JIS, EUC-JP, EUC-KR)
- **Encodages cyrilliques** (KOI8-R, KOI8-U)

L'outil essaie automatiquement différents encodages par ordre de probabilité et revient à UTF-8 avec remplacement pour les octets non décodables.

Exemple avec encodages mixtes :
```sh
./concatener -o fichiers_mixtes.txt fichier_utf8.txt fichier_utf16le.txt fichier_windows1252.txt
```

## Tests

Ce projet inclut des tests unitaires complets et des benchmarks :

```sh
# Exécuter les tests unitaires
cargo test

# Exécuter les benchmarks de performance
cargo bench

# Exécuter les tests avec sortie
cargo test -- --nocapture
```

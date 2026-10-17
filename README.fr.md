<p align="center">
  <img src="assets/grafito-icon-256x256.png" alt="Grafito" width="110" />
</p>

<h1 align="center">Grafito</h1>

<p align="center">
  <b>Le laboratoire de mathématiques et physique qui explique avant de montrer.</b><br/>
  Géométrie interactive, algèbre computationnelle, statistiques, calcul et
  <b>67 laboratoires de physique visuelle</b> + assistant tuteur + tableau blanc —
  100% Rust natif, esthétique scandinave minimaliste.
</p>

---

## Points forts

- **764 commandes** (alias en espagnol et anglais) avec palette fuzzy bilingue (`Ctrl+K`).
- **Canvas 2D/3D accéléré par GPU** : les compute shaders compilent les expressions de l'utilisateur en WGSL à l'exécution.
- **Géométrie avec contraintes** : solveur de Newton à jacobiennes analytiques ; loci persistants ; opérations booléennes sur polygones.
- **CAS réel** : intégration Risch–Norman, limites de Gruntz, bases de Gröbner, algèbre linéaire exacte rationnelle jusqu'à 32×32, EDO, cartes pas-à-pas.
- **3D + 4D** : surfaces implicites/paramétriques, les six polychores réguliers et les familles simplexe/hypercube en 3–10D.
- **67 laboratoires de physique** avec palette sémantique (une couleur = un sens) et 64 recettes prêtes.
- **70 modèles d'animation** (style 3Blue1Brown) : export GIF/MP4/WebM, sous-titres SRT/ASS, voix Piper TTS.
- **Tuteur socratique** : 68 objectifs d'apprentissage, 18 familles d'exercices déterministes, suivi de maîtrise BKT + Leitner, mode examen.
- **Salle de classe P2P** sans comptes ; **pont Obsidian** avec blocs ```grafito``` ré-exécutables.
- **Serveur MCP** (70 tools) + **mode agent** avec transcript live des outils.

## Installation

| Plateforme | Comment |
|---|---|
| Linux x86_64 | `grafito-linux-x64.tar.gz` depuis les [Releases](https://github.com/Diez111/Grafito/releases) — glibc 2.35+ et `libdbus-1-3` |
| Windows x64 | `grafito-windows-x64.zip` portable — extraire et lancer `grafito.exe` |
| macOS | `grafito-macos-*.tar.gz` (x86_64 / arm64) |
| Source | `git clone` + `cargo run -p grafito-app --release` (Rust ≥ 1.92) |

## Démarrage rapide

```text
y = sin(x)                 # fonction vivante
k = 3                      # slider automatique
Derivative[sin(x)]         # dérivée symbolique
Root[sin(x)]               # racines sur le canvas
```

`Ctrl+K` ouvre la palette complète. Guide enseignant en 5 minutes :
[README en espagnol](README.es.md#guía-docente-en-5-minutos) et recettes dans
[`docs/fisica-animada.md`](docs/fisica-animada.md).

## Documentation

- [Architecture](docs/architecture.md) · [Catalogue des commandes](docs/commands.md) · [Moteur d'animation](docs/animation-engine.md)

## Contribuer et licence

PR bienvenues — `cargo fmt` et `cargo clippy -D warnings` obligatoires
(17 jobs CI). **GPL-3.0-or-later** — voir [LICENSE](LICENSE).

> 🌐 Interface disponible en français, espagnol, anglais, portugais, italien et allemand.

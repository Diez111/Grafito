<p align="center">
  <img src="assets/grafito-icon-256x256.png" alt="Grafito" width="110" />
</p>

<h1 align="center">Grafito</h1>

<p align="center">
  <b>Il laboratorio di matematica e fisica che spiega prima di mostrare.</b><br/>
  Geometria interattiva, algebra computazionale, statistica, calcolo e
  <b>67 laboratori di fisica visiva</b> + tutor assistente + lavagna —
  100% Rust nativo, estetica scandinava minimalista.
</p>

---

## In evidenza

- **764 comandi** (alias in spagnolo e inglese) con palette fuzzy bilingue (`Ctrl+K`).
- **Canvas 2D/3D accelerato su GPU**: i compute shader compilano le espressioni dell'utente in WGSL a runtime.
- **Geometria con vincoli**: solver di Newton con jacobiani analitici; loci persistenti; operazioni booleane sui poligoni.
- **CAS reale**: integrazione Risch–Norman, limiti di Gruntz, basi di Gröbner, algebra lineare razionale esatta fino a 32×32, EDO, schede passo a passo.
- **3D + 4D**: superfici implicite/parametriche, i sei policori regolari e le famiglie simplesso/ipercubo in 3–10D.
- **67 laboratori di fisica** con palette semantica (un colore = un significato) e 64 ricette pronte.
- **70 template di animazione** (in stile 3Blue1Brown): export GIF/MP4/WebM, sottotitoli SRT/ASS, voce Piper TTS.
- **Tutor socratico**: 68 obiettivi di apprendimento, 18 famiglie di esercizi deterministici, tracciamento della padronanza BKT + Leitner, modalità esame.
- **Aula P2P** senza account; **ponte Obsidian** con blocchi ```grafito``` ri-eseguibili.
- **Server MCP** (70 tools) + **modalità agente** con transcript live degli strumenti.

## Installazione

| Piattaforma | Come |
|---|---|
| Linux x86_64 | `grafito-linux-x64.tar.gz` dai [Releases](https://github.com/Diez111/Grafito/releases) — richiede glibc 2.35+ e `libdbus-1-3` |
| Windows x64 | `grafito-windows-x64.zip` portatile — estrai ed esegui `grafito.exe` |
| macOS | `grafito-macos-*.tar.gz` (x86_64 / arm64) |
| Sorgente | `git clone` + `cargo run -p grafito-app --release` (Rust ≥ 1.92) |

## Inizio rapido

```text
y = sin(x)                 # funzione viva
k = 3                      # slider automatico
Derivative[sin(x)]         # derivata simbolica
Root[sin(x)]               # radici sul canvas
```

`Ctrl+K` apre la palette completa. Guida per il docente in 5 minuti:
[README in spagnolo](README.es.md#guía-docente-en-5-minutos) e ricette in
[`docs/fisica-animada.md`](docs/fisica-animada.md).

## Documentazione

- [Architettura](docs/architecture.md) · [Catalogo comandi](docs/commands.md) · [Motore di animazione](docs/animation-engine.md)

## Contribuire e licenza

PR benvenuti — `cargo fmt` e `cargo clippy -D warnings` obbligatori
(17 job CI). **GPL-3.0-or-later** — vedi [LICENSE](LICENSE).

> 🌐 Interfaccia disponibile in italiano, spagnolo, inglese, portoghese, francese e tedesco.

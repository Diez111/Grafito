<p align="center">
  <img src="assets/grafito-icon-256x256.png" alt="Grafito" width="110" />
</p>

<h1 align="center">Grafito</h1>

<p align="center">
  <b>O laboratório de matemática e física que explica antes de mostrar.</b><br/>
  Geometria interativa, álgebra computacional, estatística, cálculo e
  <b>67 laboratórios de física visual</b> + tutor assistente + quadro branco —
  100% Rust nativo, estética escandinava minimalista.
</p>

---

## Destaques

- **764 comandos** (aliases em espanhol e inglês) com paleta fuzzy bilíngue (`Ctrl+K`).
- **Canvas 2D/3D acelerado por GPU**: compute shaders compilam expressões do usuário para WGSL em runtime.
- **Geometria com restrições**: solver de Newton com jacobianos analíticos; loci persistentes; operações booleanas de polígonos.
- **CAS real**: integração Risch–Norman, limites de Gruntz, bases de Gröbner (Buchberger), álgebra linear exata racional até 32×32, EDOs, cards passo a passo.
- **3D + 4D**: superfícies implícitas e paramétricas, os seis policoros regulares e famílias simplex/hipercubo em 3–10D.
- **67 laboratórios de física** com paleta semântica (uma cor = um significado) e 64 receitas prontas.
- **70 templates de animação** (estilo 3Blue1Brown): export GIF/MP4/WebM, legendas SRT/ASS, narração Piper TTS.
- **Tutor socrático**: 68 objetivos de aprendizagem, 18 famílias de exercícios determinísticos, rastreamento de domínio BKT + Leitner, modo exame.
- **Sala de aula P2P** sem contas; **ponte Obsidian** com receitas ```grafito``` re-executáveis.
- **Servidor MCP** (70 tools) + **modo agente** com transcript ao vivo das ferramentas.

## Instalação

| Plataforma | Como |
|---|---|
| Linux x86_64 | `grafito-linux-x64.tar.gz` dos [Releases](https://github.com/Diez111/Grafito/releases) — requer glibc 2.35+ e `libdbus-1-3` |
| Windows x64 | `grafito-windows-x64.zip` portátil — extrair e executar `grafito.exe` |
| macOS | `grafito-macos-*.tar.gz` (x86_64 / arm64) |
| Fonte | `git clone` + `cargo run -p grafito-app --release` (Rust ≥ 1.92) |

## Início rápido

```text
y = sin(x)                 # função viva
k = 3                      # slider automático
Derivative[sin(x)]         # derivada simbólica
Root[sin(x)]               # raízes no canvas
```

`Ctrl+K` abre a paleta completa. Guia do professor em 5 minutos:
[README em espanhol](README.es.md#guía-docente-en-5-minutos) e receitas em
[`docs/fisica-animada.md`](docs/fisica-animada.md).

## Documentação

- [Arquitetura](docs/architecture.md) · [Catálogo de comandos](docs/commands.md) · [Motor de animação](docs/animation-engine.md)

## Contribuir e licença

PRs bem-vindos — `cargo fmt` e `cargo clippy -D warnings` obrigatórios
(17 jobs de CI). **GPL-3.0-or-later** — veja [LICENSE](LICENSE).

> 🌐 Interface disponível em português, espanhol, inglês, italiano, francês e alemão.

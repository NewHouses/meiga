# Código

> Estado: subfase 1.5.

Explicación detallada do código, arquivo por arquivo. Para o *porqué* das decisións xerais, ver [Arquitectura](../arquitectura.md) e [Tecnoloxías](../tecnoloxias.md).

## Mapa do repositorio

```text
meiga/
├─ .cargo/config.toml      → workspace.md
├─ .github/
│  ├─ workflows/ci.yml     → github.md
│  └─ dependabot.yml       → github.md
├─ .gitattributes          → workspace.md
├─ .gitignore              → workspace.md
├─ Cargo.toml              → workspace.md
├─ Cargo.lock              → workspace.md
├─ about.toml              → workspace.md
├─ clippy.toml             → workspace.md
├─ deny.toml               → workspace.md
├─ rust-toolchain.toml     → workspace.md
├─ rustfmt.toml            → workspace.md
├─ README.md, LICENSE, SECURITY.md → workspace.md
├─ crates/
│  ├─ meiga-core/          → meiga-core.md
│  ├─ meiga-render/        → meiga-render.md
│  ├─ meiga-win/           → meiga-win.md
│  └─ meiga/               → meiga.md
├─ tools/                  → ferramentas.md
│  └─ perf/                → perf.md
│     ├─ scenarios/        → escenarios.md
│     └─ gen-bigfile.ps1, run-scenario.ps1, test-scenarios.ps1 → escenarios.md
└─ docs/                   (esta documentación)
```

| Documento | Contido |
|---|---|
| [workspace.md](workspace.md) | Configuración da raíz: Cargo, versión de Rust, *lints*, formato, licenzas e Git. |
| [github.md](github.md) | Integración continua (GitHub Actions) e Dependabot. |
| [meiga-core.md](meiga-core.md) | Lóxica independente de Windows: as cores, os rectángulos, o tamaño da grella, as zonas que cambian, cando debuxar o seguinte fotograma, o ancho dos caracteres e o terminal: a fachada sobre `alacritty_terminal`, o escáner e as gravacións, coas súas probas e un exemplo. |
| [meiga-render.md](meiga-render.md) | Renderizador por CPU: o lenzo, o *framebuffer* e a súa presentación na xanela, as fontes de DirectWrite, a caché de glifos e o debuxo do texto. |
| [meiga-win.md](meiga-win.md) | Capa de plataforma de Windows: a xanela, a súa escala, os seus eventos, o bucle de mensaxes e o temporizador dos fotogramas; as sesións de ConPTY, coas súas probas e un exemplo que tamén as grava. |
| [meiga.md](meiga.md) | O programa executábel: as opcións, o fluxo de fotogramas, a demo de texto e rectángulos, os modos de medición e o rexistro dos fotogramas; o seu manifesto e o `build.rs` que o incrusta. |
| [ferramentas.md](ferramentas.md) | Scripts e modelos de `tools/`: empaquetado, descarga do ConPTY, comprobación das DLL, da xanela, dos fotogramas, das ligazóns da documentación e de que os scripts piden PowerShell 7, e avisos de terceiros. |
| [perf.md](perf.md) | Medición do rendemento (`tools/perf/`): o script que mide os terminais, o código C# que le os procesos, a súa proba e as medicións do arranque e dos fotogramas, tamén os de texto. |
| [escenarios.md](escenarios.md) | Escenarios de carga (`tools/perf/scenarios/`): o spinner, a IA e o arquivo grande, en PowerShell e en bash, o script que mide un terminal mentres corren e as súas probas. |

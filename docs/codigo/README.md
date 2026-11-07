# Código

> Estado: subfase 0.6.

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
└─ docs/                   (esta documentación)
```

| Documento | Contido |
|---|---|
| [workspace.md](workspace.md) | Configuración da raíz: Cargo, versión de Rust, *lints*, formato, licenzas e Git. |
| [github.md](github.md) | Integración continua (GitHub Actions) e Dependabot. |
| [meiga-core.md](meiga-core.md) | Lóxica independente de Windows. |
| [meiga-render.md](meiga-render.md) | Renderizador por CPU. |
| [meiga-win.md](meiga-win.md) | Capa de plataforma de Windows. |
| [meiga.md](meiga.md) | O programa executábel. |
| [ferramentas.md](ferramentas.md) | Scripts e modelos de `tools/`: empaquetado, comprobación das DLL e das ligazóns da documentación, e avisos de terceiros. |
| [perf.md](perf.md) | Medición do rendemento (`tools/perf/`): o script que mide os terminais, o código C# que le os procesos e a súa proba. |

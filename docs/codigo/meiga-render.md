# Código › `meiga-render`

> Estado: subfase 0.3. O crate existe, pero aínda non ten código.

**Responsabilidade:** o renderizado por CPU. Rasteriza cada glifo unha soa vez con DirectWrite e gárdao nunha caché; compón no *framebuffer* só as celas que cambiaron; e presenta na xanela só os rectángulos modificados. Ver [Arquitectura](../arquitectura.md#3-fluxo-de-datos-previsto).

## `crates/meiga-render/Cargo.toml`

```toml
[package]
name = "meiga-render"
description = "CPU renderer of meiga: glyph cache, compositor and presentation of the changed regions."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[dependencies]
meiga-core.workspace = true

[lints]
workspace = true
```

- A estrutura é a mesma que en [`meiga-core`](meiga-core.md#cratesmeiga-corecargotoml).
- `[dependencies] meiga-core.workspace = true`: depende de `meiga-core`, coa ruta que define o `Cargo.toml` raíz. Así o renderizador poderá usar os tipos definidos no núcleo, como as celas da grella.

## `crates/meiga-render/src/lib.rs`

```rust
//! CPU renderer of meiga.
//!
//! Rasterizes glyphs once with DirectWrite into a cache, composes only the
//! cells that changed into a framebuffer and presents only the changed
//! rectangles.
```

Polo de agora só contén a documentación do crate. Ao contrario que `meiga-core`, este crate **si** poderá usar `unsafe`, porque chamará ás APIs gráficas de Windows. Cada bloque `unsafe` terá que levar o seu comentario `// SAFETY:` (regra `undocumented_unsafe_blocks`).

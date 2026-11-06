# Código › `meiga-core`

> Estado: subfase 0.3. O crate existe, pero aínda non ten código.

**Responsabilidade:** toda a lóxica que non depende de Windows: o modelo de xanelas, tabs e grupos; a fachada do terminal; a codificación do teclado e do rato; a busca difusa; e a configuración. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

## `crates/meiga-core/Cargo.toml`

```toml
[package]
name = "meiga-core"
description = "Platform-independent core of meiga: model, terminal facade, input encoding, search and configuration."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[lints]
workspace = true
```

- `name`: o nome do crate. No código úsase con guión baixo: `meiga_core`.
- `description`: unha liña que describe o crate.
- As liñas `campo.workspace = true` herdan os valores de `[workspace.package]` do `Cargo.toml` raíz.
- `[lints] workspace = true`: aplica as regras de calidade comúns.
- Non ten sección `[dependencies]`: non depende de ningún outro crate.

## `crates/meiga-core/src/lib.rs`

```rust
//! Platform-independent core of meiga.
//!
//! This crate holds the logic that does not depend on Windows: the window and
//! tab model, the terminal facade, input encoding, search and configuration.
//! It must stay free of `unsafe` code and of platform APIs, so that it is easy
//! to test and fast to compile.

#![forbid(unsafe_code)]
```

- `lib.rs` é a raíz dunha biblioteca: o compilador empeza por este arquivo, e desde el decláranse os demais módulos.
- As liñas que empezan por `//!` son **documentación do crate**. `cargo doc` convérteas en documentación HTML.
- `#![forbid(unsafe_code)]` é un **atributo interno** (`#!`), que se aplica a todo o crate. Prohíbe calquera bloque `unsafe` en `meiga-core`. Co nivel `forbid`, ningún módulo pode desactivar a regra cun `#[allow]`. Así, esta parte do programa, que será a máis grande, conta con todas as garantías de seguridade de memoria de Rust.

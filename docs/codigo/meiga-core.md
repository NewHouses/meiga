# Código › `meiga-core`

> Estado: subfase 1.1. Só contén o tipo das cores.

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

pub mod color;
```

- `lib.rs` é a raíz dunha biblioteca: o compilador empeza por este arquivo, e desde el decláranse os demais módulos.
- As liñas que empezan por `//!` son **documentación do crate**. `cargo doc` convérteas en documentación HTML.
- `#![forbid(unsafe_code)]` é un **atributo interno** (`#!`), que se aplica a todo o crate. Prohíbe calquera bloque `unsafe` en `meiga-core`. Co nivel `forbid`, ningún módulo pode desactivar a regra cun `#[allow]`. Así, esta parte do programa, que será a máis grande, conta con todas as garantías de seguridade de memoria de Rust.
- `pub mod color;` declara o módulo `color`, que está en `src/color.rs`. Con `pub`, os outros crates poden usalo como `meiga_core::color`.

## `crates/meiga-core/src/color.rs`

```rust
//! Colours.

/// A colour with 8 bits per channel and no transparency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
```

Unha cor en RGB, con 8 bits (0–255) por canle: vermello (`r`), verde (`g`) e azul (`b`). Na subfase 1.1 só se usa para o fondo da xanela; máis adiante servirá para o texto, os temas e a configuración.

- Está en `meiga-core`, e non en `meiga-win`, porque unha cor non depende de Windows. Cada plataforma convértea ao seu formato: para GDI, `meiga-win` convértea nun `COLORREF` (ver [Código › meiga-win](meiga-win.md#conversións)).
- `#[derive(…)]` fai que o compilador escriba automaticamente estas implementacións:
  - `Clone` e `Copy`: unha cor son 3 bytes, así que se copia ao asignala ou ao pasala a unha función, en vez de *moverse* (en Rust, por defecto, asignar un valor transfire a súa propiedade e o orixinal xa non se pode usar);
  - `Debug`: permite mostrala nas mensaxes de diagnóstico con `{:?}`;
  - `PartialEq` e `Eq`: permiten comparar dúas cores con `==`;
  - `Hash`: permite usala como clave nun `HashMap`, por exemplo nunha futura caché de glifos por cor.
- `const fn new`: unha función **constante**, que o compilador pode executar mentres compila. Por iso `meiga` pode declarar a cor do fondo como unha constante: `const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);`.
- Os campos son públicos (`pub`): calquera combinación de tres bytes é unha cor válida, así que non hai ningunha regra que protexer.

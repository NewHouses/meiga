# Código › `meiga` (o programa)

> Estado: subfase 0.3. O programa arranca e sae inmediatamente co código 0.

**Responsabilidade:** unir as pezas. Creará a xanela, o bucle de mensaxes e o fluxo de fotogramas, usando os outros tres crates. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

## `crates/meiga/Cargo.toml`

```toml
[package]
name = "meiga"
description = "A lightweight terminal for Windows, with performance as the top priority."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[dependencies]
meiga-core.workspace = true
meiga-render.workspace = true
meiga-win.workspace = true

[lints]
workspace = true
```

- Como o crate ten un `src/main.rs`, Cargo sabe que é un **binario**: o resultado da compilación é `meiga.exe`.
- Depende dos tres crates de biblioteca. É o único que pode velos todos.

## `crates/meiga/src/main.rs`

```rust
//! meiga: a lightweight terminal for Windows.

// Release builds are GUI applications without a console window. Debug builds
// keep the console so that diagnostics are visible during development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {}
```

- `#![cfg_attr(condición, atributo)]` aplica o atributo só se se cumpre a condición.
  - `debug_assertions` está activado nas compilacións de depuración e desactivado en *release*.
  - Polo tanto, en *release* aplícase `windows_subsystem = "windows"`, e meiga é unha aplicación gráfica sen xanela de consola. Ver [Tecnoloxías › Subsistema](../tecnoloxias.md#6-subsistema-de-windows).
- `fn main() {}` é o punto de entrada do programa. Polo de agora está baleiro: o programa sae co código 0. A subfase 1.1 creará eiquí a xanela e o bucle de mensaxes.

## Comprobacións feitas

- `cargo build --release` xera `target\release\meiga.exe`, duns 200 KB.
- Ao executalo, sae co código 0.
- `dumpbin /dependents` mostra que só depende de DLL do sistema (`KERNEL32.dll`, `ntdll.dll` e un *API set*), grazas ao CRT estático.

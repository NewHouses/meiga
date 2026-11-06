# Código › `meiga-win`

> Estado: subfase 0.3. O crate existe, pero aínda non ten código.

**Responsabilidade:** a capa de plataforma. Envolve todo o que é específico de Windows: ConPTY, a detección das shells instaladas, o sistema de arquivos, o portapapeis, a xanela Win32 e a xestión do DPI. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

## `crates/meiga-win/Cargo.toml`

```toml
[package]
name = "meiga-win"
description = "Windows platform layer of meiga: ConPTY, shell profiles, file system, clipboard and Win32 windowing."
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

Igual que [`meiga-render`](meiga-render.md#cratesmeiga-rendercargotoml): metadatos herdados, dependencia de `meiga-core` e regras de calidade comúns.

## `crates/meiga-win/src/lib.rs`

```rust
//! Windows platform layer of meiga.
//!
//! Wraps ConPTY, shell profile discovery, the file system, the clipboard, the
//! Win32 window and DPI handling. Every `unsafe` block in this crate must
//! explain why it is sound in a `// SAFETY:` comment.
```

Polo de agora só contén a documentación do crate, que lembra a súa regra principal: este será o lugar con máis código `unsafe` do proxecto, e cada bloque terá que explicar por que é seguro.

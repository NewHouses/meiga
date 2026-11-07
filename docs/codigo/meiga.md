# Código › `meiga` (o programa)

> Estado: subfase 1.1. O programa abre unha xanela baleira, que non fai nada ata que se pecha, e entón sae co código 0.

**Responsabilidade:** unir as pezas. Crea a xanela e o bucle de mensaxes e, máis adiante, o fluxo de fotogramas, usando os outros tres crates. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeigacargotoml) | O crate e as súas dependencias. |
| [`build.rs`](#cratesmeigabuildrs) | Incrusta o manifesto en `meiga.exe` ao compilar. |
| [`meiga.exe.manifest`](#cratesmeigameigaexemanifest) | O manifesto da aplicación: como debe executala Windows. |
| [`src/main.rs`](#cratesmeigasrcmainrs) | O punto de entrada. |

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
- Depende dos tres crates de biblioteca. É o único que pode velos todos. `meiga-render` aínda non se usa: chegará coa subfase 1.2.
- Cargo atopa `build.rs` só, porque está na raíz do crate; non fai falta declaralo.

## `crates/meiga/build.rs`

```rust
//! Embeds the application manifest (`meiga.exe.manifest`) in `meiga.exe`.

use std::env;

const MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\meiga.exe.manifest");

fn main() {
    println!("cargo::rerun-if-changed=meiga.exe.manifest");

    // The options below belong to the MSVC linker (link.exe).
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" || target_env != "msvc" {
        return;
    }

    println!("cargo::rustc-link-arg-bins=/MANIFEST:EMBED");
    println!("cargo::rustc-link-arg-bins=/MANIFESTINPUT:{MANIFEST}");
    // The manifest already says how to run (trustInfo); the linker must not
    // add its own copy.
    println!("cargo::rustc-link-arg-bins=/MANIFESTUAC:NO");
}
```

Un ***build script***: Cargo compila e executa `build.rs` antes de compilar o crate. O script fala con Cargo escribindo liñas `cargo::instrución=valor` na súa saída.

- `env!("CARGO_MANIFEST_DIR")` lese ao compilar `build.rs`: é a ruta absoluta do cartafol do crate. `concat!` xunta os textos, tamén ao compilar. A ruta ten que ser absoluta porque o ligador non traballa no cartafol do crate.
- `cargo::rerun-if-changed=meiga.exe.manifest`: Cargo só volta executar o script se cambia o manifesto (ou o propio `build.rs`). Sen esta liña, executaríao con calquera cambio no crate. Cando o script se volta executar, Cargo volta ligar `meiga.exe` co manifesto novo; comprobouse cambiando a data do arquivo.
- `CARGO_CFG_TARGET_OS` e `CARGO_CFG_TARGET_ENV`: variábeis que Cargo lle pasa ao script coa plataforma **para a que** se compila (que non ten por que ser na que se compila). As opcións son do ligador de Microsoft (`link.exe`, ver [Tecnoloxías › MSVC](../tecnoloxias.md#13-msvc-e-o-windows-sdk)), e outro ligador non as entendería.
- `unwrap_or_default()`: se a variábel non existe, un texto baleiro. Nunca provoca un pánico, como esixen as regras do proxecto contra `unwrap()`.
- `cargo::rustc-link-arg-bins=…`: pasa unha opción ao ligador, só ao ligar os binarios (`meiga.exe`) e non as bibliotecas.
- `/MANIFEST:EMBED`: o ligador xera un manifesto e métello ao `.exe` como un recurso, en vez de deixalo nun arquivo á parte que se podería perder ao copiar o programa.
- `/MANIFESTINPUT:…`: o contido do manifesto sae deste arquivo.
- `/MANIFESTUAC:NO`: o ligador engade pola súa conta un `trustInfo` (como se executa o programa respecto ao UAC). O manifesto de meiga xa o leva, así que o do ligador desactívase: o que vai dentro do `.exe` é exactamente o que hai no arquivo.

**Comprobado:** sen `build.rs`, `meiga.exe` non leva ningún manifesto. Rust non lle pide ningún ao ligador, e Windows trataría meiga como un programa que non sabe escalar: en pantallas ao 150 %, estiraría a súa imaxe e o texto veríase borroso.

## `crates/meiga/meiga.exe.manifest`

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<!-- Application manifest of meiga.exe. build.rs embeds it with the linker. -->
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <!-- Run without elevation, with the rights of the user who starts it. -->
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <!-- Tested on Windows 10 and 11: no compatibility shims. -->
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <!-- meiga scales itself on every monitor, so Windows must not stretch its
       pixels. -->
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
```

O **manifesto da aplicación** é un XML que Windows le antes de arrancar o programa. Que é e como funciona explícase en [Tecnoloxías › Manifesto](../tecnoloxias.md#23-manifesto-da-aplicación-e-dpi). Leva tres declaracións:

| Elemento | Que declara | Sen el |
|---|---|---|
| `requestedExecutionLevel level="asInvoker"` | meiga corre cos permisos de quen o abre e nunca pide ser administrador. `uiAccess="false"`: non necesita controlar as xanelas doutros programas. | Windows aplicaría a algúns programas regras de compatibilidade pensadas para os anteriores ao UAC. |
| `supportedOS Id="{8e0f7a12-…}"` | meiga está feito para Windows 10 e 11 (os dous comparten este identificador). | Windows activaría comportamentos de compatibilidade para programas antigos; por exemplo, as funcións que dan a versión de Windows mentirían. |
| `dpiAwareness` = `PerMonitorV2` | meiga escala el mesmo o seu contido en cada monitor, e Windows non debe estirar os seus píxeles. | En pantallas con escala, Windows debuxaría meiga ao 100 % e estiraría a imaxe: texto borroso. |

- O elemento antigo `dpiAware` non fai falta: `dpiAwareness` funciona desde Windows 10 versión 1607, e meiga necesita a 1809 (pola ConPTY).
- Os comentarios van en inglés, como no resto do código.
- O ligador reescribe o XML ao incrustalo (sen comentarios nin espazos), pero o contido é o mesmo.

## `crates/meiga/src/main.rs`

```rust
//! meiga: a lightweight terminal for Windows.

// Release builds are GUI applications without a console window. Debug builds
// keep the console so that diagnostics are visible during development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::process::ExitCode;

use meiga_core::color::Rgb;
use meiga_win::window::{self, Window, WindowOptions};

/// Colour of the empty window.
const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let text = format!("meiga stopped because of an unexpected error.\n\n{error}");
            // Release builds have no console, so the message box is the only
            // way to tell the user.
            eprintln!("{text}");
            window::show_error("meiga", &text);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), meiga_win::Error> {
    let window = Window::create(&WindowOptions {
        title: "meiga",
        background: BACKGROUND,
    })?;
    window.show()?;
    window::run_message_loop()
}
```

- `#![cfg_attr(condición, atributo)]` aplica o atributo só se se cumpre a condición. `debug_assertions` está activado nas compilacións de depuración e desactivado en *release*, así que en *release* meiga é unha aplicación gráfica, sen xanela de consola. Ver [Tecnoloxías › Subsistema](../tecnoloxias.md#6-subsistema-de-windows).
- `#![forbid(unsafe_code)]`: o programa non ten nin un bloque `unsafe`. Todas as chamadas a Windows pasan por `meiga-win`, que as envolve en funcións seguras.
- `use meiga_win::window::{self, Window, WindowOptions};` importa o módulo (`self`, para escribir `window::run_message_loop()`) e os dous tipos.
- `BACKGROUND`: a cor do fondo, un gris azulado escuro (`#16181D`). É provisional: o tema de meiga chegará na subfase 8.3.
- `fn main() -> ExitCode`: `main` pode devolver o código de saída do proceso: `ExitCode::SUCCESS` (0) ou `ExitCode::FAILURE` (1).
- `run()` fai os tres pasos (crear, mostrar e o bucle de mensaxes) e devolve `Result<(), meiga_win::Error>`. O operador `?` remata a función e devolve o erro se un paso falla.
- Se algo falla:
  - `format!` compón o texto, coa mensaxe do erro (por exemplo, `CreateWindowExW failed: …`);
  - `eprintln!` escríbeo na saída de erros. Vese nas compilacións de depuración, que teñen consola; en *release* non hai consola, e Rust descarta o texto sen fallar;
  - `window::show_error` móstrao nunha caixa de mensaxe, que en *release* é a única forma de avisar;
  - o proceso remata co código 1.

## Comprobacións feitas

- `cargo build --release` xera `target\release\meiga.exe`, duns 210 KB.
- Ao abrilo, aparece a xanela, co fondo escuro e o título «meiga», sen ningunha consola. Ao pechala, o proceso remata co código 0.
- `tools/check-window.ps1` comproba o subsistema, a xanela, a escala por monitor e o código de saída (ver [Código › ferramentas](ferramentas.md#toolscheck-windowps1)).
- O manifesto que leva dentro, extraído con `mt.exe`, ten as tres declaracións, e un só `trustInfo`.
- `dumpbin /dependents` mostra que só depende de DLL do sistema: `kernel32.dll`, `user32.dll`, `gdi32.dll`, `ntdll.dll`, `api-ms-win-core-synch-l1-2-0.dll` e `KERNEL32.dll`. `kernel32.dll` aparece dúas veces porque a biblioteca estándar de Rust e `windows-sys` declaran as súas funcións de forma distinta (ver [Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust)); Windows cárgaa unha soa vez.

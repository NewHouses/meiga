# Código › `meiga` (o programa)

> Estado: subfase 1.6. O programa é un terminal: lanza nunha pseudoconsola o programa da súa liña de comandos ou, sen el, PowerShell, e mostra a súa saída na xanela, coas cores, os estilos, o cursor e o título que pon o programa. Presenta só as zonas que cambian. Tamén ten tres modos de medición (`--bench`) e un rexistro dos fotogramas (`--trace-frames`). Na 1.6 desapareceu a demo, e chegaron `session.rs` e `view.rs`. Aínda non se pode escribir nel (subfase 1.7), e a grella non cambia co tamaño da xanela (1.9).

**Responsabilidade:** unir as pezas. Crea a xanela, lanza o programa nunha sesión de ConPTY e leva a súa saída á pantalla co fluxo de fotogramas, usando os outros tres crates. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates) e, para o camiño da saída, [Arquitectura › Da saída á pantalla](../arquitectura.md#da-saída-á-pantalla-o-aviso-a-copia-e-o-fotograma).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeigacargotoml) | O crate e as súas dependencias. |
| [`build.rs`](#cratesmeigabuildrs) | Incrusta o manifesto en `meiga.exe` ao compilar. |
| [`meiga.exe.manifest`](#cratesmeigameigaexemanifest) | O manifesto da aplicación: como debe executala Windows. |
| [`src/main.rs`](#cratesmeigasrcmainrs) | O punto de entrada: as opcións, a xanela e a mensaxe de erro. |
| [`src/options.rs`](#cratesmeigasrcoptionsrs) | As opcións da liña de comandos, co programa que se lanza. |
| [`src/app.rs`](#cratesmeigasrcapprs) | `App`, o *handler* da xanela: crea o contido, o terminal ou un modo de medición, e leva os eventos a el e aos fotogramas. |
| [`src/session.rs`](#cratesmeigasrcsessionrs) | A sesión do terminal: o programa, e o fío lector que lle dá a súa saída ao terminal, responde ao programa e esperta a xanela. |
| [`src/view.rs`](#cratesmeigasrcviewrs) | `TerminalView`, o terminal na xanela: a súa copia da pantalla, posta ao día antes de cada fotograma, e o debuxo das celas, as súas liñas e o cursor. |
| [`src/frames.rs`](#cratesmeigasrcframesrs) | `Frames`, o fluxo de fotogramas: que cambiou, cando debuxalo e como chega á xanela. |
| [`src/trace.rs`](#cratesmeigasrctracers) | O rexistro dos fotogramas, en CSV. |
| [`src/bench.rs`](#cratesmeigasrcbenchrs) | Os tres modos de medición: fotogramas parciais, completos e completos de texto, sen parar. |

Así se reparten o traballo:

```mermaid
graph LR
    conpty["ConPTY<br/>(meiga-win)"] -- "a saída do programa" --> reader["Fío lector<br/>(session.rs)"]
    reader -- "feed, baixo o lock" --> shared["Shared (Mutex)<br/>o terminal e tres marcas"]
    reader -- "WM_WAKE, se non<br/>había outro pendente" --> win["meiga-win<br/>Window"]
    win -- "eventos e WM_PAINT" --> app["App"]
    app -- "App::wake:<br/>Frames::changed" --> frames["Frames"]
    frames -- "Scene::update e,<br/>despois, draw" --> content["Content<br/>TerminalView ou Bench"]
    shared -- "copia do que cambiou<br/>(baixo o lock)" --> content
    frames -- "cando" --> pacer["FramePacer<br/>(meiga-core)"]
    frames -- "compoñer e presentar" --> surface["Surface<br/>(meiga-render)"]
    frames -- "unha liña por fotograma" --> trace["Trace"]
```

1. O fío lector da sesión ([`session.rs`](#cratesmeigasrcsessionrs)) dálle cada bloque de saída ao terminal, baixo o *lock* de `Shared`, envía as respostas ao programa e esperta a xanela con `WM_WAKE`, só se non había outro aviso pendente.
2. `App::wake` ([`app.rs`](#cratesmeigasrcapprs)) pon o título e, se cambiou a pantalla, pídelle un fotograma a `Frames` (`changed`), que o fai no momento ou ao final do intervalo ([`frames.rs`](#cratesmeigasrcframesrs)).
3. O fotograma pon primeiro ao día o contido (`Scene::update`): a vista do terminal ([`view.rs`](#cratesmeigasrcviewrs)) copia baixo o *lock* só o que cambiou, e di que zonas son. Despois, fóra do *lock*, compón esas zonas (`draw`) e preséntaas.
4. Coa xanela minimizada non hai fotogramas, e a vista non copia nada: só fai o que non pode esperar (`Scene::update_hidden`): rematar á súa hora unha actualización sincronizada que o programa deixou aberta, para que as súas preguntas teñan resposta ([`frames.rs`](#coa-xanela-minimizada-hidden)).

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
- Depende dos tres crates de biblioteca. É o único que pode velos todos. Desde a subfase 1.2 úsaos os tres: `meiga-render` debuxa e presenta. Desde a 1.6, `meiga-win` lanza tamén o programa nunha pseudoconsola, e o terminal de `meiga-core` le a súa saída.
- Non depende de `windows-sys`: os tipos de Windows que necesita, `HWND` e `HDC`, publícaos `meiga-win` (ver [Código › meiga-win](meiga-win.md#cratesmeiga-winsrclibrs)).
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

mod app;
mod bench;
mod frames;
mod options;
mod session;
mod trace;
mod view;

use std::env;
use std::error::Error;
use std::process::ExitCode;

use meiga_win::window::{self, Window, WindowOptions};

use crate::app::{App, TITLE};
use crate::options::{Options, USAGE};
use crate::trace::Trace;

/// The exit code when the command line is wrong.
const USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    let options = match Options::parse(env::args_os().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            report(&format!("{error}\n\n{USAGE}"));
            return ExitCode::from(USAGE_ERROR);
        }
    };
    match run(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&format!(
                "meiga stopped because of an unexpected error.\n\n{error}"
            ));
            ExitCode::FAILURE
        }
    }
}

fn run(options: &Options) -> Result<(), Box<dyn Error>> {
    let trace =
        match &options.trace_frames {
            Some(path) => Some(Trace::create(path).map_err(|error| {
                format!("Cannot create the frame log {}: {error}", path.display())
            })?),
            None => None,
        };
    let window = Window::create(&WindowOptions { title: TITLE })?;
    let mut app = App::new(&window, options, trace)?;
    window.run(&mut app)
}

/// Tells the user about an error. Release builds have no console, so the
/// message box is the only way.
fn report(text: &str) {
    eprintln!("{text}");
    window::show_error("meiga", text);
}
```

- `#![cfg_attr(condición, atributo)]` aplica o atributo só se se cumpre a condición. `debug_assertions` está activado nas compilacións de depuración e desactivado en *release*, así que en *release* meiga é unha aplicación gráfica, sen xanela de consola. Ver [Tecnoloxías › Subsistema](../tecnoloxias.md#6-subsistema-de-windows).
- `#![forbid(unsafe_code)]`: o programa non ten nin un bloque `unsafe`. Todas as chamadas a Windows pasan por `meiga-win` e `meiga-render`, que as envolven en funcións seguras.
- `mod …;` declara os sete módulos do programa, que están nos arquivos do mesmo nome.
- `use meiga_win::window::{self, Window, WindowOptions};` importa o módulo (`self`, para escribir `window::show_error`) e os dous tipos.
- `TITLE` é o título da xanela, «meiga»: o que ten ao abrila e sempre que o programa do terminal non pon outro (ver [`app.rs`](#cratesmeigasrcapprs)).
- `fn main() -> ExitCode`: `main` pode devolver o código de saída do proceso: `ExitCode::SUCCESS` (0), `ExitCode::FAILURE` (1) ou outro número, con `ExitCode::from`.
- **As opcións.** `env::args_os()` dá os argumentos da liña de comandos, como `OsString`, os textos do sistema operativo, que en Windows poden non ser UTF-8 válido. `.skip(1)` salta o primeiro, que é o nome do programa. Se as opcións están mal, meiga mostra o erro e como se usa ([`USAGE`](#cratesmeigasrcoptionsrs)) e sae co código 2, o habitual para os erros de uso.
- **`run`** fai os pasos en orde:
  1. Se se pediu o rexistro dos fotogramas, crea o arquivo. Se non pode, o erro di que arquivo era: `map_err` cambia o erro por un texto, e `?` convérteo en `Box<dyn Error>`, que acepta textos.
  2. Crea a xanela, aínda oculta.
  3. Crea `App`, o *handler*, con todas as opcións: lanza o programa no terminal, ou prepara un modo de medición, e compón xa o primeiro fotograma no *framebuffer* (ver [`app.rs`](#cratesmeigasrcapprs)).
  4. `window.run` mostra a xanela e entrega os seus eventos a `App` ata que se pecha. Ao rematar `run`, `app` destrúese e, con el, a sesión do terminal.
- `match &options.trace_frames`: o `&` mira o valor sen quitarllo a `options`. As sangrías raras de `let trace =` son as de rustfmt.
- `Box<dyn Error>`: calquera erro nunha caixa (ver [`HandlerError`](meiga-win.md#handler)). Os erros de `meiga-win`, de `meiga-render` e os textos convértense solos con `?`.
- Se algo falla, `report` avisa:
  - `eprintln!` escribe o texto na saída de erros. Vese nas compilacións de depuración, que teñen consola; en *release* non hai consola, e Rust descarta o texto sen fallar;
  - `window::show_error` móstrao nunha caixa de mensaxe, que en *release* é a única forma de avisar;
  - o proceso remata co código 1, ou co 2 se as opcións estaban mal.
- **O programa do terminal non é meiga.** Se non se pode lanzar, non é un erro de meiga: o terminal mostra por que, e a xanela queda aberta (ver [`session.rs`](#cratesmeigasrcsessionrs)). E remate como remate o programa, meiga sae co código 0 cando se pecha a xanela: o código do programa vese na última liña do terminal ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).

## `crates/meiga/src/options.rs`

```rust
pub const USAGE: &str = "Usage: meiga [--trace-frames <file>] [--] [<program> [<arguments>]]
       meiga [--trace-frames <file>] --bench partial|full|text

  <program>              The program to run in the terminal, and its arguments.
                         Without it, PowerShell 7 (pwsh) or, if it is not
                         installed, Windows PowerShell.
  --                     The end of the options: what follows is the program.
  --trace-frames <file>  Write a line per frame to <file> (CSV).
  --bench partial        Instead of a terminal, move a small square without stopping.
  --bench full           Instead of a terminal, change the whole window without stopping.
  --bench text           Instead of a terminal, fill the whole window with text that
                         changes at every frame.";

pub enum BenchMode {
    Partial,
    Full,
    Text,
}

pub struct Options {
    pub trace_frames: Option<PathBuf>,
    pub bench: Option<BenchMode>,
    /// The command line of the program, quoted for Windows, or `None` for
    /// the default shell.
    pub command: Option<String>,
}
```

As opcións da liña de comandos. Hai dúas formas de usar meiga: cun terminal, que executa un programa, ou cun modo de medición, que non executa ningún. Sen nada, meiga abre un terminal coa shell por defecto.

| Argumento | Que fai |
|---|---|
| `<programa> [<argumentos>]` | O programa que se executa no terminal, cos seus argumentos. Sen el, PowerShell 7 (`pwsh`) ou, se non está instalado, Windows PowerShell (ver [`session.rs`](#cratesmeigasrcsessionrs)). Úsano `check-window.ps1`, `check-present.ps1` e `run-scenario.ps1 -Start`, que lanzan meiga cun programa propio. |
| `--` | O fin das opcións: o seguinte argumento é o programa, aínda que empece por `-`. |
| `--trace-frames <arquivo>` | Escribe unha liña por fotograma nese arquivo CSV (ver [`trace.rs`](#cratesmeigasrctracers)). É o que usan `check-present.ps1` e `measure-frames.ps1`. |
| `--bench partial` | En vez dun terminal, un cadrado de 64 píxeles que cruza a xanela sen parar: fotogramas parciais. |
| `--bench full` | En vez dun terminal, a xanela enteira cambia de cor en cada fotograma: fotogramas completos. |
| `--bench text` | En vez dun terminal, a xanela enteira chea de texto que cambia en cada fotograma: fotogramas completos de texto. |

Por exemplo:

```text
meiga
meiga cmd
meiga --trace-frames frames.csv pwsh -NoProfile -File "C:\my scripts\spinner.ps1"
meiga --bench full
```

```rust
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    let mut program = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--trace-frames") => {
                let path = args.next().ok_or("--trace-frames needs the path of a file.")?;
                options.trace_frames = Some(PathBuf::from(path));
            }
            Some("--bench") => {
                options.bench = Some(match args.next().as_deref().and_then(OsStr::to_str) {
                    Some("partial") => BenchMode::Partial,
                    Some("full") => BenchMode::Full,
                    Some("text") => BenchMode::Text,
                    _ => return Err("--bench needs 'partial', 'full' or 'text'.".into()),
                });
            }
            Some("--") => {
                program = args.next();
                break;
            }
            Some(option) if option.starts_with('-') => {
                return Err(format!("Unknown option: {option}"));
            }
            _ => {
                program = Some(arg);
                break;
            }
        }
    }
    let Some(program) = program else {
        return Ok(options);
    };
    if options.bench.is_some() {
        return Err("--bench does not run a program.".into());
    }
    let text = |arg: OsString| {
        arg.into_string().map_err(|arg| {
            format!("The command line can only have valid Unicode: {}", arg.to_string_lossy())
        })
    };
    let program = text(program)?;
    let arguments = args.map(text).collect::<Result<Vec<_>, _>>()?;
    let command = shell::command_line(&program, &arguments)
        .ok_or_else(|| format!("The name of a program cannot have quotes: {program}"))?;
    options.command = Some(command);
    Ok(options)
}
```

- **Sen dependencias.** Hai crates para ler opcións (`clap`, por exemplo), pero para tan poucas abonda un bucle, e meiga mantén as dependencias ao mínimo ([ADR 0006](../adr/0006-licenzas-e-dependencias.md)).
- `impl IntoIterator<Item = OsString>`: calquera colección de argumentos. O programa pásalle os da liña de comandos, e as probas, unha lista feita a man.
- `arg.to_str()` dá o texto se é UTF-8 válido. A ruta do rexistro non se converte: `PathBuf::from` acepta calquera nome de arquivo de Windows. `ok_or` converte un `Option` nun `Result`: se non hai seguinte argumento, o erro. `as_deref` mira o `OsString` como `&OsStr` sen copialo.
- **O programa.** O primeiro argumento que non é unha opción, ou o que vén despois de `--`, é o programa, e todos os que veñen despois, os seus argumentos, aínda que parezan opcións de meiga: en `meiga cmd --bench full`, `--bench full` é de `cmd`. Así, os argumentos do programa nunca necesitan `--` ([Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows)). `break` sae do bucle, e `args` queda co resto.
- **Unha opción descoñecida** é un erro, e non o programa: un argumento que empeza por `-` antes do programa é para meiga. Así, `meiga -NoProfile pwsh` avisa, en vez de intentar lanzar un programa chamado `-NoProfile`. Un programa cun nome así lánzase con `--` diante.
- `let … else`: se non hai programa, as opcións xa están, co `command` a `None`, que é a shell por defecto.
- **`--bench` cun programa** é un erro: os modos de medición non executan ningún programa, e ignoralo en silencio faría crer que se mediu con el.
- **Só Unicode válido.** `into_string` converte o `OsString` nun `String` sen copialo, se é Unicode válido; se non, devolve o orixinal, que a mensaxe mostra con `to_string_lossy`, cambiando o que non é válido por �. En Windows, un argumento pode ter UTF-16 que non é Unicode válido (a metade dun par de *surrogates*); cambialo por � daría ao programa un argumento distinto sen avisar. Un argumento así nunca é unha opción: chega ao programa, e dá este erro.
- `args.map(text).collect::<Result<Vec<_>, _>>()?` converte cada argumento e xúntaos nun `Vec`, ou para no primeiro erro: un iterador de `Result` pódese recoller nun `Result` dunha colección.
- **A liña de comandos.** En Windows, un programa recibe os seus argumentos nun só texto ([Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows)). [`shell::command_line`](meiga-core.md#cratesmeiga-coresrcshellrs) xunta o programa e os argumentos, coas comiñas e as barras invertidas que fagan falta para que o programa os divida igual ca os recibiu meiga: `C:\my scripts\spinner.ps1`, por exemplo, vai como `"C:\my scripts\spinner.ps1"`. Por iso `Options` garda un `String`, e non unha lista: é o que recibe a pseudoconsola.
- **Un programa con comiñas** é un erro: ningún nome de arquivo as pode ter, e Windows le o nome do programa sen ningunha forma de escapalas. `command_line` devolve `None`, e `ok_or_else` convérteo no erro, creando o texto só se fai falta.
- `Options` deriva `Default` (as tres opcións a `None`) e `PartialEq`, para comparalas nas probas.

**Probas.** Catro: sen argumentos, e con `--` só, a shell por defecto; as opcións cos seus valores, en calquera orde, e os tres modos; o programa e os seus argumentos forman a liña de comandos, coas comiñas de Windows, as opcións despois do programa son súas, e despois de `--` o programa pode empezar por `-`; e os erros: unha opción descoñecida, tamén antes do programa (`-NoProfile pwsh`), `--trace-frames` sen arquivo, `--bench` sen modo ou cun que non existe, `--bench` cun programa e un programa con comiñas.

## `crates/meiga/src/app.rs`

`App` é o *handler* da xanela ([`Handler`](meiga-win.md#handler)): crea o contido, recibe os eventos, pásallos ao contido e decide os fotogramas con `Frames`.

### `Content` e `App`

```rust
/// The title of the window while the program does not give one.
pub const TITLE: &str = "meiga";

/// What the window shows. Both hold a whole font, so they go in boxes.
enum Content {
    Terminal(Box<TerminalView>),
    Bench(Box<Bench>),
}

impl Scene for Content {
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        match self {
            Content::Terminal(view) => view.update(now, damage),
            Content::Bench(_) => None,
        }
    }

    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        match self {
            Content::Terminal(view) => view.update_hidden(now),
            Content::Bench(_) => None,
        }
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        match self {
            Content::Terminal(view) => view.draw(canvas),
            Content::Bench(bench) => bench.draw(canvas),
        }
    }
}

pub struct App {
    frames: Frames,
    content: Content,
    /// The session of the terminal, which closes when it is dropped. `None`
    /// for the benchmarks, and if the program could not start.
    _session: Option<Pty>,
}
```

- `TITLE`: o título da xanela ao abrila, e sempre que o programa non pon ningún. `main.rs` crea a xanela con el.
- `Content` é o que mostra a xanela: o terminal ([`TerminalView`](#cratesmeigasrcviewrs)) ou un dos modos de medición ([`Bench`](#cratesmeigasrcbenchrs)). Implementa [`Scene`](#scene), así que `Frames` pode poñelo ao día, debuxalo e, coa xanela minimizada, deixarlle facer o que non pode esperar (`update_hidden`), sen saber cal é. Os modos de medición non teñen nada que poñer ao día nin que facer coa xanela minimizada (`None`): as súas zonas chegan con `step`, antes de cada fotograma.
- **As dúas variantes en `Box`.** Un `enum` ocupa en memoria o que a súa variante máis grande, e as dúas levan dentro unha fonte enteira ([`Font`](meiga-render.md#cratesmeiga-rendersrctextrs)); a vista, ademais, a súa copia da pantalla, coa paleta de 256 cores. Na `Box` van na memoria dinámica, e o `enum` só garda un punteiro: mover o contido, de `terminal` a `App::new` e de aí a `App`, só copia ese punteiro.
- **`_session`**: a sesión do terminal ([`Pty`](meiga-win.md#cratesmeiga-winsrcconptyrs)). Ninguén a le: vai en `App` para vivir tanto coma ela, porque destruíla pecha a sesión ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)). Ao pecharse a xanela, `run` remata, `App` destrúese e o programa e a súa consola rematan tamén, como comproba `check-window.ps1`. O guión baixo do nome di que non se usa, pero, ao contrario que `_` só, non a destrúe no momento. É `None` nos modos de medición, e se o programa non se puido lanzar.

### `App::new` e `terminal`

```rust
pub fn new(window: &Window, options: &Options, trace: Option<Trace>) -> Result<Self, HandlerError> {
    // Only the content with text reads the fonts of the system.
    let (mut content, session) = match options.bench {
        Some(BenchMode::Text) => {
            let system = FontSystem::new()?;
            let font = Font::new(&system, &FONT_FAMILIES, FONT_POINTS, window.dpi())?;
            let bench = Bench::new(BenchMode::Text, Some(font));
            (Content::Bench(Box::new(bench)), None)
        }
        Some(mode) => (Content::Bench(Box::new(Bench::new(mode, None))), None),
        None => terminal(window, options.command.as_deref())?,
    };
    let mut frames = Frames::new(window, &mut content, trace)?;
    if options.bench.is_some() {
        frames.schedule(window)?;
    }
    Ok(Self { frames, content, _session: session })
}
```

- **O contido:** un modo de medición, como antes, ou o terminal, que crea `terminal` co programa das opcións. `as_deref` dá un `Option<&str>` sen copiar o texto.
- **As fontes.** Só o contido con texto le as fontes do sistema ([`FontSystem`](meiga-render.md#fontsystem)): o terminal e o modo `text`, cunha fonte ao DPI da xanela ([`Window::dpi`](meiga-win.md#windowdpi)). Os modos `partial` e `full` non cargan nada de DirectWrite, e miden o mesmo ca na subfase 1.2.
- Os erros de `meiga-render` convértense en `HandlerError` con `?`, como os de `meiga-win`: se non hai ningunha das fontes, meiga para cunha caixa de mensaxe que o di.
- `Frames::new` crea o *framebuffer* co tamaño da xanela, pon ao día o contido e compóno enteiro nel. Así, cando Windows pide o primeiro `WM_PAINT`, só hai que copialo.
- **As marcas** (o bordo magenta das zonas presentadas) empezan sempre desactivadas, e a tecla P actívaas: no terminal son unha axuda para ver que presenta cada fotograma, e nos modos de medición engadirían traballo aos fotogramas que se miden.
- Un modo de medición pide xa o seu primeiro fotograma (`schedule`), que chegará en canto a xanela empece o seu bucle de mensaxes.

```rust
fn terminal(window: &Window, command: Option<&str>) -> Result<(Content, Option<Pty>), HandlerError> {
    let font = Font::new(&FontSystem::new()?, &FONT_FAMILIES, FONT_POINTS, window.dpi())?;
    let cell = font.cell();
    let client = window.client_size()?;
    // The grid keeps this size until it can change with the window
    // (subphase 1.9).
    let cells = |pixels: u32, cell: u32| u16::try_from(pixels / cell.max(1)).unwrap_or(u16::MAX);
    let mut terminal = Terminal::new(GridSize::new(
        cells(client.width, cell.width),
        cells(client.height, cell.height),
    ));
    terminal.set_cell_size(cell);
    let size = terminal.size();
    let shared = Arc::new(Mutex::new(Shared::new(terminal)));
    let waker = window.waker();
    let session = match session::start(command, size, &shared, &waker) {
        Ok(pty) => Some(pty),
        Err(message) => {
            lock(&shared).terminal.feed(message.as_bytes());
            None
        }
    };
    let input = session.as_ref().map(Pty::input);
    let view = TerminalView::new(shared, input, waker, font, window.dpi());
    Ok((Content::Terminal(Box::new(view)), session))
}
```

O terminal, paso a paso:

1. **A fonte**, ao DPI da xanela: dela sae o tamaño da cela.
2. **A grella:** tantas celas enteiras como caben na xanela. O que sobra son as marxes, que a vista pinta coa cor do fondo. `cell.max(1)` evita dividir entre 0, e `try_from` dá `u16::MAX` se o número non cabe: unha cela sempre ten píxeles, e ningunha xanela ten tantas celas, pero o tipo non o garante. [`Terminal::new`](meiga-core.md#cratesmeiga-coresrctermmodrs) dálle polo menos o seu tamaño mínimo. A grella queda así ata a subfase 1.9 ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
3. **`set_cell_size`**: co tamaño da cela, o terminal responde aos programas que preguntan polo tamaño da grella en píxeles (`CSI 14 t`, [Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control)).
4. **`size`**: o tamaño que ten de verdade o terminal, co mínimo xa aplicado. A pseudoconsola ten que ter o mesmo.
5. **`Shared`** ([`session.rs`](#cratesmeigasrcsessionrs)), o terminal e as súas marcas, vai nun `Arc<Mutex<…>>`: o `Mutex` deixa usalo desde dous fíos, un cada vez, e o `Arc`, un contador de referencias que se pode compartir entre fíos, mantén vivo o que comparten mentres o use calquera deles.
6. **O `Waker`** da xanela é o que usa o fío lector para espertala ([Código › meiga-win](meiga-win.md#cratesmeiga-winsrcwindowrs)): garda o *handle* da xanela, e pódese copiar e levar a outro fío.
7. **A sesión** lánzase xa, antes do primeiro fotograma: así o programa empeza canto antes, e o que escriba mentres meiga compón ese fotograma xa está no terminal, ou chega en seguida ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Se non se pode lanzar, a mensaxe vai ao terminal coma se a escribise o programa, e o primeiro fotograma móstraa.
8. **`Pty::input`** dá unha copia da entrada da sesión para a vista, que a usa para as poucas respostas que aparecen cando ela remata unha actualización sincronizada (ver [`update`](#poñer-ao-día-update)).

### `wake`: o aviso do fío lector

```rust
fn wake(&mut self, window: &Window) -> Result<(), HandlerError> {
    let Content::Terminal(view) = &self.content else {
        return Ok(());
    };
    let (title, screen) = {
        let mut shared = lock(view.shared());
        let title = mem::take(&mut shared.title_changed)
            .then(|| shared.terminal.title().map(str::to_owned));
        (title, shared.screen_changed)
    };
    if let Some(title) = title {
        window.set_title(title.as_deref().unwrap_or(TITLE));
    }
    if screen {
        self.frames.changed(window, &mut self.content)?;
    }
    Ok(())
}
```

O fío lector espertou a xanela con `WM_WAKE`: cambiou o título, a pantalla ou as dúas cousas, ou o programa abriu unha actualización sincronizada ([Tecnoloxías › Avisar á xanela](../tecnoloxias.md#36-avisar-á-xanela-desde-outro-fío-postmessage)).

- `let … else`: se o contido non é o terminal, non hai nada que facer.
- **Baixo o *lock*, o mínimo.** O `MutexGuard` que devolve `lock` solta o `Mutex` ao destruírse, ao final do bloque `{ … }`. Dentro só se len as marcas do título e da pantalla e, se cambiou, cópiase o título. `lock` recibe un `&Mutex`, e o `&Arc<Mutex<…>>` de `shared()` convértese só (*deref coercion*).
- **O título.** `mem::take` devolve a marca e déixaa a `false`: a xanela colleu o título, e o seguinte cambio voltará espertala. `bool::then` executa a *closure* só se a marca estaba posta, e dá o seu resultado en `Some`. Queda un `Option<Option<String>>`: o de fóra di se cambiou; o de dentro, o título, ou `None` se o programa o quitou, e entón a xanela volta a «meiga». Copialo (`to_owned`) permite usalo despois de soltar o *lock*, e os títulos cambian pouco.
- `set_title` ([`SetWindowTextW`](../tecnoloxias.md#39-o-foco-e-o-título-da-xanela)) chámase fóra do *lock*, como todo o que non é copiar ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **A pantalla.** A súa marca só se le: quítaa o fotograma, cando copia o que cambiou ([`TerminalView::update`](#poñer-ao-día-update)). Mentres tanto, o fío lector non volta espertar a xanela pola pantalla: coa xanela minimizada, que non copia nada, a saída só a esperta unha vez, agás para rematar á súa hora unha actualización sincronizada. `Frames::changed` pide un fotograma, que o *pacer* fai no momento ou ao final do intervalo; coa xanela minimizada, en vez del, a vista fai só o que non pode esperar ([`hidden`](#coa-xanela-minimizada-hidden)).
- **Unha actualización sincronizada** tamén esperta a xanela por eiquí, e `wake` non mira a súa marca: para abrila, o programa escribiu, así que a marca da pantalla xa está posta. `Frames::changed` leva entón á vista, no fotograma ou, coa xanela minimizada, en `update_hidden`, e a vista le a hora límite e dilla a `Frames` ([`end_late_sync`](#as-actualizacións-sincronizadas-end_late_sync)).

### Os eventos

```rust
fn event(&mut self, window: &Window, event: Event) -> Result<(), HandlerError> {
    match event {
        Event::Resized { size, minimized } => {
            self.frames.resized(window, size, minimized, &mut self.content)?;
            if self.is_bench() {
                self.frames.schedule(window)?;
            }
        }
        Event::SizeMoveStarted => self.frames.size_move_started(),
        Event::SizeMoveEnded => self.frames.size_move_ended(window, &mut self.content)?,
        Event::FrameTimer => {
            if let Content::Bench(bench) = &mut self.content {
                for area in bench.step(self.frames.size()) {
                    self.frames.damage(area);
                }
            }
            self.frames.timer(window, &mut self.content)?;
            if self.is_bench() {
                self.frames.schedule(window)?;
            }
        }
        Event::Wake => self.wake(window)?,
        Event::Focus { focused } => {
            if let Content::Terminal(view) = &mut self.content {
                view.focus(focused, Instant::now());
                self.frames.changed(window, &mut self.content)?;
            }
        }
        // Until the keyboard reaches the program (subphase 1.7).
        Event::KeyDown { key: Key::P, repeat: false } => self.frames.toggle_marks(window),
        _ => {}
    }
    Ok(())
}

fn paint(&mut self, _window: &Window, dc: HDC, area: Rect) -> Result<(), HandlerError> {
    self.frames.paint(dc, area)
}
```

| Evento | Que fai |
|---|---|
| `Wake` | O fío lector avisou: `wake` pon o título e, se cambiou a pantalla, pide un fotograma; coa xanela minimizada, a vista só fai o que non pode esperar. |
| `Focus` | A xanela gañou ou perdeu o foco ([`Event::Focus`](meiga-win.md#cratesmeiga-winsrceventrs)): o cursor do terminal cambia ([`focus`](#terminalview)), e pídese un fotograma para debuxalo. Nos modos de medición, nada. |
| `FrameTimer` | Toca debuxar. Nun modo de medición, primeiro avanza un paso e marca as zonas que cambian. Despois, `Frames` debuxa o fotograma pendente; se non había ningún, é o momento que pediu o terminal, como o seguinte cambio do pestanexo do cursor, e pide un fotograma para el ([`timer`](#pedir-un-fotograma)). Coa xanela minimizada, o temporizador só se programa para a hora límite dunha actualización sincronizada, e a vista remátaa sen debuxar nada. Nun modo de medición, despois pide o seguinte fotograma. |
| `Resized` | `Frames` adapta o *framebuffer* ao tamaño novo, ou, se a xanela se minimizou, deixa de debuxar, e o temporizador queda só para a hora límite dunha actualización sincronizada aberta, se a hai. Nun modo de medición, ao restaurala, os fotogramas continúan. A grella do terminal non cambia: se a xanela medra, as marxes medran, e se se fai pequena, a grella córtase. |
| `SizeMoveStarted` e `SizeMoveEnded` | O usuario empeza e remata de mover ou redimensionar a xanela: mentres tanto, o *framebuffer* reserva memoria con marxe ([`Fit::Room`](meiga-render.md#fit-e-surface)). Ao rematar, preséntase o que cambiase mentres tanto. |
| Tecla P | Activa ou desactiva as marcas. `repeat: false`: manter a tecla premida non as acende e apaga sen parar. Ata que o teclado chegue ao programa, na subfase 1.7, é a única tecla que fai algo. |
| Calquera outro | Nada: as outras teclas e o rato aínda non chegan ao programa. |

- `WM_PAINT` (`paint`) copia a zona que pide Windows desde o *framebuffer*. `_window`: o guión baixo indica que o parámetro non se usa.
- `is_bench` di se o contido é un modo de medición: `matches!` compara un valor cun patrón e dá `true` ou `false`.
- **Os préstamos.** En `Focus`, `view` é unha referencia `&mut` a `self.content`. Despois de `view.focus(…)` xa non se usa, e Rust permite pasar outra vez `&mut self.content` a `Frames::changed`.
- `if let Content::Bench(bench) = &mut self.content`: entra só se o contido é un modo de medición, e dá unha referencia a el.

## `crates/meiga/src/session.rs`

A **sesión do terminal**: o programa que corre nunha pseudoconsola, o fío lector que lle dá a súa saída ao terminal, e o que ese fío comparte coa xanela ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). A sesión en si, coas súas tubaxes e os seus dous fíos, é de `meiga-win` ([`Conpty` e `Pty`](meiga-win.md#cratesmeiga-winsrcconptyrs)); este módulo decide que programa se lanza e que fai o fío lector con cada bloque de saída.

### `Shared` e `lock`

```rust
pub struct Shared {
    pub terminal: Terminal,
    /// The screen may have changed since the window last copied it.
    pub screen_changed: bool,
    /// The title changed since the window last took it.
    pub title_changed: bool,
    /// The window knows when the synchronized update that the program has
    /// open ends, and ends it then, even while it is minimized.
    sync_noted: bool,
}

impl Shared {
    /// Notes that the terminal read output, with `changes`, and says whether
    /// the window has to be woken up for it.
    pub fn note(&mut self, changes: Changes) -> bool {
        let screen = !mem::replace(&mut self.screen_changed, true);
        let title = changes.title && !mem::replace(&mut self.title_changed, true);
        // Until the update ends, the terminal keeps what the program writes,
        // its questions too: the window has to end it in time, even if it
        // does not copy the screen.
        let sync =
            self.terminal.sync_deadline().is_some() && !mem::replace(&mut self.sync_noted, true);
        screen || title || sync
    }
}

pub fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}
```

O que comparten o fío lector e a xanela: o terminal ([`Terminal`](meiga-core.md#cratesmeiga-coresrctermmodrs)) e tres marcas, detrás dun `Mutex`. O fío lector dálle a saída ao terminal, e a xanela copia del o que cambiou: os dous collendo o mesmo `Mutex`, un cada vez ([ADR 0005](../adr/0005-fios-e-fotogramas.md)).

- **As marcas** din para que se espertou a xanela e aínda non o atendeu: `screen_changed`, a pantalla, ata que a xanela a copia; `title_changed`, o título, ata que a xanela o colle; e `sync_noted`, unha actualización sincronizada aberta, ata que a xanela sabe cando remata ([máis abaixo](#as-actualizacións-sincronizadas-end_late_sync)). Son tres porque se atenden en momentos distintos: o título, en canto chega o aviso ([`App::wake`](#wake-o-aviso-do-fío-lector)); a pantalla, no seguinte fotograma, que pode chegar despois ou, coa xanela minimizada, non chegar; e a actualización sincronizada, cando a vista le a súa hora límite, tamén coa xanela minimizada.
- `sync_noted` é privada: só a cambian `note` e `end_late_sync`, os dous en `Shared`. As outras dúas son públicas, porque a xanela as le e as quita.
- **`note`** anota que o terminal leu saída, cos seus `Changes`, e di se hai que espertar a xanela: só se unha das marcas que toca non estaba posta. `mem::replace` pon o valor novo e devolve o vello. A da pantalla ponse con calquera saída; se ao final non cambiou ningunha cela, o fotograma non ten zonas e non debuxa nada ([`draw`](#draw)). A do título, só se cambiou. A da actualización sincronizada, se despois do bloque o programa ten unha aberta (`sync_deadline`).
- **Por que avisar dunha actualización sincronizada.** Mentres está aberta (`ESC [ ? 2026 h`), `vte` garda todo o que escribe o programa, tamén as súas preguntas, ata que o programa a remata ou ata que alguén a remata á súa hora límite: o terminal non ten temporizador ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control)). Se o programa non a remata, remátaa a xanela, co temporizador dos fotogramas, e para iso ten que saber que hai unha. Antes de cada fotograma xa o mira, pero coa xanela minimizada non hai fotogramas, e a marca da pantalla, que xa está posta, non a esperta: sen esta marca, un programa que preguntase algo dentro dunha actualización que non remata agardaría a resposta ata que se restaurase a xanela ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- Así, por moita saída que chegue, o fío lector non envía outro aviso pola pantalla ata que a xanela a copie, nin outro polo título ata que o colla, nin outro pola actualización sincronizada ata que a xanela saiba cando remata: os avisos non se acumulan na cola da xanela ([Tecnoloxías › Avisar á xanela](../tecnoloxias.md#36-avisar-á-xanela-desde-outro-fío-postmessage)).
- **`lock`** colle o `Mutex`. Se un fío tivese un pánico mentres o ten, o `Mutex` quedaría envelenado (*poisoned*), e `lock()` devolvería un erro que leva igualmente o acceso, que `PoisonError::into_inner` colle. Ningún dos dous fíos deixa o terminal a medio cambiar, así que a xanela pode seguir mostrándoo. En *release* non pasa nunca, porque un pánico remata o proceso (`panic = "abort"`); e así o código non necesita `unwrap()`, que as regras do proxecto prohiben.

### As actualizacións sincronizadas: `end_late_sync`

```rust
pub fn end_late_sync(&mut self, now: Instant, answers: &mut Vec<u8>) -> (Option<Instant>, bool) {
    let mut title = false;
    if self.terminal.sync_deadline().is_some_and(|deadline| deadline <= now) {
        let changes = self.terminal.end_sync();
        if changes.responses {
            self.terminal.take_responses(answers);
        }
        title = changes.title && !mem::replace(&mut self.title_changed, true);
    }
    let deadline = self.terminal.sync_deadline();
    self.sync_noted = deadline.is_some();
    (deadline, title)
}
```

Remata á súa hora a actualización sincronizada que o programa deixou aberta. Chámaa a vista do terminal, baixo o *lock*: antes de cada fotograma ([`update`](#poñer-ao-día-update)) e, coa xanela minimizada, cando o fío lector a esperta ou vence o temporizador ([`update_hidden`](#coa-xanela-minimizada-update_hidden)).

1. Se hai unha aberta e a súa hora límite xa chegou (`deadline <= now`), o terminal móstraa tal como está (`end_sync`): le o que gardaba ([Código › meiga-core](meiga-core.md#cratesmeiga-coresrctermmodrs)). Iso pode traer respostas ás preguntas que o programa fixo mentres tanto, que se engaden a `answers`, e un título novo, que pon a súa marca coma en `note`.
2. Lese a hora límite da que quede aberta: a mesma, se aínda non chegou, ou ningunha, se se acaba de rematar: `end_sync` le todo o que gardaba e non deixa ningunha aberta.
3. `sync_noted` queda posta se hai unha aberta: desde agora a xanela sabe cando remata, e o fío lector non a esperta máis por ela. Se non hai ningunha, quítase, e a seguinte que abra o programa voltará espertala.
4. Devolve esa hora, para que `Frames` programe para ela o temporizador dos fotogramas, e se hai que espertar a xanela polo título: `false` se a súa marca xa estaba posta, porque entón a xanela xa ten un aviso pendente por el.

- **Por que en `Shared`.** Rematar a actualización e cambiar as marcas tocan o que está detrás do *lock*, e `sync_noted` cámbiaa tamén `note`, desde o fío lector: así, as dúas funcións que a cambian están xuntas, e a vista usa o mesmo código con fotogramas e sen eles.
- **As respostas** non se envían eiquí: `end_late_sync` só as recolle, baixo o *lock*, e a vista envíaas despois, fóra del ([`send`](#poñer-ao-día-update)). O búfer `answers` é o da vista, que se reutiliza.
- **Unha marca, e non a hora.** Se o programa remata a actualización e abre outra antes de que a xanela mire, a marca segue posta e o fío lector non avisa: o temporizador vence á hora da primeira, que é antes, e a vista le entón a da segunda. Como moito, a xanela esperta unha vez de máis.
- `now` e `answers` son parámetros, e non o reloxo e a sesión: así, a proba decide a hora e le as respostas.

### `start`: lanzar o programa

```rust
const DEFAULT_SHELLS: [&str; 2] = ["pwsh", "powershell"];

pub fn start(
    command: Option<&str>,
    size: GridSize,
    shared: &Arc<Mutex<Shared>>,
    waker: &Waker,
) -> Result<Pty, String> {
    let conpty = Conpty::load();
    let reader = || Reader {
        shared: Arc::clone(shared),
        waker: waker.clone(),
        answers: Vec::new(),
    };
    let (program, started) = match command {
        Some(command) => (command, conpty.spawn(command, size, reader())),
        None => {
            let [first, second] = DEFAULT_SHELLS;
            match conpty.spawn(first, size, reader()) {
                Err(error) if is_not_found(&error) => {
                    (second, conpty.spawn(second, size, reader()))
                }
                started => (first, started),
            }
        }
    };
    started.map_err(|error| start_error(program, &error))
}

fn is_not_found(error: &Error) -> bool {
    error
        .os_error()
        .is_some_and(|error| error.kind() == io::ErrorKind::NotFound)
}

fn start_error(command: &str, error: &Error) -> String {
    format!("[could not start `{command}`: {error}]\x1b[?25l")
}
```

Lanza o programa nunha sesión do tamaño da grella. A súa saída vai ao terminal de `shared`, e o fío lector esperta a xanela con `waker`.

- **Que ConPTY.** [`Conpty::load`](meiga-win.md#cratesmeiga-winsrcconptyrs) usa a que vai ao lado de `meiga.exe` (`conpty.dll` e `OpenConsole.exe`) ou, se non está, a de Windows ([ADR 0004](../adr/0004-conpty-empaquetado.md)).
- **Que programa.** O da liña de comandos, se o hai, xa coas súas comiñas ([`options.rs`](#cratesmeigasrcoptionsrs)). Se non, `pwsh`, PowerShell 7, e, se non está instalado, `powershell`, Windows PowerShell, que vén con Windows. `let [first, second] = DEFAULT_SHELLS;` dálle un nome a cada elemento do *array*.
- **Só se non se atopa.** O segundo só se proba se o primeiro non existe: `CreateProcessW` busca o programa como explica [Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows) e, se non o atopa, falla co erro 2, que Rust clasifica como `io::ErrorKind::NotFound`. Con calquera outro erro, como a falta de permisos ou un fallo da ConPTY, o segundo fallaría igual, e probalo só agocharía o primeiro erro. `is_not_found` mira o erro de Windows que leva dentro o de `meiga-win` ([`Error::os_error`](meiga-win.md#cratesmeiga-winsrcerrorrs)). O patrón `Err(error) if …` só encaixa se se cumpre a condición; os demais resultados, de éxito ou de erro, quedan como están (`started`).
- **Un `Reader` por intento.** `spawn` quédase co `Output` que recibe, así que, para poder probar dous programas, `reader` é unha *closure* que crea un novo cada vez. `Arc::clone` só suma un ao contador de referencias: os dous apuntan ao mesmo `Shared`. Copiar o `Waker` tampouco custa nada.
- **O erro** é un texto para o terminal, non un erro de meiga: [`terminal`](#appnew-e-terminal) dállo ao terminal coma se o escribise o programa, e a xanela queda aberta ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Di que programa, coa súa liña de comandos, e por que, entre corchetes coma a liña do final: ``[could not start `no-such-program`: CreateProcessW failed: …]``. `ESC [ ? 25 l` oculta o cursor, porque non hai nada no que escribir. Como todos os textos do programa, vai en inglés.

### O fío lector: `Reader`

```rust
struct Reader {
    shared: Arc<Mutex<Shared>>,
    waker: Waker,
    /// The answers for the program, kept to avoid an allocation each time.
    answers: Vec<u8>,
}

impl Output for Reader {
    fn received(&mut self, bytes: &[u8], input: &Input) {
        let wake = {
            let mut shared = lock(&self.shared);
            let changes = shared.terminal.feed(bytes);
            if changes.responses {
                shared.terminal.take_responses(&mut self.answers);
            }
            shared.note(changes)
        };
        // Outside the lock: the window never waits for the input pipe.
        if !self.answers.is_empty() {
            input.write(&self.answers);
            self.answers.clear();
        }
        if wake {
            self.waker.wake();
        }
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        let wake = {
            let mut shared = lock(&self.shared);
            let new_line = shared.terminal.cursor().column > 0;
            let changes = shared.terminal.feed(end(exit_code, new_line).as_bytes());
            shared.note(changes)
        };
        if wake {
            self.waker.wake();
        }
    }
}
```

`Reader` é o [`Output`](meiga-win.md#cratesmeiga-winsrcconptyrs) da sesión: `meiga-win` chama a `received` desde o fío lector, con cada bloque que le da pseudoconsola, e a `ended` despois do último.

- **`Send + 'static`.** `Output` esíxeo, porque o `Reader` vai ao fío lector e vive tanto coma el. Por iso leva un `Arc`, e non unha referencia, e un `Waker`, que garda o *handle* da xanela coma un número, que pode ir a outro fío ([Código › meiga-win](meiga-win.md#cratesmeiga-winsrcwindowrs)). O terminal tamén pode: `meiga-core` compróbao ao compilar.

**`received`, paso a paso:**

1. Baixo o *lock*, o terminal le o bloque (`feed`) e di que pasou (`Changes`).
2. Se hai respostas para o programa, `take_responses` móveas ao búfer, aínda baixo o *lock*.
3. `note` di se hai que espertar a xanela. Eiquí remata o bloque, e con el o *lock*: só dura o que o terminal tarda en ler o bloque ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
4. Fóra do *lock*, as respostas van ao programa con `input`, a entrada da sesión, que `meiga-win` lle pasa ao `Output` con cada bloque. `Input::write` non agarda: deixa os bytes para o fío escritor. Así, a xanela, que necesita o *lock* para copiar, nunca agarda polo envío das respostas. O búfer baléirase sen liberar a memoria.
5. Se fai falta, `wake` envía `WM_WAKE` á xanela con `PostMessageW`, que volta de inmediato.

- **Por que responde o fío lector.** As respostas saen xusto despois do bloque que as pide, e na mesma orde ca as preguntas, aínda que a xanela estea ocupada ou minimizada. Desde a xanela, agardarían a que atendese o aviso, e a súa orde coas outras respostas perderíase ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). O ConPTY xa pregunta ao arrancar polos atributos do terminal (`ESC [ c`), e moitos programas preguntan pola cor do fondo antes de escoller as súas cores ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control)). Dentro dunha actualización sincronizada, as preguntas agardan co resto da saída: se o programa a remata, respóndeas o fío lector; se pasa a súa hora límite, a vista ([`end_late_sync`](#as-actualizacións-sincronizadas-end_late_sync)).
- **`ended`**: o programa rematou, e a pseudoconsola pechou a súa saída despois do último bloque. O terminal mostra unha liña co código de saída (`end`), nunha fila propia se o cursor non está ao principio dunha (`column > 0`), e o fío lector esperta a xanela para que a mostre. A xanela queda aberta, coa última pantalla do programa.

### O fin do programa: `end`

```rust
fn end(exit_code: Option<u32>, new_line: bool) -> String {
    let new_line = if new_line { "\r\n" } else { "" };
    match exit_code {
        Some(code) => {
            format!("\x1b[0m{new_line}[process exited with code {code} (0x{code:08x})]\x1b[?25l")
        }
        None => format!("\x1b[0m{new_line}[process exited]\x1b[?25l"),
    }
}
```

O que mostra o terminal cando remata o programa, coma no Windows Terminal: por exemplo, `[process exited with code 0 (0x00000000)]`.

- `ESC [ 0 m` volta ás cores e aos estilos por defecto, por se o programa deixou outros.
- `\r\n` leva o cursor ao principio da fila seguinte, se fai falta.
- O código vai en decimal e en hexadecimal, con oito cifras (`{code:08x}`): moitos códigos de Windows recoñécense mellor así, como `0xc000013a`, o dun programa que rematou con Ctrl+C.
- `None`: o programa seguía vivo uns segundos despois de que a pseudoconsola pechase, o que só pasa se falla a propia ConPTY ([`Output::ended`](meiga-win.md#cratesmeiga-winsrcconptyrs)). Entón, a liña vai sen código.
- `ESC [ ? 25 l` oculta o cursor: xa non se pode escribir nada.
- Vai ao terminal coma se fose saída do programa, e o terminal colócaa coma calquera outro texto: tamén se a pantalla está chea e hai que desprazala.

**Probas.** Tres: a xanela só se esperta unha vez ata que colle o que cambiou (a segunda saída xa non a esperta; un título novo si, porque é outra cousa; e despois de copiar a pantalla, ou de coller o título, voltan espertala); unha actualización sincronizada esperta a xanela unha vez, aínda coa marca da pantalla posta, coma coa xanela minimizada, e remata á súa hora (o título e a pregunta `ESC [ c` que leva dentro agardan; antes da hora, `end_late_sync` só devolve a hora; na hora, saen o título e a resposta, `ESC [ ? 6 c`, e xa non queda nada que esperar; e a seguinte actualización volta espertar a xanela); e a liña do final vai nunha fila propia, coas cores por defecto aínda que o programa deixase o vermello, e oculta o cursor. Comproba tamén o texto exacto co código 0 sen fila nova, e sen código.

## `crates/meiga/src/view.rs`

A **vista do terminal**: o terminal na xanela. Garda a súa propia copia da pantalla, un [`Screen`](meiga-core.md#cratesmeiga-coresrctermscreenrs), que pon ao día desde o terminal da sesión antes de cada fotograma, e debuxa desde ela as celas, as súas liñas e o cursor. É a [`Scene`](#scene) do terminal.

**Por que unha copia.** Compoñer leva tempo, e o fío lector, que está lendo a saída do programa, non debe agardar por iso. Baixo o *lock* só se copian as celas que cambiaron; compoñer e presentar fanse despois, fóra del ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).

### Constantes

```rust
pub const FONT_FAMILIES: [&str; 2] = ["Cascadia Mono", "Consolas"];
pub const FONT_POINTS: f32 = 12.0;

/// How far dim text (SGR 2) goes from its colour towards the one of its
/// background: a third of the way, out of 255.
const DIM: u8 = 85;

/// The dots per inch at a scale of 100 %.
const BASE_DPI: u32 = 96;
```

- **A fonte:** Cascadia Mono ou, se non está, Consolas, a 12 puntos, como no Windows Terminal. Antes estaban na demo; agora úsanas o terminal e o modo `--bench text`, que así debuxa coa mesma fonte.
- `DIM`: o texto tenue (`SGR 2`) vai un terzo do camiño, 85 de 255, desde a súa cor cara á do seu fondo ([`Rgb::mix`](meiga-core.md#cratesmeiga-coresrccolorrs)).
- `BASE_DPI`: os puntos por polgada ao 100 %, para escalar o grosor do cursor ([`thickness`](#o-pestanexo-caret)).

### `TerminalView`

```rust
pub struct TerminalView {
    shared: Arc<Mutex<Shared>>,
    input: Option<Input>,
    waker: Waker,
    font: Font,
    cell: Size,
    screen: Screen,
    caret: Caret,
    run: Vec<CellChar>,
    answers: Vec<u8>,
}
```

| Campo | Que garda |
|---|---|
| `shared` | O que comparte co fío lector: o terminal e as súas marcas ([`Shared`](#shared-e-lock)). |
| `input` | A entrada da sesión, para as respostas que aparecen cando a vista remata unha actualización sincronizada. `None` se o programa non se lanzou. |
| `waker` | Para espertar a xanela cando esa actualización cambia o título. |
| `font` e `cell` | A fonte, coa súa caché de glifos, e o tamaño da súa cela. |
| `screen` | A copia da pantalla: as celas, o cursor, a paleta e o que cambiou desde o último fotograma. |
| `caret` | Como se ve e como pestanexa o cursor ([`Caret`](#o-pestanexo-caret)). |
| `run` e `answers` | Os caracteres dun tramo de celas e as respostas, que se gardan para non reservar memoria cada vez. |

```rust
pub fn new(shared: Arc<Mutex<Shared>>, input: Option<Input>, waker: Waker, font: Font, dpi: u32) -> Self {
    Self {
        shared,
        input,
        waker,
        cell: font.cell(),
        font,
        screen: Screen::new(),
        caret: Caret::new(dpi, Instant::now()),
        run: Vec::new(),
        answers: Vec::new(),
    }
}

pub fn focus(&mut self, focused: bool, now: Instant) {
    if focused == self.caret.focused {
        return;
    }
    if focused {
        // The settings can change while meiga runs: for example, when
        // the user connects to the session through Remote Desktop.
        self.caret.read_settings();
        self.caret.since = now;
    }
    self.caret.focused = focused;
    self.screen.mark_cursor_changed();
}
```

- **`new`**: `cell: font.cell()` vai antes ca `font` porque os campos se avalían na orde na que se escriben, e despois de mover a fonte ao seu campo xa non se podería usar. A copia da pantalla empeza baleira, sen celas: a primeira copia dálle o tamaño do terminal e marca todas as celas como cambiadas.
- **`focus`**: a xanela gañou ou perdeu o foco ([Tecnoloxías › O foco](../tecnoloxias.md#39-o-foco-e-o-título-da-xanela)).
  - Ao gañalo, volta ler os axustes do cursor de Windows, que poden cambiar mentres meiga corre, e o pestanexo empeza de novo: recuperar o foco é actividade.
  - `mark_cursor_changed` marca as celas do cursor como cambiadas, e o seguinte fotograma debúxao coa forma nova: a súa ou, sen o foco, un rectángulo baleiro. Ese fotograma pídeo [`App`](#os-eventos).
  - O cursor empeza sen o foco (`focused: false`): Windows envía `WM_SETFOCUS` ao mostrar e activar a xanela.
- `area` dá os píxeles dunhas columnas dunha fila, e `shared` dálle a [`App::wake`](#wake-o-aviso-do-fío-lector) o que comparte co fío lector.

### Poñer ao día: `update`

```rust
fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
    let (sync, title) = {
        let mut shared = lock(&self.shared);
        // If the program did not end its synchronized update in time, it
        // is shown as it is.
        let late = shared.end_late_sync(now, &mut self.answers);
        shared.terminal.copy_damage(&mut self.screen);
        shared.screen_changed = false;
        late
    };
    self.send(title);

    // What the program shows is activity: the cursor blinks again from
    // the start, shown.
    if self.screen.all_changed() || self.screen.changes().next().is_some() {
        self.caret.since = now;
    }
    let (shown, blink) = self.caret.phase(self.screen.cursor(), now);
    if mem::replace(&mut self.caret.shown, shown) != shown {
        self.screen.mark_cursor_changed();
    }

    if self.screen.all_changed() {
        // Also the margins, which have the colour of the background.
        damage(Rect::new(0, 0, i32::MAX, i32::MAX));
    } else {
        for (row, columns) in self.screen.changes() {
            damage(self.area(row, columns));
        }
    }
    self.screen.clear_changes();
    match (sync, blink) {
        (Some(sync), Some(blink)) => Some(sync.min(blink)),
        (sync, blink) => sync.or(blink),
    }
}

fn send(&mut self, title: bool) {
    if !self.answers.is_empty() {
        if let Some(input) = &self.input {
            input.write(&self.answers);
        }
        self.answers.clear();
    }
    if title {
        // The window takes the title when it wakes up.
        self.waker.wake();
    }
}
```

Antes de cada fotograma, `Frames` chama a `update` ([`Scene`](#scene)). Paso a paso:

1. **Baixo o *lock*:**
   - **O fin dunha actualización sincronizada.** Se o programa deixou unha aberta (`ESC [ ? 2026 h`) e pasou a súa hora límite, móstrase tal como está ([`end_late_sync`](#as-actualizacións-sincronizadas-end_late_sync)): `vte` garda a saída e coñece a hora, pero non ten temporizador ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control)). O que gardaba pode traer respostas e un título novo. Vai antes da copia, para que este fotograma xa o mostre. Devolve a hora límite da que siga aberta, se a hai.
   - **A copia.** [`copy_damage`](meiga-core.md#cratesmeiga-coresrctermmodrs) copia á vista as filas que cambiaron, o cursor e a paleta, e marca as celas que agora son distintas.
   - **A marca da pantalla** quítase: a xanela xa copiou, e a seguinte saída voltará espertala.
2. **Fóra do *lock*,** `send` envía as respostas ao programa coa copia de `Input` e, se cambiou o título, esperta a xanela: o título cólleo [`App::wake`](#wake-o-aviso-do-fío-lector), coma cando o cambia o fío lector, porque a vista non ten a xanela. A marca do título xa quedou posta, así que o fío lector non envía outro aviso por el. Se o programa non se lanzou, non hai `Input`, e o búfer baléirase igual.
3. **A actividade.** Se cambiou algo na pantalla, o pestanexo empeza de novo, visíbel: mentres o programa escribe, o cursor non pestanexa, e deixa de facelo cando pasa o tempo de espera sen cambios ([`Caret`](#o-pestanexo-caret)).
4. **O pestanexo.** `phase` di se o cursor se ve agora e cando cambia iso. Se cambiou desde o último fotograma, as súas celas márcanse como cambiadas. Vai despois da actividade: se non, cada cambio do pestanexo contaría como actividade, e o cursor non deixaría nunca de pestanexar.
5. **As zonas.** Se cambiaron todas as celas, por exemplo porque o programa cambiou a paleta, cambia a xanela enteira, coas marxes, que teñen a cor do fondo: un rectángulo enorme, que `Frames` recorta ao *framebuffer*. Se non, as columnas cambiadas de cada fila, en píxeles (`area`).
6. Os cambios da copia esquécense (`clear_changes`): xa están nas zonas.
7. **Cando outra vez:** o momento máis próximo entre a hora límite da actualización sincronizada e o seguinte cambio do pestanexo, se hai algún. `Frames` programa para el o temporizador dos fotogramas ([`wait_for_scene`](#a-escena-e-o-temporizador-update-e-wait_for_scene)). En repouso, sen saída e co cursor quieto, non hai ningún, e a xanela non esperta.

### Coa xanela minimizada: `update_hidden`

```rust
fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
    // Only the synchronized update: the screen is copied when the
    // window is restored, and the cursor does not blink in a window
    // that is not shown.
    let (sync, title) = lock(&self.shared).end_late_sync(now, &mut self.answers);
    self.send(title);
    sync
}
```

Coa xanela minimizada, `Frames` non pon ao día a vista nin debuxa nada, pero chama a `update_hidden` cando a vista pode ter algo que non pode esperar: ao minimizar a xanela, cando o fío lector a esperta e cando vence o temporizador ([`hidden`](#coa-xanela-minimizada-hidden)).

- **Só a actualización sincronizada.** Remátaa se chegou a súa hora, envía as respostas e esperta a xanela polo título, coma `update`, co mesmo código (`end_late_sync` e `send`); e devolve a hora límite da que siga aberta, para que `Frames` programe para ela o temporizador. Así, o programa recibe a resposta ás preguntas que fixo dentro dela, aínda que a xanela non se vexa ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **Non copia a pantalla:** cópiase unha vez ao restaurar a xanela, con todo o que cambiou mentres tanto. Copiala agora sería traballo para nada, e quitaría a marca da pantalla, así que a seguinte saída voltaría espertar a xanela.
- **Nin calcula o pestanexo:** un cursor non pestanexa nunha xanela que non se ve. Así, o temporizador só serve para a hora límite, e sen ningunha actualización aberta, `update_hidden` devolve `None`, e a xanela minimizada non esperta.
- `lock(&self.shared).end_late_sync(…)`: o `MutexGuard` é temporal, e destrúese ao final da sentenza, así que o *lock* xa está solto en `send`.

### Debuxar: `draw` e `draw_row`

```rust
fn draw(&mut self, canvas: &mut Canvas<'_>) {
    let Self { font, cell, screen, caret, run, .. } = self;
    let palette = screen.palette();
    let size = screen.size();
    let (width, height) = cell_size(*cell);
    let grid = Rect::new(0, 0, i32::from(size.columns) * width, i32::from(size.rows) * height);
    // The margins: the window can be larger than the grid.
    let clip = canvas.clip();
    canvas.fill(Rect::new(grid.right, clip.top, clip.right, clip.bottom), palette.background);
    canvas.fill(Rect::new(clip.left, grid.bottom, grid.right, clip.bottom), palette.background);

    let visible = clip.intersection(grid);
    if visible.is_empty() {
        return;
    }
    // `visible` is inside the grid, so the divisions are not negative.
    let columns = (visible.left / width) as usize..((visible.right - 1) / width + 1) as usize;
    for row in visible.top / height..=(visible.bottom - 1) / height {
        let cells = screen.row(row as u16);
        draw_row(canvas, font, run, cells, columns.clone(), row * height, palette);
    }
    draw_cursor(canvas, font, screen, caret, *cell);
}
```

- **Os préstamos.** `let Self { font, cell, screen, caret, run, .. } = self;` separa os campos en referencias distintas: así `draw_row` pode cambiar a fonte (a súa caché de glifos) e o búfer `run` mentres le a pantalla e a súa paleta. Con métodos de `self`, Rust non o permitiría, porque cada chamada colle `self` enteiro.
- **As marxes.** A xanela pode ser maior ca a grella: á dereita e debaixo dela, píntase a cor do fondo da paleta. Son dous rectángulos sen parte común; `fill` recórtaos ao lenzo, e un rectángulo baleiro non pinta nada.
- **As filas e as columnas** son só as que toca o recorte: desde a do seu primeiro píxel ata a do último, `right - 1` e `bottom - 1`, porque o recorte non inclúe os seus bordos dereito e inferior. Como `visible` está dentro da grella, as divisións non son negativas, e os `as` non perden nada. `Range` non é `Copy`, e cada fila recibe unha copia (`clone`).
- **O cursor** debúxase ao final, por riba das súas celas, que se acaban de debuxar: cando o cursor cambia, as súas celas están sempre entre as cambiadas (ver [`update`](#poñer-ao-día-update)).
- `cell_size` dá o tamaño da cela en coordenadas (`i32`): unha cela son unhas decenas de píxeles.

```rust
fn draw_row(
    canvas: &mut Canvas<'_>,
    font: &mut Font,
    run: &mut Vec<CellChar>,
    cells: &[Cell],
    columns: Range<usize>,
    top: i32,
    palette: &Palette,
) {
    let (width, height) = cell_size(font.cell());
    let end = columns.end.min(cells.len());
    let mut start = columns.start;
    while start < end {
        let first = &cells[start];
        let same = |cell: &Cell| cell.fg == first.fg && cell.bg == first.bg && cell.flags == first.flags;
        let stop = cells[start + 1..end]
            .iter()
            .position(|cell| !same(cell))
            .map_or(end, |count| start + 1 + count);
        let style = Style::of(first, palette);
        run.clear();
        if style.hidden {
            run.resize(stop - start, CellChar::Blank);
        } else {
            run.extend(cells[start..stop].iter().map(|cell| cell.char));
        }
        let left = start as i32 * width;
        font.draw(canvas, Point::new(left, top), run, style.font, style.text, style.background);
        if !style.hidden {
            let area = Rect::new(left, top, stop as i32 * width, top + height);
            draw_lines(canvas, first.flags, area, font.decorations(), style.text, font.cell());
        }
        start = stop;
    }
}
```

- **Por tramos.** As celas seguidas coas mesmas cores e os mesmos estilos (`same`) debúxanse cunha soa chamada a [`Font::draw`](meiga-render.md#cratesmeiga-rendersrctextrs), que pinta primeiro o fondo de todas e despois a tinta dos seus glifos, só das que están dentro do recorte: unha chamada por tramo, e non por cela.
- `position` dá cantas celas despois da primeira son iguais ca ela; se ningunha é distinta, o tramo chega ao final (`map_or(end, …)`).
- **O oculto** debuxa celas en branco: só o fondo, e sen liñas.
- `run` reutilízase de tramo en tramo e de fotograma en fotograma: `clear` baléirao sen liberar a memoria.
- `end` recorta as columnas ás da fila: unha fila fóra da pantalla non ten celas, e non debuxa nada. As columnas dunha fila caben nun `u16`, así que os seus píxeles caben nun `i32`.

### `Style`

```rust
struct Style {
    font: FontStyle,
    text: Rgb,
    background: Rgb,
    /// The text is not drawn (SGR 8), only the background.
    hidden: bool,
}

impl Style {
    fn of(cell: &Cell, palette: &Palette) -> Self {
        let flags = cell.flags;
        let mut text = cell.fg.rgb(palette, palette.foreground);
        let mut background = cell.bg.rgb(palette, palette.background);
        if flags.contains(CellFlags::INVERSE) {
            mem::swap(&mut text, &mut background);
        }
        if flags.contains(CellFlags::DIM) {
            text = text.mix(background, DIM);
        }
        Self {
            font: FontStyle {
                bold: flags.contains(CellFlags::BOLD),
                italic: flags.contains(CellFlags::ITALIC),
            },
            text,
            background,
            hidden: flags.contains(CellFlags::HIDDEN),
        }
    }
}
```

Como se debuxa unha cela ([Tecnoloxías › A paleta e os estilos](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)):

- **As cores.** [`Color::rgb`](meiga-core.md#cratesmeiga-coresrctermscreenrs) converte a cor da cela coa paleta da copia da pantalla: un número, a súa cor na paleta; unha cor *truecolor*, tal cal; e a cor por defecto, a do texto ou a do fondo. As celas gardan o número da cor, e non a cor, así que, se o programa cambia a paleta, o texto que xa está na pantalla cambia tamén, coma no xterm.
- **O inverso** troca as dúas cores, e **o tenue** achega o texto á cor do fondo, despois do inverso.
- **A negra e a cursiva** só escollen a fonte ([`FontStyle`](meiga-render.md#cratesmeiga-rendersrctextrs)): a cor non cambia, coma no Alacritty e no Warp, en vez de usar a cor brillante, coma o xterm ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **O oculto** (`hidden`) só debuxa o fondo.

### As liñas: `draw_lines`

```rust
fn draw_lines(canvas: &mut Canvas<'_>, flags: CellFlags, area: Rect, decorations: Decorations, color: Rgb, cell: Size) {
    let Decorations { underline, strikeout } = decorations;
    // A cell is a few dozen pixels.
    let thickness = underline.thickness as i32;
    let line = |canvas: &mut Canvas<'_>, top: u32, height: u32| {
        let top = area.top + top as i32;
        canvas.fill(Rect::new(area.left, top, area.right, top + height as i32), color);
    };
    if flags.contains(CellFlags::UNDERLINE) {
        line(canvas, underline.top, underline.thickness);
    }
    if flags.contains(CellFlags::DOUBLE_UNDERLINE) {
        // Two lines with a gap as thick as them, higher if they do not fit.
        let top = underline.top.min(cell.height.saturating_sub(3 * underline.thickness));
        line(canvas, top, underline.thickness);
        line(canvas, top + 2 * underline.thickness, underline.thickness);
    }
    let top = area.top + underline.top as i32;
    if flags.contains(CellFlags::DOTTED_UNDERLINE) {
        // Dots as long as the line is thick, with gaps as long, at the same
        // places of the row however much of it a frame draws: from the left
        // of the window, not of the area.
        let period = 2 * thickness;
        let first = area.left - area.left.rem_euclid(period);
        for left in (first..area.right).step_by(period as usize) {
            let dot = Rect::new(left.max(area.left), top, (left + thickness).min(area.right), top + thickness);
            canvas.fill(dot, color);
        }
    }
    if flags.contains(CellFlags::DASHED_UNDERLINE) {
        // A dash in the middle of each cell, two thirds of its width.
        let width = cell.width as i32;
        let dash = (width * 2 / 3).max(1);
        for left in (area.left..area.right).step_by(width.max(1) as usize) {
            let left = left + (width - dash) / 2;
            canvas.fill(Rect::new(left, top, left + dash, top + thickness), color);
        }
    }
    if flags.contains(CellFlags::UNDERCURL) {
        // A zigzag with a period of a cell, as high as the space below the
        // underline allows.
        let period = (cell.width as i32).max(4);
        let height = (cell.height as i32 / 8).max(thickness);
        let top = top.min(area.top + cell.height as i32 - height - thickness);
        let half = period / 2;
        for x in area.left..area.right {
            let phase = x.rem_euclid(period);
            let rise = if phase < half { phase } else { period - phase };
            let y = top + rise * height / half;
            canvas.fill(Rect::new(x, y, x + 1, y + thickness), color);
        }
    }
    if flags.contains(CellFlags::STRIKEOUT) {
        line(canvas, strikeout.top, strikeout.thickness);
    }
}
```

As liñas das celas dun tramo, por riba do texto ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control)):

| Estilo | Como se debuxa |
|---|---|
| Subliñado (`4`) | Unha liña onde di a fonte, co seu grosor. |
| Dobre (`4:2`) | Dúas liñas co grosor do subliñado, cun oco igual entre elas. Se non caben debaixo, soben, para quedar dentro da cela. |
| Ondulado (`4:3`) | Un zigzag dun píxel de ancho en cada columna, cun período dunha cela e unha altura dun oitavo dela, polo menos o grosor. Sobe se non cabe debaixo do subliñado. |
| De puntos (`4:4`) | Cadrados do grosor do subliñado, separados por outro tanto, contados desde o bordo esquerdo da xanela. |
| De trazos (`4:5`) | Un trazo no medio de cada cela, de dous terzos do seu ancho, así que entre dúas celas queda un oco. |
| Riscado (`9`) | Unha liña onde di a fonte, co grosor do riscado. |

- **Onde e de que grosor** dío a fonte ([`Decorations`](meiga-render.md#cratesmeiga-rendersrcdwriters)): a distancia en píxeles desde o alto da cela ata o bordo superior de cada liña, e o seu grosor ([Tecnoloxías › DirectWrite](../tecnoloxias.md#27-directwrite-fontes-e-glifos)).
- **A cor** é a do texto, despois do inverso e do tenue. Aínda non hai cor propia para o subliñado (`SGR 58`, [ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **Os puntos e o zigzag** cóntanse desde o bordo esquerdo da xanela, e non desde o da zona que se debuxa: así seguen no mesmo sitio dun tramo ao seguinte, e cando un fotograma debuxa de novo só unha parte da liña. Os puntos empezaban antes no bordo da zona, e ao 100 %, con celas de 9 píxeles e unha liña de 1, hai un punto cada 2 píxeles: un fotograma que empezaba nunha columna impar, como o das celas do cursor en cada pestanexo, debuxaba os seus puntos desprazados un píxel respecto dos das celas do lado.
- `first` é o múltiplo do período máis próximo ao bordo esquerdo da zona sen pasalo: alí empeza un punto, no bordo ou xusto antes. Cada punto recórtase á zona (`max` e `min`): o primeiro pode empezar antes dela e o último acabar despois, e fóra dela os píxeles son doutras celas, que poden ter outro estilo. Un punto que queda enteiro fóra é, recortado, un rectángulo baleiro, que non pinta nada. `rem_euclid` dá un resto que nunca é negativo.
- `line` é unha *closure* que recibe o lenzo como parámetro, en vez de collelo de fóra: así non o garda prestado, e o resto da función pode usalo entre unha chamada e outra.

### O cursor: `draw_cursor`

```rust
fn draw_cursor(canvas: &mut Canvas<'_>, font: &mut Font, screen: &Screen, caret: &Caret, cell: Size) {
    let cursor = screen.cursor();
    if cursor.shape == CursorShape::Hidden || !caret.shown {
        return;
    }
    let cells = screen.row(cursor.row);
    let column = usize::from(cursor.column);
    let Some(under) = cells.get(column) else {
        return;
    };
    // Over a wide character, it takes its two cells.
    let wide = matches!(under.char, CellChar::WideLeft(_));
    let (width, height) = cell_size(cell);
    let (x, y) = (i32::from(cursor.column) * width, i32::from(cursor.row) * height);
    let area = Rect::new(x, y, x + if wide { 2 * width } else { width }, y + height);
    if !area.intersects(canvas.clip()) {
        return;
    }
    let palette = screen.palette();
    let color = palette.cursor;
    let thickness = caret.thickness as i32;
    let shape = if caret.focused { cursor.shape } else { CursorShape::HollowBlock };
    match shape {
        CursorShape::Beam => canvas.fill(Rect::new(x, y, x + thickness, y + height), color),
        CursorShape::Underline => canvas.fill(
            Rect::new(x, area.bottom - thickness, area.right, area.bottom),
            color,
        ),
        CursorShape::HollowBlock => canvas.outline(area, caret.thickness, color),
        CursorShape::Block => {
            // The cells under it, with the colour of their background on the
            // one of the cursor.
            let style = Style::of(under, palette);
            let count = if wide { 2 } else { 1 };
            let mut chars = [CellChar::Blank; 2];
            if !style.hidden {
                for (char, cell) in chars.iter_mut().zip(&cells[column..]) {
                    *char = cell.char;
                }
            }
            font.draw(canvas, Point::new(x, y), &chars[..count], style.font, style.background, color);
        }
        CursorShape::Hidden => {}
    }
}
```

O cursor, por riba das celas ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)):

| Forma | Como se debuxa |
|---|---|
| Barra (`Beam`) | Unha barra vertical á esquerda da cela, co grosor do cursor. É a forma por defecto, coma no Windows Terminal. |
| Subliñado (`Underline`) | Unha liña no fondo da cela, co grosor do cursor. |
| Bloque (`Block`) | A cela enteira da cor do cursor, co seu carácter da cor do fondo da cela: as cores ao revés. |
| Baleiro (`HollowBlock`) | O bordo da cela, por dentro ([`Canvas::outline`](meiga-render.md#cratesmeiga-rendersrccanvasrs)), co grosor do cursor. É a forma de calquera cursor sen o foco, e non pestanexa. |
| Oculto | Nada. |

- Non se debuxa se o programa o ocultou (`ESC [ ? 25 l`), nin na metade oculta do pestanexo (`shown`).
- **Sobre un carácter ancho**, na súa metade esquerda (`WideLeft`), ocupa as súas dúas celas ([Tecnoloxías › O ancho dos caracteres](../tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)).
- Se o cursor non toca o recorte, este fotograma debuxa outra zona, e non hai nada que facer.
- **O bloque** debuxa outra vez o carácter de debaixo, ou os dous dun carácter ancho, co seu estilo e coa cor do fondo da cela sobre a do cursor. Se a cela está oculta, só a cor do cursor. `chars` é un *array* de dous, sen reservar memoria.
- **A cor** é a do cursor na paleta: a do texto, salvo que o programa a cambie (`OSC 12`).
- **O grosor** é o do cursor de Windows, á escala da xanela ([`thickness`](#o-pestanexo-caret)).

### O pestanexo: `Caret`

```rust
fn thickness(settings: CaretSettings, dpi: u32) -> u32 {
    let scaled = (settings.width.saturating_mul(dpi) + BASE_DPI / 2) / BASE_DPI;
    scaled.max(1)
}

struct Caret {
    settings: CaretSettings,
    dpi: u32,
    thickness: u32,
    focused: bool,
    since: Instant,
    shown: bool,
}

impl Caret {
    fn phase(&self, cursor: Cursor, now: Instant) -> (bool, Option<Instant>) {
        let blinks = self.focused && cursor.blinking && cursor.shape != CursorShape::Hidden;
        let Some(period) = self.settings.blink.filter(|_| blinks) else {
            return (true, None);
        };
        let Some(end) = self.since.checked_add(self.settings.timeout) else {
            return (true, None);
        };
        if now >= end || period.is_zero() {
            return (true, None);
        }
        let elapsed = now.saturating_duration_since(self.since);
        let halves = u32::try_from(elapsed.as_nanos() / period.as_nanos()).unwrap_or(u32::MAX);
        let next = period
            .checked_mul(halves.saturating_add(1))
            .and_then(|offset| self.since.checked_add(offset))
            .map_or(end, |next| next.min(end));
        (halves.is_multiple_of(2), Some(next))
    }
}
```

Como se ve e como pestanexa o cursor ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)).

| Campo | Que garda |
|---|---|
| `settings` | Os axustes do cursor de Windows ([`CaretSettings`](meiga-win.md#cratesmeiga-winsrcwindowrs)): o tempo de cada metade do pestanexo, ou `None` se Windows non o permite; o tempo de espera, despois do que deixa de pestanexar; e o grosor. Lense ao crear a vista e cada vez que a xanela recupera o foco (`read_settings`). |
| `dpi` e `thickness` | O DPI da xanela, e o grosor do cursor a ese DPI. |
| `focused` | Se a xanela ten o foco. |
| `since` | Cando empezou a pestanexar: na última actividade, un cambio na pantalla ou recuperar o foco. |
| `shown` | Se o cursor se ve agora. O fotograma no que cambia marca as súas celas como cambiadas ([`update`](#poñer-ao-día-update)). |

- **`thickness`**: o grosor de Windows, en píxeles ao 100 %, á escala da xanela: `ancho × DPI / 96`, redondeado (sumar a metade de 96 antes de dividir), e polo menos un píxel. Co grosor por defecto, 1, ao 150 % (144 DPI) son 2 píxeles.

**`phase`** di se o cursor se ve no instante `now`, e cando cambia iso, mentres pestanexa:

1. Só pestanexa se a xanela ten o foco, se o programa non pediu un cursor fixo (`blinking`), se non está oculto e se Windows o permite (`blink`). Se non, vese sempre, e non hai ningún cambio que esperar: `(true, None)`. `Option::filter` só deixa o tempo do pestanexo se pestanexa.
2. Deixa de pestanexar, visíbel, en `since + timeout`. `checked_add` dá `None` se a suma non cabe nun `Instant`, e entón tampouco pestanexa. Un período de cero, que Windows non dá pero o tipo permite, tampouco.
3. As metades que pasaron desde `since`: nas pares vese, e nas impares non. O seguinte cambio é ao final da metade actual, pero nunca despois do fin: se o tempo de espera remata no medio dunha metade oculta, o cursor aparece nese momento, e queda.
4. As metades cóntanse dividindo os nanosegundos. Cos tempos por defecto de Windows, 530 ms por metade e 5 000 de espera, son menos de dez; e, como os tempos de Windows son milisegundos enteiros, as metades antes do fin caben sempre nun `u32`: `unwrap_or(u32::MAX)` só cobre un caso que non se dá.

Así, **en repouso a xanela non esperta**: despois do tempo de espera, `phase` non pide ningún momento, e o temporizador dos fotogramas queda parado. E se Windows desactiva o pestanexo, como pode facer o Escritorio remoto, non pestanexa nunca ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).

**Probas.** Catro: o cursor pestanexa ata o tempo de espera e queda visíbel (con metades de 500 ms e 5 000 ms de espera, visíbel ata os 500, oculto ata os 1 000…, e o último cambio, aos 5 000, déixao visíbel para sempre); non pestanexa se algo non o permite: un cursor fixo, un oculto, Windows, como por Escritorio remoto, ou a falta do foco; os puntos dun subliñado de puntos quedan no mesmo sitio cando un fotograma debuxa só parte da liña (en catro celas de 9 × 19 píxeles, coma ao 100 %, cun grosor de 1 e de 2: a fila enteira ten un punto ao principio de cada período, contado desde o bordo esquerdo, e un fotograma que só debuxa a segunda e a terceira cela, que empezan nun píxel impar, debuxa nelas os mesmos puntos, e nada fóra delas); e os estilos dan as cores do texto e do fondo: as por defecto, as dunha cor da paleta en inverso e negra, e as dunha cor *truecolor* tenue e oculta sobre outra da paleta.

## `crates/meiga/src/frames.rs`

O **fluxo de fotogramas**: que cambiou, cando debuxalo e como chega á xanela. Aplica tres principios do proxecto: debuxar só o que cambia, como moito un fotograma por intervalo, e non debuxar o que non se ve ([ADR 0005](../adr/0005-fios-e-fotogramas.md), [ADR 0009](../adr/0009-xanela-que-non-se-ve.md)). Desde a subfase 1.6, cada fotograma pon primeiro ao día a escena, e a escena pode pedir un fotograma para un momento concreto, como o seguinte cambio do pestanexo do cursor ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Coa xanela minimizada, sen fotogramas, a escena só fai o que non pode esperar, e o temporizador dos fotogramas queda só para iso.

### `Scene`

```rust
pub trait Scene {
    /// Brings the scene up to date before a frame, at `now`: gives `damage`
    /// the areas that changed. Returns when it has to be brought up to date
    /// again, even if nothing else changes, if it has to: for example, when
    /// the cursor blinks.
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        let _ = (now, damage);
        None
    }

    /// While the window is minimized nothing is drawn, and the scene is not
    /// brought up to date: does at `now` only what cannot wait until the
    /// window is restored, such as answering the program. Returns when it has
    /// to do it again, if it has to.
    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        let _ = now;
        None
    }

    /// Draws everything that is inside the clip rectangle of `canvas`. It
    /// takes `&mut self` because drawing can fill caches, such as the one of
    /// the glyphs.
    fn draw(&mut self, canvas: &mut Canvas<'_>);
}
```

O que mostra a xanela. En cada fotograma, `Frames` chama a `update` e a `draw`; coa xanela minimizada, só a `update_hidden`.

**`update`**, desde a subfase 1.6, pon ao día a escena no instante `now`, e dálle a `damage` as zonas que cambiaron nela. O terminal copia eiquí o que cambiou no terminal da sesión ([`TerminalView::update`](#poñer-ao-día-update)). Devolve cando hai que poñela ao día outra vez, aínda que non cambie nada máis: a hora límite dunha actualización sincronizada, ou o seguinte cambio do pestanexo. Por defecto non fai nada nin pide ningún momento: é o caso dos modos de medición, que marcan as súas zonas antes de cada fotograma.

**`update_hidden`**, coa xanela minimizada, fai só o que non pode esperar a que se restaure, sen poñer ao día a escena, e devolve cando hai que facelo outra vez. O terminal remata eiquí á súa hora a actualización sincronizada que o programa deixou aberta, para que reciba as respostas ás súas preguntas ([`TerminalView::update_hidden`](#coa-xanela-minimizada-update_hidden)). Por defecto, nada.

**`draw`** debuxa todo o que cae dentro do recorte do lenzo, sen preocuparse de que cambiou: `Frames` pásalle un lenzo recortado a cada zona que cambiou ([`Canvas::clipped`](meiga-render.md#canvas)), e así só se tocan eses píxeles.

- **Por que no fotograma.** A escena ponse ao día cando se vai debuxar, e non cada vez que chega saída: moitos bloques de saída nun intervalo fan unha soa copia, e coa xanela minimizada, que non debuxa, non se copia nada.
- **Por que outro método para a xanela minimizada.** `update` pon ao día a escena enteira, e coa xanela minimizada iso sería traballo para nada: o terminal copiaría a pantalla en cada aviso, e, coa marca da pantalla quitada, a saída espertaría a xanela outra vez ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). `update_hidden` deixa á vista separar o que pode esperar do que non.
- `&mut dyn FnMut(Rect)`: unha *closure* que se pode chamar moitas veces e que cambia o que captura: a de `Frames` engade cada zona ao seu `damage`. Vai como `dyn`, e non como un tipo xenérico, porque `Frames` recibe a escena como `&mut dyn Scene`, e un *trait* con métodos xenéricos non se pode usar así.
- `let _ = (now, damage);` e `let _ = now;` usan os parámetros sen facer nada con eles: o compilador non avisa de que sobran, e os seus nomes seguen explicando o *trait*.
- **`&mut self`.** Desde a subfase 1.3, debuxar pode cambiar a escena: o texto garda na caché os glifos que rasteriza a primeira vez ([`Font`](meiga-render.md#font-e-blending)). Por iso `Frames` recibe a escena como `&mut dyn Scene` en todos os seus métodos.

### `Frames`

```rust
pub struct Frames {
    surface: Surface,
    damage: Damage,
    pacer: FramePacer,
    changed: bool,
    due: Option<Instant>,
    minimized: bool,
    live_resize: bool,
    marks: Option<Vec<Rect>>,
    trace: Option<Trace>,
}
```

| Campo | Que garda |
|---|---|
| `surface` | O *framebuffer* ([`Surface`](meiga-render.md#fit-e-surface)). |
| `damage` | As zonas que cambiaron e aínda non se presentaron ([`Damage`](meiga-core.md#cratesmeiga-coresrcdamagers)). |
| `pacer` | Cando foi o último fotograma e se hai outro programado ([`FramePacer`](meiga-core.md#cratesmeiga-coresrcpacingrs)). |
| `changed` | Se a escena cambiou desde a última vez que se puxo ao día. As súas zonas só se saben ao poñela ao día, así que, mentres tanto, fai falta un fotograma aínda que `damage` estea baleiro. |
| `due` | Cando hai que poñer ao día a escena outra vez, como ela pediu, ou `None`. |
| `minimized` | Se a xanela está minimizada: entón non se debuxa nada, e a escena só fai o que non pode esperar ([`hidden`](#coa-xanela-minimizada-hidden)). |
| `live_resize` | Se o usuario está movendo ou redimensionando a xanela. |
| `marks` | Coas marcas activadas, as zonas que presentou o último fotograma, que levan un bordo na xanela. `None` coas marcas desactivadas, como empezan. |
| `trace` | O rexistro dos fotogramas, se se pediu. |

**A regra que mantén:** a xanela mostra o *framebuffer*, agás nas zonas de `damage`, que cambiaron e que o seguinte fotograma compón e presenta. O que cambiou na escena e aínda non se copiou (`changed`) chega a `damage` ao principio dese fotograma.

### Crear: `Frames::new`

```rust
pub fn new(window: &Window, scene: &mut dyn Scene, trace: Option<Trace>) -> Result<Self, HandlerError> {
    let mut frames = Self {
        surface: Surface::new(window.client_size()?)?,
        damage: Damage::new(),
        pacer: FramePacer::new(),
        changed: false,
        due: None,
        minimized: false,
        live_resize: false,
        marks: None,
        trace,
    };
    frames.compose_all(scene)?;
    // The first paint of the window copies all of it.
    frames.damage.clear();
    frames.wait_for_scene(window)?;
    Ok(frames)
}
```

- Recibe a xanela: dela saen o tamaño do *framebuffer* (`client_size`) e o temporizador.
- `compose_all` pon ao día a escena e compón o *framebuffer* enteiro. No terminal, esa primeira copia leva todo o que haxa: a pantalla baleira, ou o que xa escribise o programa, que se lanzou antes ([`app.rs`](#appnew-e-terminal)).
- `damage.clear()`: a primeira copia deu a xanela enteira como cambiada, pero non hai nada que presentar: o primeiro `WM_PAINT` copia a xanela enteira desde o *framebuffer*.
- `wait_for_scene` programa o temporizador se a escena pediu un momento ([máis abaixo](#a-escena-e-o-temporizador-update-e-wait_for_scene)).
- As marcas empezan desactivadas, e a tecla P actívaas ([`app.rs`](#os-eventos)).

### Pedir un fotograma

```rust
pub fn damage(&mut self, area: Rect) {
    let area = area.intersection(Rect::from_size(self.surface.size()));
    if !area.is_empty() {
        self.damage.add(area);
    }
}

pub fn changed(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    self.changed = true;
    self.request(window, scene)
}

pub fn request(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    if self.minimized {
        return self.hidden(window, scene);
    }
    if self.damage.is_empty() && !self.changed {
        return self.wait_for_scene(window);
    }
    match self.pacer.request(Instant::now(), interval()) {
        Pace::Now => self.draw(window, scene, Cause::Now),
        Pace::After(delay) => Ok(window.start_frame_timer(delay)?),
        Pace::Scheduled => Ok(()),
    }
}

pub fn timer(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    if self.pacer.is_scheduled() {
        return self.draw(window, scene, Cause::Timer);
    }
    self.due = None;
    self.changed(window, scene)
}
```

- `damage` anota unha zona que cambiou, recortada ao *framebuffer*: o que queda fóra non se debuxa. Úsana os modos de medición; o terminal dá as súas zonas en `update`.
- **`changed`**: a escena cambiou, pero as súas zonas só se saben ao poñela ao día. Marca `changed`, para que haxa fotograma aínda sen zonas, e pídeo coma `request`. Chámano o aviso do fío lector e o cambio do foco ([`app.rs`](#os-eventos)), e `timer`.
- **`request`** decide cando se debuxa o que cambiou:
  - coa xanela minimizada, nunca: os cambios esperan en `damage` e na escena, e `changed` segue marcado. A escena só fai o que non pode esperar ([`hidden`](#coa-xanela-minimizada-hidden));
  - se non cambiou nada, só queda programar o temporizador para o momento que pediu a escena (`wait_for_scene`), que pode ser novo: ao cambiar o tamaño da xanela, `compose_all` acaba de poñela ao día;
  - se non, o que diga o *pacer*: agora, cun temporizador ou xa está programado.
- **`timer`**: venceu o temporizador. Se o *pacer* tiña un fotograma programado, debúxase. Se non, o temporizador era o da escena: chegou o momento que pediu, e iso é un cambio coma calquera outro, que pasa polo *pacer*. Así, un cambio do pestanexo xusto despois dun fotograma agarda ao final do intervalo. No rexistro, se se debuxa no momento, ese fotograma aparece como `now`. Coa xanela minimizada non hai fotogramas programados, e o temporizador só pode ser o da escena: `changed` leva a `request`, e de aí a `hidden`, que lle deixa facer o que non pode esperar.
- `interval()` dá o intervalo de `max_fps = "auto"`: 30 fps se a sesión é remota e 60 en local. Pregúntase en cada fotograma, porque a sesión pode pasar de local a remota mentres meiga está aberto.

```rust
pub fn schedule(&mut self, window: &Window) -> Result<(), HandlerError> {
    if self.minimized {
        return Ok(());
    }
    let delay = match self.pacer.request(Instant::now(), interval()) {
        Pace::Now => Duration::ZERO,
        Pace::After(delay) => delay,
        Pace::Scheduled => return Ok(()),
    };
    Ok(window.start_frame_timer(delay)?)
}
```

Para os modos de medición: pide un fotograma para o primeiro momento que permita o *pacer*, aínda que non cambiase nada. A diferenza con `request` é que nunca debuxa no momento, sempre co temporizador, para que o bucle de mensaxes trate as mensaxes entre un fotograma e o seguinte.

### `draw`

```rust
fn draw(&mut self, window: &Window, scene: &mut dyn Scene, cause: Cause) -> Result<(), HandlerError> {
    window.stop_frame_timer();
    if self.minimized {
        self.pacer.cancel();
        return Ok(());
    }
    let start = Instant::now();
    self.update(scene, start);
    if self.damage.is_empty() {
        self.pacer.cancel();
        return self.wait_for_scene(window);
    }

    let mut canvas = self.surface.canvas();
    for &area in self.damage.rects() {
        scene.draw(&mut canvas.clipped(area));
    }
    let composed = Instant::now();

    let handle = window.handle();
    let erased = match &self.marks {
        Some(marks) => erase(&self.surface, handle, marks),
        None => Ok(()),
    };
    let presented = self.surface.present(handle, self.damage.rects());
    let present = composed.elapsed();
    match erased.and(presented) {
        Ok(()) => {
            if let Some(marks) = &mut self.marks {
                marks.clear();
                marks.extend_from_slice(self.damage.rects());
                let _ = self.surface.mark(handle, marks, MARK_COLOR, MARK_THICKNESS);
            }
        }
        Err(_) => window.invalidate(None),
    }

    log(&mut self.trace, &Record { kind: Kind::Frame, cause: Some(cause), … })?;
    self.damage.clear();
    self.pacer.drawn(start);
    self.wait_for_scene(window)
}
```

Un fotograma, paso a paso:

1. **O temporizador** detense. Se había un fotograma programado, é este: normalmente chega co temporizador, pero tamén pode chegar antes, cando o temporizador vai con atraso e chega outro cambio (ver [`FramePacer`](meiga-core.md#cratesmeiga-coresrcpacingrs)). Coa xanela minimizada, non se debuxa nada, nin se pon ao día a escena.
2. **Poñer ao día a escena** (`update`): as zonas que cambiaron nela súmanse ás de `damage`. Se ao final non hai ningunha, por exemplo porque a saída non cambiou ningunha cela, non hai fotograma: cancélase o do *pacer*, e só queda esperar polo momento da escena.
3. **Compoñer.** Para cada zona que cambiou, a escena debúxase nun lenzo recortado a ela.
4. **Presentar.** Coas marcas activadas, primeiro bórranse as do fotograma anterior: preséntanse de novo, desde o *framebuffer*, os seus bordos. Despois preséntanse as zonas que cambiaron.
5. **As marcas novas**, se están activadas: o bordo das zonas que se acaban de presentar, en magenta, debuxado só na xanela.
6. **Se a presentación falla**, por exemplo porque a sesión está bloqueada ou desconectada, pídeselle a Windows que pinte a xanela enteira cando poida (`invalidate`): ese `WM_PAINT` copiará o *framebuffer*, que xa está ao día. Non é motivo para parar meiga.
7. **O rexistro**, se se pediu: unha liña co tempo de compoñer e o de presentar. O de compoñer inclúe o de poñer ao día a escena: no terminal, a copia baixo o *lock*.
8. Esquécense as zonas, anótase o fotograma no *pacer*, co instante en que empezou, e prográmase o temporizador para o seguinte momento da escena, se o pediu (`wait_for_scene`).

- `Result::and` devolve o primeiro erro dos dous, pero as dúas presentacións fanse sempre: os argumentos avalíanse antes.
- `let _ = …` descarta o resultado a propósito: unha marca que non se debuxa non estraga o fotograma.
- `marks.clear()` e `extend_from_slice` reutilizan o mesmo `Vec`, sen reservar memoria nova en cada fotograma.

### A escena e o temporizador: `update` e `wait_for_scene`

```rust
fn update(&mut self, scene: &mut dyn Scene, now: Instant) {
    self.changed = false;
    let bounds = Rect::from_size(self.surface.size());
    let damage = &mut self.damage;
    self.due = scene.update(now, &mut |area| {
        let area = area.intersection(bounds);
        if !area.is_empty() {
            damage.add(area);
        }
    });
}

fn wait_for_scene(&mut self, window: &Window) -> Result<(), HandlerError> {
    if self.minimized || self.pacer.is_scheduled() {
        return Ok(());
    }
    match self.due {
        Some(due) => Ok(window.start_frame_timer(due.saturating_duration_since(Instant::now()))?),
        None => Ok(()),
    }
}

fn compose_all(&mut self, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    let start = Instant::now();
    self.update(scene, start);
    scene.draw(&mut self.surface.canvas());
    …
}
```

- **`update`** pon ao día a escena e garda o momento que pide (`due`). `changed` quítase, porque a escena xa está ao día. As zonas recórtanse ao *framebuffer*: o terminal dá un rectángulo enorme cando cambia todo, coas marxes ([`view.rs`](#poñer-ao-día-update)).
- **Un temporizador para todo.** O temporizador dos fotogramas serve tamén para os momentos da escena e, coa xanela minimizada, para o que non pode esperar ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). `wait_for_scene` só o programa se non hai ningún fotograma programado: se o hai, ese fotograma porá ao día a escena e calculará o seguinte momento. Programalo de novo substitúe o momento anterior ([Código › meiga-win](meiga-win.md#cratesmeiga-winsrcwindowrs)).
- `saturating_duration_since` dá cero se o momento xa pasou: o temporizador vence de inmediato.
- Coa xanela minimizada, `wait_for_scene` non fai nada: o temporizador é cousa de [`hidden`](#coa-xanela-minimizada-hidden), e só para o que non pode esperar, sen pestanexo.
- `wait_for_scene` chámase en todos os camiños que deixan o temporizador libre: despois dun fotograma, con ou sen zonas, ao crear `Frames`, en `request` cando non cambiou nada e ao restaurar a xanela.
- **`compose_all`** compón o *framebuffer* enteiro, sen presentalo, e anótao no rexistro como `compose`. Pon antes ao día a escena, para que o *framebuffer* teña o último que hai. As zonas desa posta ao día quedan en `damage`, e quen chama decide: ao crear `Frames` e ao restaurar a xanela, que se pinta enteira, esquécense; noutro caso, preséntaas o seguinte fotograma.

### Coa xanela minimizada: `hidden`

```rust
fn hidden(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    let now = Instant::now();
    match scene.update_hidden(now) {
        Some(due) => Ok(window.start_frame_timer(due.saturating_duration_since(now))?),
        None => {
            window.stop_frame_timer();
            Ok(())
        }
    }
}
```

Coa xanela minimizada, deixa que a escena faga o que non pode esperar ([`Scene::update_hidden`](#scene)), e garda o temporizador dos fotogramas só para a próxima vez que teña que facelo, sen debuxar nada.

- **Cando:** ao minimizar a xanela ([`resized`](#o-tamaño-da-xanela)) e, despois, cada vez que algo pediría un fotograma (`request`), por exemplo cando o fío lector esperta a xanela ou cando vence o temporizador (`timer` → `changed` → `request` → `hidden`). A escena fai o que toca, se toca, e di cando outra vez.
- **O temporizador** prográmase para ese momento ou, se non hai ningún, detense: coa xanela minimizada e sen ningunha actualización sincronizada aberta, nada esperta a xanela ([ADR 0009](../adr/0009-xanela-que-non-se-ve.md)). Nin se compón nin se presenta nada, e o rexistro dos fotogramas non ten ningunha liña nova.
- **Por que tamén ao minimizar.** `resized` detén o temporizador e cancela o fotograma programado, e `hidden` prográmao de novo se a escena ten algo pendente. Unha actualización sincronizada que xa estaba aberta, da que a xanela xa sabía a hora, quedaría aberta ata restaurar a xanela se o programa non escribe máis: o fío lector xa avisou dela, e non volta avisar ([`session.rs`](#as-actualizacións-sincronizadas-end_late_sync)).
- `due` e `wait_for_scene` quedan á parte: son para os fotogramas, que coa xanela minimizada non hai. Ao restaurala, `compose_all` pon ao día a escena, e o temporizador volta aos fotogramas ([máis abaixo](#o-tamaño-da-xanela)).

### O tamaño da xanela

```rust
pub fn resized(&mut self, window: &Window, size: Size, minimized: bool, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    if minimized {
        window.stop_frame_timer();
        self.pacer.cancel();
        self.minimized = true;
        return self.hidden(window, scene);
    }
    let restored = mem::take(&mut self.minimized);
    if size == self.surface.size() && !restored {
        return Ok(());
    }
    let fit = if self.live_resize { Fit::Room } else { Fit::Exact };
    self.surface.resize(size, fit)?;
    self.compose_all(scene)?;
    if restored {
        self.damage.clear();
        if let Some(marks) = &mut self.marks {
            marks.clear();
        }
        window.invalidate(None);
        // The frame timer may still be set for what the scene had to do
        // while the window was minimized: now it is for the frames.
        window.stop_frame_timer();
        self.wait_for_scene(window)
    } else {
        self.request(window, scene)
    }
}
```

- **Minimizada:** cancélase o fotograma programado, ou o momento da escena, e non se debuxa nada máis ([ADR 0009](../adr/0009-xanela-que-non-se-ve.md)). Os cambios seguen acumulándose en `damage` e na escena, que non se pon ao día: o terminal non copia nada, e o fío lector non volta espertar a xanela pola pantalla ([`session.rs`](#shared-e-lock)). Só queda o que non pode esperar ([`hidden`](#coa-xanela-minimizada-hidden)): se o programa ten aberta unha actualización sincronizada, o temporizador prográmase de novo para a súa hora límite.
- `mem::take` devolve o valor e deixa o valor por defecto (`false`): `restored` é se estaba minimizada.
- **Cun tamaño novo**, o *framebuffer* adáptase: con marxe (`Room`) mentres o usuario redimensiona co rato, e exacto (`Exact`) noutro caso, como ao maximizar. Despois compónse enteiro, coa escena ao día, sen presentalo:
  - **ao restaurala**, píntase a xanela enteira desde o *framebuffer* (`invalidate`), que xa inclúe todos os cambios: non queda nada que presentar, nin marcas. O temporizador detense, porque aínda pode estar programado para o que a escena tiña que facer coa xanela minimizada: se a escena non pide ningún momento, `wait_for_scene` non o toca, e ese temporizador vencería despois e pediría un fotograma inútil. Despois, `wait_for_scene` prográmao para o momento da escena, se o pediu;
  - **noutro caso**, Windows pide `WM_PAINT` para a parte nova da xanela, e o resto segue igual; as zonas que cambiaran e aínda non se presentaran preséntanse co seguinte fotograma (`request`).
- **Compóñese enteiro**, porque despois dun cambio de tamaño o *framebuffer* pode ter memoria nova, sen píxeles, ou zonas que antes quedaban fóra da xanela. É máis sinxelo ca levar a conta das partes que seguen ben, e o contido pode depender do tamaño: as marxes do terminal dependen del e, desde a subfase 1.9, tamén a súa grella. Mentres se redimensiona co rato, cada paso compón a xanela enteira, pero só presenta a parte nova.

```rust
pub fn size_move_ended(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
    self.live_resize = false;
    if self.surface.resize(self.surface.size(), Fit::Exact)? {
        self.compose_all(scene)?;
        return self.request(window, scene);
    }
    Ok(())
}
```

Ao rematar de redimensionar co rato, o *framebuffer* devolve a memoria de máis (`Exact`). Se a memoria é nova, compónse de novo, coa escena ao día. A xanela xa mostra o mesmo, agás o que cambiase na escena mentres tanto, que presenta o seguinte fotograma (`request`).

### `WM_PAINT` e as marcas

```rust
pub fn paint(&mut self, dc: HDC, area: Rect) -> Result<(), HandlerError> {
    let start = Instant::now();
    let _ = self.surface.paint(dc, area);
    …
}

pub fn toggle_marks(&mut self, window: &Window) {
    match self.marks.take() {
        Some(marks) => {
            let _ = erase(&self.surface, window.handle(), &marks);
        }
        None => self.marks = Some(Vec::new()),
    }
}
```

- **`paint`** copia a zona que pide Windows e anótao no rexistro. Se a copia falla, esa zona queda mal ata o seguinte repintado, que non é motivo para parar meiga. Como `BeginPaint` xa validou a zona, Windows non a volta pedir, e non hai bucle.
- **`toggle_marks`**: ao desactivalas, bórranse as que hai na xanela; ao activalas, o seguinte fotograma xa as debuxa.

### Funcións auxiliares

```rust
fn interval() -> Duration {
    auto_interval(window::is_remote_session())
}

fn erase(surface: &Surface, window: HWND, marks: &[Rect]) -> Result<(), meiga_render::Error> {
    for mark in marks {
        surface.present(window, &mark.border(MARK_THICKNESS))?;
    }
    Ok(())
}

fn log(trace: &mut Option<Trace>, record: &Record<'_>) -> Result<(), HandlerError> {
    if let Some(trace) = trace {
        trace.write(record).map_err(|error| format!("Cannot write the frame log: {error}"))?;
    }
    Ok(())
}
```

- `erase` borra as marcas presentando de novo os catro lados do bordo de cada unha.
- `log` escribe no rexistro, se o hai. Se falla, por exemplo porque o disco está cheo, meiga para cunha mensaxe: o rexistro só se usa para comprobar e medir, e un rexistro incompleto daría resultados falsos.
- As marcas son magenta (`#FF00FF`), para que se distingan do contido, e dun píxel de ancho.

## `crates/meiga/src/trace.rs`

O **rexistro dos fotogramas** (`--trace-frames`): unha liña CSV por cada vez que meiga debuxa algo. Serve para comprobar que se presenta e para medir canto custa ([`check-present.ps1`](ferramentas.md#toolscheck-presentps1) e [`measure-frames.ps1`](perf.md#toolsperfmeasure-framesps1)). Un exemplo, coas primeiras liñas dun rexistro da subfase 1.3, cando a xanela mostraba unha demo na que se debuxaban rectángulos co rato, ao premer e arrastrar (os tempos dependen da máquina):

```text
frame,time_ms,kind,cause,interval_ms,width,height,rects,pixels,compose_us,present_us,areas
1,6.355,compose,,33.333,1418,769,1,1090442,917,0,0 0 1418 769
2,24.614,paint,,33.333,1418,769,1,1090442,0,902,0 0 1418 769
3,362.585,frame,now,33.333,1418,769,1,1,0,111,150 150 1 1
4,396.140,frame,timer,33.333,1418,769,1,2806,4,90,150 150 61 46
```

| Columna | Que é |
|---|---|
| `frame` | O número da liña: 1, 2, 3… |
| `time_ms` | Cando empezou, en milisegundos desde que se abriu o rexistro. |
| `kind` | `compose`: compúxose o *framebuffer* enteiro, sen presentalo (ao arrancar e ao cambiar o tamaño); `paint`: Windows pediu pintar unha zona (`WM_PAINT`); `frame`: meiga presentou o que cambiou. |
| `cause` | Nos `frame`: `now` se se debuxou no momento, porque xa pasara o intervalo, ou `timer` se o debuxou o temporizador. |
| `interval_ms` | O intervalo entre fotogramas nese momento: 33,333 en remoto e 16,667 en local. |
| `width` e `height` | O tamaño do *framebuffer*. |
| `rects` e `pixels` | Cantas zonas, e cantos píxeles en total: as presentadas (`frame`), a pintada (`paint`) ou a composta (`compose`). |
| `compose_us` e `present_us` | O tempo de compoñer e o de presentar, en microsegundos. |
| `areas` | As zonas, como `x y ancho alto`, separadas por `;`. |

- Co terminal, o rexistro empeza igual, cun `compose` e un `paint` da xanela enteira, e despois cada `frame` presenta as celas que cambiaron.
- As marcas (tecla P) non contan nas zonas: son unha axuda visual, e os modos de medición non as usan.
- Os decimais van sempre cun punto, aínda que Windows estea en galego.

```rust
pub fn write(&mut self, record: &Record<'_>) -> io::Result<()> {
    self.lines += 1;
    self.line.clear();
    format_line(&mut self.line, self.lines, self.start, record);
    self.file.write_all(self.line.as_bytes())
}
```

- Cada liña escríbese no arquivo **no momento**, cunha soa chamada: se meiga remata de calquera maneira, o rexistro está completo ata a última liña, e outro programa pódeo ler mentres meiga corre.
- `line` é un `String` que se reutiliza: `clear()` baléirao sen liberar a memoria, así que escribir unha liña non reserva memoria nova.
- `Record` leva os datos dunha liña. `rects: &'a [Rect]` é unha referencia ás zonas, sen copialas.

```rust
fn write_millis(line: &mut String, duration: Duration) {
    let micros = duration.as_nanos().saturating_add(500) / 1000;
    let _ = write!(line, "{}.{:03}", micros / 1000, micros % 1000);
}
```

- Os milisegundos con tres decimais escríbense con números enteiros: os microsegundos, redondeados (`+ 500`), divididos entre 1000, un punto e o resto con tres cifras (`{:03}`). Formatar números decimais (`f64`) engadiría uns 20 KB a `meiga.exe`; comprobouse compilando as dúas versións.
- `write!` sobre un `String` non pode fallar, pero devolve un `Result`: `let _ =` descártao.

**Probas:** dúas liñas completas, un `frame` co temporizador e dúas zonas e un `paint` sen causa, comparadas co texto esperado. A primeira comproba tamén que a liña ten tantas columnas coma a cabeceira.

## `crates/meiga/src/bench.rs`

Os **modos de medición** (`--bench`): fotogramas sen parar, ao máximo que permite o intervalo, para medir que custa un fotograma parcial, un completo e un completo de texto a meiga e ao resto do sistema. Úsaos [`measure-frames.ps1`](perf.md#toolsperfmeasure-framesps1).

```rust
pub fn step(&mut self, size: Size) -> [Rect; 2] {
    self.frame += 1;
    match self.mode {
        BenchMode::Partial => {
            let old = self.square;
            self.square = square_at(self.frame, size);
            [old, self.square]
        }
        BenchMode::Full | BenchMode::Text => [Rect::from_size(size), Rect::EMPTY],
    }
}

fn square_at(frame: u64, size: Size) -> Rect {
    let positions = u64::from(size.width.saturating_sub(SQUARE)) + 1;
    let x = i32::try_from(frame.wrapping_mul(STEP) % positions).unwrap_or(0);
    let y = i32::try_from(size.height / 2).unwrap_or(0) - SQUARE as i32 / 2;
    Rect::new(x, y, x + SQUARE as i32, y + SQUARE as i32)
}
```

- **`partial`**: un cadrado azul de 64 × 64 píxeles cruza o medio da xanela, 16 píxeles por fotograma, e ao chegar á dereita empeza de novo. Cambian o sitio onde estaba e o sitio onde está: dous rectángulos que `Damage` xunta nun de 80 × 64, ou deixa separados cando o cadrado volta á esquerda.
- **`full`**: a xanela enteira cambia entre dúas cores moi parecidas (`#16181D` e `#1C1F26`) en cada fotograma. Preséntase toda: o peor caso, como desprazar unha saída masiva. As cores son parecidas para que a xanela non pestanexe.
- **`text`**: a xanela enteira chea de texto, que cambia en cada fotograma. Tamén se presenta toda, pero agora cada cela busca o seu glifo na caché e debuxa a súa tinta: o peor caso do texto.
- `step` devolve as zonas que cambian nun *array* de dous, sen reservar memoria. `Rect::EMPTY` non engade nada a `Damage`.
- `wrapping_mul` multiplica sen comprobar o desbordamento: a 30 fotogramas por segundo, tardaría máis de mil millóns de anos en chegar. `%` dá a posición dentro das que caben.

```rust
BenchMode::Text => {
    let Some((font, row)) = &mut self.text else {
        canvas.fill(canvas.clip(), BACKGROUND);
        return;
    };
    let cell = font.cell();
    let size = canvas.size();
    let columns = size.width.div_ceil(cell.width);
    for y in 0..size.height.div_ceil(cell.height) {
        row.clear();
        row.extend((0..columns).map(|x| text_at(self.frame, y, x)));
        let top = i32::try_from(y * cell.height).unwrap_or(i32::MAX);
        font.draw(canvas, Point::new(0, top), row, FontStyle::REGULAR, FOREGROUND, BACKGROUND);
    }
}

fn text_at(frame: u64, row: u32, column: u32) -> CellChar {
    let index = (u64::from(row) * 7 + u64::from(column) + frame) % CHARS;
    CellChar::Narrow(char::from(FIRST_CHAR + index as u8))
}
```

O debuxo dun fotograma de texto:

- **Os caracteres** son os 94 visíbeis de ASCII, de `!` a `~`, que nunca deixan unha cela en branco. En cada fotograma, o texto móvese unha cela á esquerda (`+ frame`), así que cambian todas as celas. Cada fila vai 7 caracteres por diante da anterior, para que as columnas non repitan o mesmo carácter.
- **As filas cobren a xanela enteira**, as últimas cortadas polos bordos (`div_ceil` divide redondeando cara arriba), así que non fai falta pintar ningún outro fondo: cada fila pinta o seu.
- `row` gárdase entre fotogramas para encher a seguinte fila sen reservar memoria.
- **A fonte** chega en `Bench::new(mode, font)`: créaa [`App`](#cratesmeigasrcapprs) co DPI da xanela, e só a usa este modo. Sen fonte, os fotogramas de texto só pintan o fondo: o programa sempre lla dá, pero o tipo non o pode garantir. Como a fonte se crea antes do primeiro fotograma, os glifos rasterízanse nel, e os fotogramas que se miden só os buscan na caché.
- **As cores e a fonte son as do terminal.** Desde a 1.6, sen a demo, as cores veñen de `meiga-core`: o texto e o fondo de meiga ([`FOREGROUND` e `BACKGROUND`](meiga-core.md#cratesmeiga-coresrccolorrs)). O fondo é tamén o de `partial` e a primeira cor de `full`. A fonte é a da vista do terminal ([`FONT_FAMILIES` e `FONT_POINTS`](#constantes)), sempre co estilo normal (`FontStyle::REGULAR`). Así, os fotogramas de texto miden o mesmo texto que debuxa o terminal.

**Probas.** Catro: o cadrado avanza e marca o sitio vello e o novo; volta á esquerda ao chegar á dereita (tamén nunha xanela máis estreita ca el); os fotogramas completos marcan a xanela enteira e alternan as dúas cores; e os de texto cobren a xanela enteira, con píxeles da cor do texto, e cada cela cambia dun fotograma ao seguinte, sen quedar nunca en branco (`!` e `~` son o primeiro e o último carácter).

## Comprobacións feitas

- `cargo build --release` xera `target\release\meiga.exe`, duns 470 KB.
- Ao abrilo sen argumentos, aparece a xanela, sen ningunha consola, con PowerShell 7 dentro: a súa cabeceira, o *prompt* e, despois del, o cursor, unha barra. Ao pechala, o proceso remata co código 0.
- Un script de PowerShell que escribe as 16 cores, as 256, un degradado *truecolor*, a negra, a cursiva, o texto tenue, inverso e oculto, os cinco subliñados, o riscado e caracteres anchos móstraos todos como describe [`view.rs`](#cratesmeigasrcviewrs): os ideogramas e o emoji, en dúas celas (o emoji, dunha soa cor, [ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Outro, que cambia a cor do fondo, a do texto e a vermella da paleta (`OSC 11`, `10` e `4`), cambia tamén a cor do texto que xa estaba escrito.
- Cun programa que non existe, o terminal mostra o erro de `CreateProcessW` coa liña de comandos que fallou, sen cursor. Cun programa que remata, a súa saída queda na xanela, cunha liña como `[process exited with code 3 (0x00000003)]` debaixo, e a xanela segue aberta ata que se pecha.
- O texto compárase co do Windows Terminal coa mesma fonte, o mesmo tamaño e as mesmas cores: os glifos teñen exactamente os mesmos píxeles cubertos, enteiros e parciais, e a tinta dunha liña é a mesma, cunha diferenza menor do 0,5 % ([Tecnoloxías › Mesturar o texto co fondo](../tecnoloxias.md#30-mesturar-o-texto-co-fondo-gamma-e-contraste)).
- O crate ten 17 probas unitarias (`cargo test -p meiga`). As novas da 1.6 son `the_program_and_its_arguments_are_the_command_line`, en `options.rs`; `the_window_is_woken_up_once_until_it_takes_what_changed`, `a_synchronized_update_wakes_the_window_once_and_ends_at_its_deadline` e `the_end_of_the_program_is_on_a_row_of_its_own_without_the_cursor`, en `session.rs`; e `the_cursor_blinks_until_the_timeout_and_stays_shown`, `the_cursor_does_not_blink_unless_everything_allows_it`, `a_dotted_underline_keeps_its_dots_when_a_frame_draws_part_of_it` e `styles_give_the_colours_of_the_text_and_its_background`, en `view.rs`. A das opcións sen argumentos é agora `no_options_run_the_default_shell`, e as sete da demo desapareceron con ela. A da actualización sincronizada e a do subliñado de puntos chegaron co arranxo de dous erros que atopou unha revisión do código: coa xanela minimizada, unha actualización sincronizada que o programa deixaba aberta non remataba ata restaurala, e as preguntas de dentro quedaban sen resposta; e os puntos do subliñado desprazábanse un píxel cando un fotograma debuxaba só parte da liña.
- `tools/check-window.ps1` comproba o subsistema, a xanela, a escala por monitor e o código de saída. Desde a 1.6, abre meiga con `cmd` no terminal, e comproba tamén que meiga lanza `cmd` e a súa consola, que o título da xanela pasa a ser o que pon `cmd` e que, ao pechar a xanela, `cmd` e a súa consola rematan tamén.
- `tools/check-present.ps1` comproba os fotogramas no rexistro (`--trace-frames`). Desde a 1.6, executa no terminal un script de PowerShell que escribe na pantalla un paso cada vez, cando a comprobación llo pide, e fai dez comprobacións: o primeiro `paint` cobre a xanela enteira; mentres cambia unha fila, cada fotograma presenta só esa fila; en repouso non se debuxa nada; moitos cambios escritos de golpe agrúpanse en poucos fotogramas; unha cor de fondo nova (`OSC 11`) dá un fotograma completo; coa xanela minimizada non se debuxa nada, aínda que a saída cambie; aínda minimizada, o programa abre unha actualización sincronizada cun título dentro e non a remata: remata igual á súa hora, o título chega á xanela, e non se debuxa nada; ao restaurala, píntase enteira; ao medrar a xanela, compón o tamaño novo e píntao; e, ao pechala, meiga sae co código 0. O script oculta antes o cursor, para que o pestanexo non debuxe nada en repouso (ver [Código › ferramentas](ferramentas.md#toolscheck-presentps1)).
- `check-present.ps1` detecta os erros: comprobouse na subfase 1.2, cando aínda enviaba eventos do rato á demo, con copias de `frames.rs` con erros feitos a propósito. Debuxando cada cambio no momento, sen agrupalos, o clic e os 40 movementos enviados de golpe dan 41 fotogramas; e debuxando coa xanela minimizada, aparecen fotogramas. Nos dous casos, a comprobación falla. Sen pedir o repintado ao restaurar a xanela, pasa igual, porque Windows xa o pide. Na 1.6, unha copia de meiga sen o arranxo das actualizacións sincronizadas coa xanela minimizada falla a sétima comprobación: o título non chega á xanela.
- Os modos de medición funcionan: o temporizador dá os fotogramas ao ritmo pedido, tamén mentres se move a xanela, aínda que entón máis lentos (ver [ADR 0008](../adr/0008-temporizador-dos-fotogramas.md)).
- O manifesto que leva dentro, extraído con `mt.exe`, ten as tres declaracións, e un só `trustInfo`.
- `dumpbin /dependents` mostra que só depende de DLL do sistema: `kernel32.dll`, `dwrite.dll`, `ntdll.dll`, `oleaut32.dll`, `combase.dll`, `gdi32.dll`, `user32.dll` e `api-ms-win-core-synch-l1-2-0.dll`. As tres novas da subfase 1.3 son DirectWrite e as dúas que usa o crate `windows` (ver [Tecnoloxías › COM e o crate `windows`](../tecnoloxias.md#28-com-e-o-crate-windows)). Ata a 1.4, `kernel32.dll` aparecía dúas veces, unha delas como `KERNEL32.dll`, porque a biblioteca estándar de Rust e `windows-sys` declaraban as súas funcións de forma distinta; desde a 1.5, as dúas van nunha soa entrada (ver [Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust)). Windows cárgaa sempre unha soa vez. A 1.6 non engade ningunha: a ConPTY de Windows, as tubaxes, os procesos e os *jobs* están en `kernel32.dll`, e a ConPTY que vai no paquete, `conpty.dll`, cárgase ao lanzar a sesión (`LoadLibraryExW`).

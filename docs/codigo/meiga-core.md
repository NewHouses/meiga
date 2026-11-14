# Código › `meiga-core`

> Estado: subfase 2.1. Contén o modelo das xanelas e as súas tabs, as cores e a paleta, coa cor da selección, a xeometría (puntos, tamaños e rectángulos, e o tamaño e as celas da grella), as zonas que cambiaron (*damage*), o ritmo dos fotogramas, as celas do texto, a liña de comandos dun programa, as teclas, coma eventos e coma o que le o programa, o rato (a cela debaixo do punteiro, os clics seguidos, a roda, en partes e en filas do historial, e os informes para os programas), o texto que se pega, e o terminal: a fachada sobre `alacritty_terminal`, coas súas respostas, a súa paleta, a selección e a vista do historial, o cambio de tamaño, que coloca a pantalla coma o ConPTY e garda á parte a principal mentres se ve a alternativa, o escáner previo e as gravacións. Na 2.1 chegou o modelo, coas súas probas e un exemplo para probalo a man.

**Responsabilidade:** toda a lóxica que non depende de Windows: o modelo de xanelas, tabs e grupos; a fachada do terminal; a codificación do teclado e do rato; a busca difusa; e a configuración. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeiga-corecargotoml) | O crate e as súas dependencias: `alacritty_terminal`, `memchr` e `unicode-width`. |
| [`src/lib.rs`](#cratesmeiga-coresrclibrs) | A raíz do crate: os módulos. |
| [`src/color.rs`](#cratesmeiga-coresrccolorrs) | `Rgb`, o tipo das cores, e `Palette`, as cores dun terminal: as 16 do esquema *Campbell*, as 256 do xterm e a da selección. |
| [`src/geometry.rs`](#cratesmeiga-coresrcgeometryrs) | `Point`, `Size` e `Rect`: posicións, tamaños e rectángulos en píxeles; `GridSize`, o tamaño da grella en celas; e `GridPoint`, unha cela da grella. |
| [`src/damage.rs`](#cratesmeiga-coresrcdamagers) | `Damage`: as zonas da xanela que cambiaron desde o último fotograma. |
| [`src/pacing.rs`](#cratesmeiga-coresrcpacingrs) | `FramePacer`: cando debuxar o seguinte fotograma. |
| [`src/text.rs`](#cratesmeiga-coresrctextrs) | `CellChar` e `cells`: o texto como celas dunha grella. |
| [`src/shell.rs`](#cratesmeiga-coresrcshellrs) | `command_line`: a liña de comandos dun programa cos seus argumentos, entre comiñas segundo as regras de Windows. |
| [`src/input.rs`](#cratesmeiga-coresrcinputrs) | `KeyEvent`, unha tecla que baixa ou sobe, e `KeyEncoder`, que a converte no que le o programa: en win32-input-mode ou en VT. Os eventos de foco. |
| [`src/mouse.rs`](#cratesmeiga-coresrcmousers) | O rato: `cell_at`, a cela debaixo do punteiro; `Clicks`, que conta os clics seguidos; `WheelTurns`, que suma os xiros da roda en partes, e `WheelRows`, en filas do historial; `report`, os informes para os programas que piden o rato, en SGR, en UTF-8 ou no formato do primeiro xterm; e `arrow_keys`, a roda coma frechas. |
| [`src/paste.rs`](#cratesmeiga-coresrcpasters) | `Paste`: o texto que le o programa ao pegar, sen caracteres de control, cos fins de liña en CR e, se o pediu, entre as marcas do *bracketed paste*. |
| [`src/model/mod.rs`](#cratesmeiga-coresrcmodelmodrs) | O modelo das xanelas e as súas tabs: os identificadores, `TabId` e `WindowId`, que nunca se repiten, e os erros. |
| [`src/model/window.rs`](#cratesmeiga-coresrcmodelwindowrs) | `Window`: as tabs dunha xanela, na orde da barra, e a activa, a enfocada e as visíbeis; abrir, pechar, mover e activar. |
| [`src/model/tests.rs`](#cratesmeiga-coresrcmodeltestsrs) | As probas do modelo, unha delas con 10 000 cambios ao chou. |
| [`src/term/mod.rs`](#cratesmeiga-coresrctermmodrs) | `Terminal`: a fachada sobre `alacritty_terminal`, cos seus eventos, modos e respostas, a paleta, o cambio de tamaño, a vista do historial e a copia do que cambia, coas celas seleccionadas. |
| [`src/term/resize.rs`](#cratesmeiga-coresrctermresizers) | O cambio de tamaño da pantalla principal, que reparte o texto e coloca a pantalla onde a pon o ConPTY, seguindo a primeira fila e o cursor cunhas marcas; e a pantalla principal á parte mentres se ve a alternativa. |
| [`src/term/handler.rs`](#cratesmeiga-coresrctermhandlerrs) | `Hooked`: o `Handler` entre o parser e o `Term`, que lle pasa todas as ordes, devolve a pantalla principal cando o programa deixa a alternativa ou reinicia o terminal, e deixa a vista do historial onde está cando se despraza unha rexión que non empeza arriba. |
| [`src/term/screen.rs`](#cratesmeiga-coresrctermscreenrs) | `Screen`: o que mostra o terminal, en tipos de meiga, coa súa paleta, e as celas que cambiaron. |
| [`src/term/selection.rs`](#cratesmeiga-coresrctermselectionrs) | A selección: creala, movela e quitala, o seu texto, e as súas celas, na grella de `alacritty_terminal`. |
| [`src/term/scanner.rs`](#cratesmeiga-coresrctermscannerrs) | `Scanner`: o escáner que le a saída antes do parser, polo de agora para `?9001`. |
| [`src/term/recording.rs`](#cratesmeiga-coresrctermrecordingrs) | `Recorder` e `Recording`: gravar a saída dun programa e lela. |
| [`src/term/tests.rs`](#cratesmeiga-coresrctermtestsrs) | As probas do terminal. |
| [`tests/recordings.rs`](#cratesmeiga-coretestsrecordingsrs) e `tests/data/` | Probas con gravacións reais do ConPTY. |
| [`examples/replay.rs`](#cratesmeiga-coreexamplesreplayrs) | Un exemplo de consola que lle dá unha gravación ao terminal e mide a velocidade. |
| [`examples/tabs.rs`](#cratesmeiga-coreexamplestabsrs) | Un exemplo de consola para probar o modelo das tabs a man. |

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

[dependencies]
# The terminal core, behind the facade of the term module (ADR 0003).
alacritty_terminal.workspace = true
# Finds ESC fast in the scanner that reads the output before the parser.
memchr.workspace = true
# The width of each character in cells, as alacritty_terminal measures it.
unicode-width.workspace = true

[lints]
workspace = true
```

- `name`: o nome do crate. No código úsase con guión baixo: `meiga_core`.
- `description`: unha liña que describe o crate.
- As liñas `campo.workspace = true` herdan os valores de `[workspace.package]` do `Cargo.toml` raíz.
- `[dependencies]`, coas versións e as opcións do `Cargo.toml` raíz ([Código › workspace](workspace.md#cargotoml-raíz)):
  - desde a subfase 1.5, **`alacritty_terminal`**, o núcleo do terminal, que trae o seu parser, `vte` ([Tecnoloxías › `alacritty_terminal` e `vte`](../tecnoloxias.md#34-alacritty_terminal-e-vte-o-núcleo-do-terminal)). Só o usa o módulo `term`;
  - desde a 1.5, **`memchr`**, que busca un byte nun texto moitos bytes á vez, con instrucións SIMD: o escáner busca así o seguinte ESC. `vte` faino igual co mesmo crate, así que non engade ningún crate novo;
  - desde a 1.3, **`unicode-width`**, que dá o ancho de cada carácter en celas, como o mide `alacritty_terminal` ([Tecnoloxías › O ancho dos caracteres](../tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)).

  Os tres son código Rust que non usa APIs de ningún sistema desde o que chama meiga, así que respectan as regras deste crate. `alacritty_terminal` ten tamén un PTY propio con código de Windows, que meiga non usa e que o ligador elimina.
- `[lints] workspace = true`: aplica as regras de calidade comúns.

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
pub mod damage;
pub mod geometry;
pub mod input;
pub mod model;
pub mod mouse;
pub mod pacing;
pub mod paste;
pub mod shell;
pub mod term;
pub mod text;
```

- `lib.rs` é a raíz dunha biblioteca: o compilador empeza por este arquivo, e desde el decláranse os demais módulos.
- As liñas que empezan por `//!` son **documentación do crate**. `cargo doc` convérteas en documentación HTML.
- `#![forbid(unsafe_code)]` é un **atributo interno** (`#!`), que se aplica a todo o crate. Prohíbe calquera bloque `unsafe` en `meiga-core`. Co nivel `forbid`, ningún módulo pode desactivar a regra cun `#[allow]`. Así, esta parte do programa, que será a máis grande, conta con todas as garantías de seguridade de memoria de Rust.
- `pub mod color;` declara o módulo `color`, que está en `src/color.rs`. Con `pub`, os outros crates poden usalo como `meiga_core::color`. O mesmo cos demais; `shell` chegou na subfase 1.6, `input`, na 1.7, `mouse` e `paste`, na 1.8, e `model`, na 2.1. `term`, desde a 1.5, e `model` teñen submódulos, así que van en cartafoles: os módulos están en `src/term/mod.rs` e en `src/model/mod.rs`.
- As dependencias, como `alacritty_terminal`, poden ter `unsafe` dentro: `forbid` só se aplica ao código deste crate.

## `crates/meiga-core/src/color.rs`

As cores e, desde a subfase 1.6, a paleta dun terminal: a cor coa que se debuxa cada número das celas ([ADR 0013](../adr/0013-da-saida-a-pantalla.md), [Tecnoloxías › A paleta e os estilos](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). Desde a 1.8, a paleta ten tamén a cor da selección ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

### `Rgb`

```rust
//! Colours, and the palette of a terminal.

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

Unha cor en RGB, con 8 bits (0–255) por canle: vermello (`r`), verde (`g`) e azul (`b`). Desde a subfase 1.2 úsase para todo o que se debuxa no *framebuffer*, desde a 1.3 tamén para as cores do texto, e desde a 1.6 para a paleta do terminal; máis adiante servirá para os temas e a configuración.

- Está en `meiga-core`, e non nun crate de Windows, porque unha cor non depende de Windows. Cada uso convértea ao seu formato: `meiga-render` convértea nun píxel do *framebuffer* (`0x00RRGGBB`) e, para as brochas de GDI, nun `COLORREF` (`0x00BBGGRR`); ver [Código › meiga-render](meiga-render.md#pixel).
- `#[derive(…)]` fai que o compilador escriba automaticamente estas implementacións:
  - `Clone` e `Copy`: unha cor son 3 bytes, así que se copia ao asignala ou ao pasala a unha función, en vez de *moverse* (en Rust, por defecto, asignar un valor transfire a súa propiedade e o orixinal xa non se pode usar);
  - `Debug`: permite mostrala nas mensaxes de diagnóstico con `{:?}`;
  - `PartialEq` e `Eq`: permiten comparar dúas cores con `==`;
  - `Hash`: permite usala como clave nun `HashMap`, por exemplo nunha táboa cun valor por cor.
- `const fn new`: unha función **constante**, que o compilador pode executar mentres compila. Por iso as cores se poden declarar como constantes: `const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);`.
- Os campos son públicos (`pub`): calquera combinación de tres bytes é unha cor válida, así que non hai ningunha regra que protexer.

```rust
pub const fn mix(self, other: Rgb, amount: u8) -> Rgb {
    const fn channel(from: u8, to: u8, amount: u8) -> u8 {
        let (from, to, amount) = (from as u32, to as u32, amount as u32);
        // Rounded to the nearest. The result is between `from` and `to`,
        // so it fits in a byte.
        ((from * (255 - amount) + to * amount + 127) / 255) as u8
    }
    Rgb::new(
        channel(self.r, other.r, amount),
        channel(self.g, other.g, amount),
        channel(self.b, other.b, amount),
    )
}
```

**`mix`**, desde a subfase 1.6, dá a cor que está a `amount` do camiño desde esta ata `other`, en 255 avos: con 0, esta; con 255, `other`; e con 85, a un terzo do camiño. A vista do terminal úsaa para o texto tenue (*SGR* 2): a cor do texto, un terzo do camiño cara á do fondo ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

- Cada canle é unha media ponderada das dúas cores: `from × (255 − amount) + to × amount`, entre 255. Sumar 127, a metade de 255, antes de dividir redondea ao enteiro máis próximo: a división de enteiros só corta os decimais.
- As contas fanse en `u32`, porque os produtos non caben nun `u8`. A conversión é con `as`, e non con `u32::from`, porque unha `const fn` non pode chamar métodos de traits, como `From::from`. O resultado sempre está entre `from` e `to`, así que o `as u8` do final nunca corta nada.
- `channel` é unha función dentro doutra: só a ve `mix`. Tamén é `const fn`, para que `mix` o poida ser: unha cor mesturada pode ser unha constante.

### `Palette`

```rust
pub const CAMPBELL: [Rgb; 16] = [
    Rgb::new(0x0c, 0x0c, 0x0c),
    Rgb::new(0xc5, 0x0f, 0x1f),
    …
    Rgb::new(0xf2, 0xf2, 0xf2),
];

pub const FOREGROUND: Rgb = Rgb::new(0xd4, 0xd7, 0xdd);
pub const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);
pub const SELECTION: Rgb = Rgb::new(0x26, 0x4f, 0x78);
```

- **`CAMPBELL`**: as 16 cores do esquema *Campbell*, o que usan por defecto o Windows Terminal e a consola de Windows desde Windows 10: negro, vermello, verde, amarelo, azul, magenta, cian e branco, e as súas versións brillantes, na orde das secuencias de cor (`30`–`37` e `90`–`97`). Os valores, nunha táboa, en [Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas).
- **`FOREGROUND`** e **`BACKGROUND`**: o texto e o fondo de meiga, `#D4D7DD` e `#16181D`, en vez dos de *Campbell*. Antes estaban na demo de `meiga.exe`, que desaparece na subfase 1.6: agora viven xunto coa paleta, e os modos de medición (`--bench`) tamén as usan.
- **`SELECTION`**, desde a subfase 1.8: o fondo das celas seleccionadas, `#264F78`, o do tema escuro do Visual Studio Code. O texto mantén a súa cor ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

```rust
const CUBE_LEVELS: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub colors: [Rgb; 256],
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
    pub selection: Rgb,
}
```

**`Palette`**, desde a subfase 1.6: as cores dun terminal. As celas gardan o número da súa cor, non a cor ([`Color`](#cratesmeiga-coresrctermscreenrs)), e a paleta dá a cor de cada número ao debuxalas:

| Campo | Que é |
|---|---|
| `colors` | As 256 cores que os programas escollen polo seu número: 0–15 as que teñen nome, 16–231 un cubo de 6 × 6 × 6 e 232–255 grises, de escuro a claro. |
| `foreground`, `background` | O texto e o fondo cando o programa non escolle ningunha cor (`Color::Default`). |
| `cursor` | A cor do cursor. |
| `selection` | Desde a subfase 1.8: o fondo das celas seleccionadas. Os programas non a poden cambiar: non ten número nas secuencias, e o terminal cóllea do tema tal cal ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |

- Os campos son públicos, como os de `Rgb`: calquera combinación é unha paleta válida.
- Deriva `Clone`, `Debug`, `PartialEq` e `Eq`, pero non `Copy`, ao contrario ca `Rgb`: son 780 bytes, e así cada copia vese no código, cun `clone`.

```rust
impl Palette {
    pub const MEIGA: Palette =
        Palette::new(CAMPBELL, FOREGROUND, BACKGROUND, FOREGROUND, SELECTION);

    pub const fn new(
        named: [Rgb; 16],
        foreground: Rgb,
        background: Rgb,
        cursor: Rgb,
        selection: Rgb,
    ) -> Self {
        let mut colors = [Rgb::new(0, 0, 0); 256];
        let mut index = 0;
        while index < 256 {
            colors[index] = match index {
                0..16 => named[index],
                16..232 => {
                    let cube = index - 16;
                    Rgb::new(
                        CUBE_LEVELS[cube / 36],
                        CUBE_LEVELS[cube / 6 % 6],
                        CUBE_LEVELS[cube % 6],
                    )
                }
                // 8, 18… 238: less than 256.
                _ => {
                    let grey = (8 + 10 * (index - 232)) as u8;
                    Rgb::new(grey, grey, grey)
                }
            };
            index += 1;
        }
        Self {
            colors,
            foreground,
            background,
            cursor,
            selection,
        }
    }
}
```

**`new`** crea a paleta do xterm coas 16 cores `named`: as outras 240 son as mesmas en todos os terminais ([Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). O texto, o fondo, o cursor e, desde a subfase 1.8, a selección van tal cal.

- **O cubo**, da 16 á 231: `cube = index − 16`, e os seus tres díxitos en base 6 son o vermello (`cube / 36`), o verde (`cube / 6 % 6`) e o azul (`cube % 6`), así que a cor `16 + 36 r + 6 g + b` ten as compoñentes `r`, `g` e `b`. Cada díxito escolle un nivel de `CUBE_LEVELS`: 0 e, desde `0x5f`, en pasos de `0x28` (40) ata `0xff`. Por exemplo, a 67 é 16 + 36 × 1 + 6 × 2 + 3: `(0x5f, 0x87, 0xaf)`.
- **Os grises**, da 232 á 255: `8 + 10 i`, de 8 a 238. O negro e o branco xa están no cubo, na 16 e na 231. O comentario lembra por que o `as u8` non corta nada: o maior é 238.
- **Por que `const fn`.** Así `MEIGA` é unha constante, que o compilador calcula mentres compila: as 256 cores van no programa como datos, e crear un terminal ou unha pantalla só as copia. `Screen::new`, que tamén é `const`, só pode usar unha paleta constante ([`screen.rs`](#cratesmeiga-coresrctermscreenrs)). Nunha `const fn` non se pode usar `for`, que chama os métodos do trait `Iterator`: por iso o bucle é un `while` cun índice.
- **`MEIGA`** son as cores de meiga ata que haxa temas (subfase 8.3): as 16 de *Campbell*, o texto e o fondo de meiga, o cursor da cor do texto ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)) e, desde a 1.8, o fondo da selección, `SELECTION` ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).
- **`Default`** dá `MEIGA`: así os tipos que levan unha paleta, como `Screen`, poden derivar `Default`.

**Probas.** Dúas:

| Proba | Que comproba |
|---|---|
| `the_palette_has_the_256_colours_of_xterm` | As 16 primeiras cores son as de *Campbell*; as esquinas do cubo (a 16, negra, e a 231, branca) e unha cor de dentro (a 67); o primeiro e o último gris (8 e 238); o texto, o fondo, o cursor e, desde a subfase 1.8, a selección de `MEIGA`; e que `Default` é `MEIGA`. |
| `mixing_goes_from_one_colour_to_the_other` | Con 0 e con 255, as dúas cores; con 85, do negro ao branco, o gris 85, e do branco ao negro, o 170; e con 128, case a metade, redondea ao máis próximo: o verde, de 200 a 100, dá 150 (149,8), e non 149. |

## `crates/meiga-core/src/geometry.rs`

Puntos, tamaños e rectángulos en píxeles: os tipos cos que todos os crates falan de posicións na xanela. Desde a subfase 1.4, tamén o tamaño da grella, en celas ([`GridSize`](#gridsize)), e desde a 1.8, unha cela da grella ([`GridPoint`](#gridpoint)).

```rust
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
```

- **`Point`**: unha posición, en píxeles desde a esquina superior esquerda. As coordenadas son `i32` porque poden ser negativas: por exemplo, o rato fóra da xanela mentres se arrastra.
- **`Size`**: un tamaño, que nunca é negativo (`u32`). `area()` dá os píxeles como `u64`, porque `u32 × u32` pode non caber nun `u32`, e `is_empty()` di se non ten ningún.
- **`Rect`**: un rectángulo. Como o `RECT` de Win32, vai desde `left` e `top`, incluídos, ata `right` e `bottom`, **excluídos**: o seu ancho é `right - left`, e dous rectángulos que comparten un lado non se tocan. Cando `right <= left` ou `bottom <= top`, está baleiro.
- Os tres derivan `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq` e `Hash`, como `Rgb`. `Default` dá o valor cero.

| Función de `Rect` | Que fai |
|---|---|
| `EMPTY` | O rectángulo baleiro que devolven as operacións cando non queda ningún píxel. |
| `new` e `from_size` | Un rectángulo cos seus catro lados, ou o dun tamaño desde (0, 0). |
| `at(origin, size)` | O rectángulo dun tamaño coa esquina superior esquerda en `origin`: por exemplo, unha cela na súa posición. |
| `offset(by)` | O mesmo rectángulo, movido. As sumas quedan no máximo de `i32` en vez de desbordar (`saturating_add`). |
| `spanning(a, b)` | O rectángulo máis pequeno que inclúe os píxeles `a` e `b`, en calquera orde: o que se debuxa ao arrastrar o rato desde `a` ata `b`. |
| `width`, `height`, `size` e `area` | As medidas. Dun rectángulo invertido, 0. |
| `intersection` | Os píxeles que están nos dous. Se non hai ningún, `EMPTY`. |
| `union` | O rectángulo máis pequeno que contén os dous. Un rectángulo baleiro non engade nada. |
| `intersects` e `contains` | Se teñen píxeles en común, e se un está enteiro dentro do outro. Un rectángulo baleiro está dentro de calquera. |
| `border(thickness)` | Os catro lados dun bordo de `thickness` píxeles, por dentro: arriba, abaixo, esquerda e dereita. Non se solapan, e un lado queda baleiro se o rectángulo é demasiado pequeno para el. |

Algúns detalles:

```rust
const fn span(from: i32, to: i32) -> u32 {
    if to > from { (to as i64 - from as i64) as u32 } else { 0 }
}

const fn saturate(value: u32) -> i32 {
    if value > i32::MAX as u32 { i32::MAX } else { value as i32 }
}
```

- `span` calcula un ancho ou un alto. A resta faise en `i64`: a diferenza entre dous `i32` pode non caber nun `i32` (por exemplo, de -2 000 000 000 a 2 000 000 000), pero sempre cabe nun `u32`.
- `saturate` converte un tamaño nunha coordenada, quedando en `i32::MAX` se non cabe, en vez de dar un número negativo.
- `spanning` suma 1 co `saturating_add`, que queda no máximo en vez de desbordar.
- Moitas funcións son `const fn`, que se poden usar en constantes, como `Rect::EMPTY`.

**Probas.** Nove, unha por propiedade: que `right` e `bottom` non se inclúen, que os rectángulos invertidos están baleiros, `from_size` (tamén co tamaño máximo), `spanning` nas dúas ordes, a intersección e a unión (tamén con rectángulos baleiros), `contains`, que `at` e `offset` moven o rectángulo (tamén preto do máximo de `i32`), e que os lados de `border` cobren o bordo sen solaparse nin saír dun rectángulo estreito.

### `GridSize`

```rust
/// A size of the terminal grid, in cells: how many columns and rows of
/// characters fit in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridSize {
    pub columns: u16,
    pub rows: u16,
}

impl GridSize {
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }
}
```

Desde a subfase 1.4: o tamaño da grella do terminal, en celas, non en píxeles. É o tamaño que recibe a pseudoconsola ao creala e, desde a 1.9, cando cambia ([Código › meiga-win](meiga-win.md#cratesmeiga-winsrcconptyrs)), e, desde a 1.5, o do núcleo do terminal. Desde a 1.6, sae do tamaño da xanela ao arrancar, dividido polo da cela, e desde a 1.9 segue a xanela, sen a franxa da barra de desprazamento, cando cambia o seu tamaño ou o das celas, coa escala ou co zoom ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)).

- `u16`: ningunha grella chega a 65 535 columnas ou filas. Cada uso comproba o seu propio límite: o ConPTY, por exemplo, recibe as medidas nun `COORD`, de 16 bits con signo, así que acepta como moito 32 767, e polo menos 1.
- Deriva o mesmo ca os outros tipos do módulo, e os campos son públicos: calquera par de números é un tamaño, e quen o usa decide se lle serve.

### `GridPoint`

```rust
/// A cell of the terminal grid: its column and its row, from the top-left
/// cell, which is (0, 0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridPoint {
    pub column: u16,
    pub row: u16,
}

impl GridPoint {
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }
}
```

Desde a subfase 1.8: unha cela da grella do terminal, en celas e non en píxeles, coma `GridSize`. É a cela debaixo do punteiro do rato ([`cell_at`](#cratesmeiga-coresrcmousers)), a que levan os informes para os programas que piden o rato, e onde empeza e remata unha selección ([`select`](#cratesmeiga-coresrctermselectionrs)).

- A columna e a fila contan desde 0, na esquina superior esquerda. Os informes do rato contan desde 1: a conversión faise ao escribilos.
- A columna vai primeiro, coma o `x` de `Point` e as columnas de `GridSize`.
- `u16`, coma `GridSize`, e deriva o mesmo. Calquera par de números é unha cela: os métodos da selección aceptan tamén as de fóra da grella, e usan a máis próxima.

## `crates/meiga-core/src/damage.rs`

As zonas da xanela que cambiaron desde o último fotograma, o ***damage***. Cada fotograma compón e presenta só estas zonas, como manda o principio de debuxar só o que cambia ([ADR 0002](../adr/0002-windows-e-renderizado-cpu.md)).

```rust
pub const MAX_RECTS: usize = 8;
pub const MERGE_SLACK: u64 = 64 * 64;

pub struct Damage {
    rects: [Rect; MAX_RECTS],
    len: usize,
}
```

- Garda como moito `MAX_RECTS` rectángulos, nun *array* de tamaño fixo: engadir un nunca reserva memoria, como manda o principio de non reservar memoria no camiño quente.
- Cada rectángulo cópiase na pantalla por separado, e cada copia ten un custo fixo, ademais do dos seus píxeles. Por iso dous rectángulos próximos convén xuntalos nun só, aínda que inclúa algúns píxeles que non cambiaron. **`MERGE_SLACK`** é ese custo fixo, contado en píxeles: dous rectángulos xúntanse se o rectángulo que os contén ten como moito 64 × 64 píxeles máis ca os dous xuntos. O valor é unha estimación, para axustar con medicións.

```rust
pub fn add(&mut self, rect: Rect) {
    if rect.is_empty() {
        return;
    }
    let mut rect = rect;
    while let Some((index, extra)) = self.cheapest_merge(rect) {
        if extra > i128::from(MERGE_SLACK) && self.len < MAX_RECTS {
            break;
        }
        rect = rect.union(self.rects[index]);
        self.len -= 1;
        self.rects[index] = self.rects[self.len];
    }
    self.rects[self.len] = rect;
    self.len += 1;
}
```

O algoritmo para engadir un rectángulo:

1. Busca, entre os que xa garda, o que menos crece ao xuntalo co novo (`cheapest_merge`).
2. Se ese crecemento é como moito `MERGE_SLACK`, ou se xa non queda sitio, xúntaos: o novo pasa a ser a unión, e o vello sae da lista. Para sacalo sen mover os demais, o último ocupa o seu sitio.
3. Repite co rectángulo xuntado, que agora é maior e pode tocar outros.
4. Cando xa non convén xuntar máis, gárdao.

- **A garantía:** cada rectángulo engadido acaba **dentro** dun dos que se gardan. Un rectángulo nunca se corta, só se xunta con outros: o que cambiou presentarase sempre.
- `while let` repite mentres o patrón encaixe: mentres haxa algún rectángulo co que comparar.
- `extra_pixels` calcula os píxeles de máis: a área da unión menos as dos dous. É negativo cando se solapan, porque os píxeles comúns se copiarían dúas veces; por iso dous rectángulos que se solapan sempre se xuntan. A conta faise en `i128`, o tipo que cabe con calquera par de áreas de `u64`.
- `rects()` dá os rectángulos gardados (`&self.rects[..self.len]`), e `clear()` esquéceos despois de debuxalos.

**Probas.** Sete casos e unha proba aleatoria:

| Proba | Que comproba |
|---|---|
| `ignores_empty_rectangles` | Os rectángulos baleiros non se gardan. |
| `a_rectangle_inside_another_adds_nothing` | Un rectángulo dentro doutro, en calquera orde, deixa só o grande. |
| `rows_one_after_another_become_one_rectangle` | Tres filas seguidas, aínda que cheguen desordenadas, son un só rectángulo. |
| `distant_changes_stay_apart` | O cursor nunha esquina e un *spinner* na outra quedan separados: xuntalos copiaría media pantalla. |
| `close_changes_are_merged` e `overlapping_changes_are_merged` | Dous rectángulos próximos ou solapados xúntanse. |
| `clear_forgets_every_change` | Despois de `clear`, non queda nada. |
| `every_change_stays_covered_within_the_capacity` | 200 roldas de ata 40 rectángulos pseudoaleatorios: nunca hai máis de `MAX_RECTS`, ningún gardado está baleiro, e cada rectángulo engadido está dentro dun gardado. A secuencia é fixa (un xerador *xorshift* coa mesma semente), así que un fallo pódese repetir. |

## `crates/meiga-core/src/pacing.rs`

**Cando** debuxar o seguinte fotograma: como moito un por intervalo, e o primeiro cambio despois dunha pausa, no momento ([ADR 0005](../adr/0005-fios-e-fotogramas.md)).

```rust
pub const fn auto_interval(remote: bool) -> Duration {
    let fps = if remote { 30 } else { 60 };
    Duration::from_nanos(1_000_000_000 / fps)
}
```

O intervalo de `max_fps = "auto"`: 30 fotogramas por segundo (33,3 ms) nas sesións de Escritorio remoto e 60 (16,7 ms) en local. Decidiuse coas medicións da subfase 1.10 ([ADR 0017](../adr/0017-os-fotogramas-por-escritorio-remoto.md)): por Escritorio remoto, 30 aínda que a sesión permita 60. `Duration` é un tempo en segundos e nanosegundos, da biblioteca estándar.

```rust
pub enum Pace {
    Now,
    After(Duration),
    Scheduled,
}

pub struct FramePacer {
    last_frame: Option<Instant>,
    due: Option<Instant>,
}
```

- **`Pace`**, a resposta: debuxar agora (`Now`), despois dun tempo (`After`, co tempo que falta: hai que pór un temporizador) ou nada, porque xa hai un fotograma programado que mostrará tamén este cambio (`Scheduled`).
- **`FramePacer`** lembra cando foi o último fotograma (`last_frame`) e cando vence o programado, se o hai (`due`).

```rust
pub fn request(&mut self, now: Instant, interval: Duration) -> Pace {
    if let Some(due) = self.due {
        return if now >= due { Pace::Now } else { Pace::Scheduled };
    }
    let Some(due) = self.last_frame.and_then(|last| last.checked_add(interval)) else {
        return Pace::Now;
    };
    if now >= due {
        return Pace::Now;
    }
    self.due = Some(due);
    Pace::After(due - now)
}
```

Algo cambiou no instante `now`: cando se debuxa?

1. Se xa hai un fotograma programado, normalmente a resposta é `Scheduled`. Pero se xa pasou a súa hora (o temporizador vai tarde, por exemplo porque chegaban mensaxes sen parar e nunca lle tocou), `Now`: non se espera por el.
2. Se nunca houbo fotograma, `Now`.
3. Se xa pasou o intervalo desde o último, `Now`: a primeira tecla despois dunha pausa vese sen atraso.
4. Se non, prográmase para cando remate o intervalo: `After` co tempo que falta.

- `now` vén de fóra, en vez de ler o reloxo dentro: así as probas usan instantes inventados e dan sempre o mesmo resultado.
- `is_scheduled()` di se hai un fotograma programado, `cancel()` esquéceo (por exemplo, ao minimizar a xanela) e `drawn(now)` anota que se debuxou un fotograma en `now`.

**Probas.** Seis: o primeiro cambio debúxase no momento; un cambio dentro do intervalo espera o que falta (23 ms dun intervalo de 33 se pasaron 10), e os seguintes xa están programados; despois do intervalo, no momento; un temporizador que vai tarde non atrasa o fotograma; `drawn` e `cancel` rematan a espera; e `auto` dá 33 333 333 ns en remoto e 16 666 666 ns en local.

## `crates/meiga-core/src/text.rs`

O texto como celas dunha grella: que caracteres ocupan unha cela, cales dúas e cales ningunha ([Tecnoloxías › O ancho dos caracteres](../tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)).

```rust
pub enum CellChar {
    Blank,
    Narrow(char),
    WideLeft(char),
    WideRight(char),
}
```

O que mostra unha cela:

| Variante | Que é |
|---|---|
| `Blank` | Só o fondo, como un espazo. |
| `Narrow(c)` | Un carácter que ocupa esta cela. |
| `WideLeft(c)` | A metade esquerda dun carácter que ocupa esta cela e a seguinte. |
| `WideRight(c)` | A metade dereita dun carácter que ocupa a cela anterior e esta. |

- **Por que dúas metades.** Así cada cela se pode debuxar soa: cando só cambia unha das dúas, ou cando só unha está dentro da zona que cambiou, só se debuxa esa. Na caché de glifos, cada metade ten o seu *slot* ([Código › meiga-render](meiga-render.md#glyphcache)).
- Deriva `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq` e `Hash`, como `Rgb`. O valor por defecto, desde a subfase 1.5, é `Blank` (`#[default]`): o dunha cela baleira do terminal ([`Cell`](#cratesmeiga-coresrctermscreenrs)).

```rust
pub fn cells(text: &str) -> impl Iterator<Item = CellChar> + '_ {
    text.chars().flat_map(|c| {
        let (first, second) = match c.width() {
            Some(1) if c == ' ' => (Some(CellChar::Blank), None),
            Some(1) => (Some(CellChar::Narrow(c)), None),
            Some(2) => (Some(CellChar::WideLeft(c)), Some(CellChar::WideRight(c))),
            _ => (None, None),
        };
        first.into_iter().chain(second)
    })
}

pub fn width(text: &str) -> usize {
    cells(text).count()
}
```

- **`cells`** dá as celas dun texto, unha tras outra. O ancho de cada carácter é o que dá `unicode-width` (`c.width()`, do seu trait `UnicodeWidthChar`), o mesmo que usa `alacritty_terminal` para a grella do terminal:
  - o espazo é unha cela en branco;
  - un carácter de ancho 1, unha cela;
  - un de ancho 2, as súas dúas metades;
  - os de control (`None`) e os que non teñen ancho (`Some(0)`), como os acentos que se combinan coa letra anterior, ningunha. Xuntalos co carácter anterior, nun grafema, é traballo da subfase 8.1. As letras con acento xa compostas, como `á`, son un só carácter.
- `flat_map` converte cada carácter en cero, unha ou dúas celas. `first.into_iter().chain(second)` encadea dous `Option`, que se percorren como listas de cero ou un elementos, sen reservar memoria: quen chama decide onde gardar as celas, por exemplo nun `Vec` con `collect`.
- `impl Iterator<Item = CellChar> + '_`: devolve un iterador sen dicir o seu tipo exacto, que toma prestado o texto (`'_`).
- **`width`** conta as celas que ocupa un texto.

**Probas.** Tres: as letras, `ñ` e `€` ocupan unha cela, e o espazo é unha cela en branco; `中` e `😀` ocupan dúas cada un, e `日本語 abc`, dez; e os caracteres de control (o tabulador) e os que non teñen ancho (un acento que se combina e o espazo de ancho cero) non ocupan ningunha.

## `crates/meiga-core/src/shell.rs`

Desde a subfase 1.6: a liña de comandos dun programa. `meiga.exe` recibe o programa que ten que lanzar e os seus argumentos xa divididos (`meiga [opcións] [--] [programa [argumentos]]`), pero a sesión lánzao con `CreateProcessW`, que recibe un só texto: hai que xuntalos outra vez, de forma que o programa os divida igual ([Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows), [ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Úsao `Options::parse` ([Código › meiga › `options.rs`](meiga.md#cratesmeigasrcoptionsrs)).

Está en `meiga-core`, e non nun crate de Windows, aínda que as regras son as de Windows: só traballa con texto, sen chamar a ningunha API, así que se proba sen lanzar ningún programa. O módulo é para as shells e as súas liñas de comandos; polo de agora ten unha soa función.

```rust
pub fn command_line<S: AsRef<str>>(program: &str, args: &[S]) -> Option<String> {
    if program.contains('"') {
        return None;
    }
    let mut line = String::with_capacity(
        program.len() + args.iter().map(|arg| arg.as_ref().len() + 3).sum::<usize>() + 2,
    );
    if needs_quotes(program) || program.is_empty() {
        line.push('"');
        line.push_str(program);
        line.push('"');
    } else {
        line.push_str(program);
    }
    for arg in args {
        line.push(' ');
        push_argument(&mut line, arg.as_ref());
    }
    Some(line)
}

fn needs_quotes(arg: &str) -> bool {
    arg.contains([' ', '\t'])
}
```

**`command_line`** dá a liña de comandos de `program` con `args`, separados por espazos, ou `None` se o nome do programa ten comiñas.

- **O programa** non se le coma os argumentos. `CreateProcessW` busca o programa ata o primeiro espazo ou tabulador ou, se empeza por comiñas, ata as seguintes, sen ningunha barra invertida especial; e o *runtime* de C do programa le igual o seu primeiro argumento. Por iso o nome vai tal cal, sen escapar nada, e entre comiñas só se ten espazos ou tabuladores: `C:\Program Files\PowerShell\7\pwsh.exe` vai como `"C:\Program Files\PowerShell\7\pwsh.exe"`.
- **Por que `None` coas comiñas.** Un nome con comiñas non se pode escribir de forma que se lea igual: unhas comiñas no medio rematarían o nome antes de tempo, e `\"` non as escapa. Ademais, en Windows ningún arquivo pode ter comiñas no nome, así que ese programa non existe. Con `None`, quen chama dio claramente, en vez de arriscarse a lanzar outro programa: `Options::parse` convérteo no erro `The name of a program cannot have quotes`.
- **Un programa baleiro** vai entre comiñas, `""`: sen elas, a liña empezaría polo espazo de antes do primeiro argumento, e ese argumento pasaría a ser o programa. Con elas, o programa é un nome baleiro, que non se pode lanzar, e o terminal mostra o erro.
- `S: AsRef<str>`: os argumentos poden ser `String`, como os ten `Options`, ou `&str`, como nas probas, sen copialos.
- `with_capacity` reserva dunha vez o habitual: cada argumento cun espazo e dúas comiñas (3 bytes máis), e o programa coas súas (2). Só as barras e as comiñas que hai que escapar fan medrar o texto despois.
- `contains([' ', '\t'])`: `contains` acepta como patrón un *array* de caracteres, e busca calquera deles.

```rust
fn push_argument(line: &mut String, arg: &str) {
    let quote = arg.is_empty() || needs_quotes(arg);
    if quote {
        line.push('"');
    }
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                // The backslashes before it, and the quote, are escaped.
                line.extend(std::iter::repeat_n('\\', backslashes + 1));
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
        line.push(c);
    }
    if quote {
        // The closing quote must not be escaped by the backslashes.
        line.extend(std::iter::repeat_n('\\', backslashes));
        line.push('"');
    }
}
```

**Os argumentos** escríbense coas regras do *runtime* de C de Microsoft, coas que case todos os programas dividen a súa liña de comandos, e que tamén segue `CommandLineToArgvW`. Nelas, os espazos e os tabuladores separan os argumentos, salvo entre comiñas, e unha barra invertida só é especial xusto antes dunhas comiñas: `2n` barras e unhas comiñas son `n` barras e o principio ou o fin dun texto entre comiñas; `2n + 1` barras e unhas comiñas son `n` barras e unhas comiñas dentro do argumento. Así que `push_argument` escribe:

| Argumento | Na liña de comandos | Por que |
|---|---|---|
| `-NoProfile` | `-NoProfile` | Sen espazos, tabuladores nin comiñas, vai tal cal. |
| `C:\my scripts\a b.ps1` | `"C:\my scripts\a b.ps1"` | Os espazos separarían os argumentos: vai entre comiñas. As barras que non van antes dunhas comiñas quedan como están. |
| Un argumento baleiro | `""` | Sen as comiñas, non quedaría nada, e o argumento desaparecería. |
| `say "ola"` | `"say \"ola\""` | Unhas comiñas dentro do argumento escríbense `\"`. |
| `a\"b` | `a\\\"b` | As barras xusto antes dunhas comiñas dóbranse, e despois vai `\"`: tres barras e unhas comiñas son unha barra e unhas comiñas. |
| `a\\b` | `a\\b` | Sen comiñas despois, as barras quedan como están. |
| `C:\a b\` | `"C:\a b\\"` | Ao final dun argumento entre comiñas, as barras dóbranse: se non, a última escaparía as comiñas que o pechan, e o argumento seguiría. |
| `C:\a\` | `C:\a\` | Sen comiñas ao final, a barra queda como está. |

- `backslashes` conta as barras seguidas que acaba de copiar. Se despois veñen unhas comiñas, engade outras tantas e unha máis, a que escapa as comiñas; con calquera outro carácter, volta a 0. Ao final dun argumento entre comiñas, engade outras tantas antes das comiñas que o pechan.
- As comiñas escápanse tamén nun argumento sen espazos, que vai sen comiñas: `a"b` vai como `a\"b`.
- `std::iter::repeat_n('\\', n)` dá o mesmo carácter `n` veces, e `extend` engádeos todos ao texto.
- **`cmd.exe` é a excepción**: le o resto da súa liña coas súas propias regras, e non coas do *runtime* de C ([Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows)).

**Probas.** Catro:

| Proba | Que comproba |
|---|---|
| `simple_arguments_stay_as_they_are` | `pwsh` só, e `pwsh -NoProfile -File C:\perf\spinner.ps1`, coas barras como están. |
| `spaces_tabs_and_empty_arguments_go_in_quotes` | O programa `C:\Program Files\PowerShell\7\pwsh.exe` vai entre comiñas, e tamén un argumento con espazos, un baleiro e un cun tabulador. |
| `quotes_and_the_backslashes_before_them_are_escaped` | As comiñas dentro dun argumento, as barras antes delas, as que non van antes de ningunhas, e as do final dun argumento con e sen comiñas: os exemplos da táboa. |
| `a_program_cannot_have_quotes` | Un programa con comiñas dá `None`, e un baleiro, `""`. |

## `crates/meiga-core/src/input.rs`

Desde a subfase 1.7: as teclas que meiga lle envía ao programa ([ADR 0014](../adr/0014-o-teclado.md)). Windows describe cada tecla que baixa ou sobe cun evento, e o teclado da xanela ([Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)) dálle a meiga eses eventos. Este módulo convérteos nos bytes que le o programa: o evento enteiro en **win32-input-mode**, se o ConPTY o activou, ou o que enviaría un terminal VT ([Tecnoloxías › As teclas no terminal](../tecnoloxias.md#42-as-teclas-no-terminal-win32-input-mode-e-vt)). Está en `meiga-core` porque só traballa con números e bytes: as súas probas non necesitan ningún teclado.

### `ControlKeys`

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ControlKeys(u16);

impl ControlKeys {
    pub const RIGHT_ALT: Self = Self(0x0001);
    pub const LEFT_ALT: Self = Self(0x0002);
    pub const RIGHT_CTRL: Self = Self(0x0004);
    pub const LEFT_CTRL: Self = Self(0x0008);
    pub const SHIFT: Self = Self(0x0010);
    pub const NUM_LOCK: Self = Self(0x0020);
    pub const SCROLL_LOCK: Self = Self(0x0040);
    pub const CAPS_LOCK: Self = Self(0x0080);
    pub const ENHANCED: Self = Self(0x0100);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn alt(self) -> bool {
        self.0 & (Self::LEFT_ALT.0 | Self::RIGHT_ALT.0) != 0
    }

    pub const fn ctrl(self) -> bool {
        self.0 & (Self::LEFT_CTRL.0 | Self::RIGHT_CTRL.0) != 0
    }

    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    const fn alt_gr(self) -> bool {
        let all = Self::LEFT_ALT.0 | Self::RIGHT_ALT.0 | Self::LEFT_CTRL.0 | Self::RIGHT_CTRL.0;
        self.0 & all == Self::LEFT_CTRL.0 | Self::RIGHT_ALT.0
    }
}

impl BitOr for ControlKeys {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for ControlKeys {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}
```

- As teclas modificadoras e as de bloqueo dun evento, cos bits que usa Windows nos eventos da consola (`dwControlKeyState`). Gardar os mesmos bits fai que o win32-input-mode os envíe tal cal, con `bits`.
- `ENHANCED` marca unha **tecla estendida**, das que non tiñan os primeiros teclados do PC: as frechas, Insert, Supr, Inicio, Fin, RePáx e AvPáx de fóra do teclado numérico, o Ctrl e o Alt dereitos, e o Intro e a `/` do teclado numérico ([Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows)).
- `alt` e `ctrl` son certos con calquera dos dous, o esquerdo ou o dereito; `contains`, se están todas as de outro valor. `BitOr` e `BitOrAssign` permiten xuntalas con `|` e `|=`.
- **`alt_gr`**, privada, di se o Ctrl e o Alt premidos son só os de AltGr: Windows dá AltGr coma o Ctrl esquerdo e o Alt dereito. Se ademais está o Ctrl dereito ou o Alt esquerdo, hai un Ctrl ou un Alt de verdade. Úsaa [`push_vt_character`](#keyencoder).
- É un *newtype* sobre `u16`: o número non se confunde con outro, e os bits só se nomean coas constantes. As funcións son `const`, así que tamén se poden usar para definir outras constantes, coma nas probas.

### `KeyEvent`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: u16,
    pub scan: u16,
    pub character: u16,
    pub down: bool,
    pub controls: ControlKeys,
    pub repeat: u16,
}
```

Unha tecla que baixou ou subiu, coma a describe Windows:

| Campo | Que é |
|---|---|
| `key` | O código de tecla virtual (`VK_…`): que tecla é na distribución do teclado. 0 para un carácter sen tecla, coma os dun IME. |
| `scan` | O código de exploración: onde está a tecla no teclado. |
| `character` | O carácter que escribe, unha unidade UTF-16, ou 0. Un carácter de fóra do plano básico, coma a maioría dos emoji, chega en dous eventos, cada un cunha metade (*surrogate*). |
| `down` | Se baixou. Cada tecla que baixa sobe despois, noutro evento. |
| `controls` | As teclas modificadoras e de bloqueo. |
| `repeat` | Cantas veces baixou: máis dunha se o teclado a repetiu máis rápido do que a xanela a colleu. |

É `Copy`, de 12 bytes: vai por valor no evento da xanela, `Event::Key` ([Código › meiga-win › `event.rs`](meiga-win.md#cratesmeiga-winsrceventrs)). Desde a subfase 1.8, a roda do rato tamén crea eventos de tecla: as frechas, na pantalla alternativa ([`arrow_keys`](#cratesmeiga-coresrcmousers)).

### `KeyEncoder`

```rust
#[derive(Debug, Default)]
pub struct KeyEncoder {
    high_surrogate: Option<u16>,
}

impl KeyEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
        if modes.win32_input {
            // The ConPTY joins the two halves of a character by itself.
            self.high_surrogate = None;
            push_win32(event, out);
            return;
        }
        // A VT terminal only sends something when a key goes down, and the
        // character that Alt brings when it goes up: the one of the code
        // typed with it on the numeric keypad. Only with Num Lock on, as
        // Windows Terminal.
        if !event.down {
            if event.key == vk::MENU
                && event.controls.contains(ControlKeys::NUM_LOCK)
                && let Some(character) = char::from_u32(u32::from(event.character))
                && character != '\0'
            {
                push_utf8(out, character);
            }
            return;
        }
        let start = out.len();
        self.push_vt(event, modes, out);
        let end = out.len();
        for _ in 1..event.repeat {
            out.extend_from_within(start..end);
        }
    }
}
```

**`encode`** engade a `out` o que le o programa para `event`, cos modos que activou ([`Modes`](#changes-e-modes)). Quen chama reutiliza o mesmo `Vec`, así que codificar unha tecla non reserva memoria.

- **Co win32-input-mode,** o evento vai enteiro, tamén ao subir e tamén as teclas modificadoras soas ([`push_win32`](#os-axudantes-das-secuencias)). As dúas metades dun carácter van en dous eventos, e o ConPTY xúntaas.
- **Sen el,** só envían algo as teclas que baixan, coma nun terminal VT. Se o teclado repetiu a tecla, `extend_from_within` copia o que escribiu a primeira vez as veces que falten, sen calculalo de novo.
- **A excepción é Alt cando sobe,** se trae un carácter: o do código que se escribiu con el no teclado numérico, coma Alt+0225, que escribe á. Windows dálle ese carácter á tecla Alt cando sobe, e as teclas do teclado numérico non escriben nada ([Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)). Vai tal cal, sen o ESC de Alt, e só co Bloq Num activado, coma no Windows Terminal. `char::from_u32` descarta unha metade dun carácter, que un código así non pode dar.
- **`high_surrogate`**, o único estado: a primeira metade dun carácter, que espera pola segunda en VT. Cambiar ao win32-input-mode descártaa.

```rust
fn push_vt(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
    // The first half of a character only waits for the second one.
    let high_surrogate = self.high_surrogate.take();
    // A key that brings a character that is not its own sends the
    // character, not its sequence.
    if !own_character(event.key, event.character) {
        self.push_vt_character(event, high_surrogate, out);
        return;
    }
    let controls = event.controls;
    let (alt, ctrl) = (controls.alt(), controls.ctrl());
    // The parameter of the modifiers in the sequences of xterm: 1, plus
    // 1 for Shift, 2 for Alt and 4 for Ctrl.
    let modifiers = 1 + u16::from(controls.shift()) + 2 * u16::from(alt) + 4 * u16::from(ctrl);
    // Alt adds ESC before the keys that have no parameter for it.
    let with_alt = |out: &mut Vec<u8>, bytes: &[u8]| {
        if alt {
            out.push(ESC);
        }
        out.extend_from_slice(bytes);
    };
    match event.key {
        vk::UP => push_cursor_key(out, b'A', modifiers, modes.app_cursor),
        vk::DOWN => push_cursor_key(out, b'B', modifiers, modes.app_cursor),
        vk::RIGHT => push_cursor_key(out, b'C', modifiers, modes.app_cursor),
        vk::LEFT => push_cursor_key(out, b'D', modifiers, modes.app_cursor),
        vk::HOME => push_cursor_key(out, b'H', modifiers, modes.app_cursor),
        vk::END => push_cursor_key(out, b'F', modifiers, modes.app_cursor),
        vk::INSERT => push_tilde_key(out, 2, modifiers),
        vk::DELETE => push_tilde_key(out, 3, modifiers),
        vk::PRIOR => push_tilde_key(out, 5, modifiers),
        vk::NEXT => push_tilde_key(out, 6, modifiers),
        // F1 to F4 always go as in the application cursor mode.
        vk::F1..=vk::F4 => {
            let last = b'P' + (event.key - vk::F1) as u8;
            push_cursor_key(out, last, modifiers, true);
        }
        vk::F5..=vk::F12 => {
            let number = F5_TO_F12[usize::from(event.key - vk::F5)];
            push_tilde_key(out, number, modifiers);
        }
        // Backspace sends DEL, and Ctrl+Backspace, BS.
        vk::BACK => with_alt(out, if ctrl { b"\x08" } else { b"\x7f" }),
        vk::TAB if controls.shift() => out.extend_from_slice(b"\x1b[Z"),
        vk::TAB => with_alt(out, b"\t"),
        vk::RETURN => with_alt(
            out,
            if ctrl {
                b"\n"
            } else if modes.line_feed_new_line {
                b"\r\n"
            } else {
                b"\r"
            },
        ),
        vk::ESCAPE => with_alt(out, &[ESC]),
        vk::PAUSE => out.push(0x1a),
        // Ctrl+Break.
        vk::CANCEL => out.push(0x03),
        _ => self.push_vt_character(event, high_surrogate, out),
    }
}
```

**`push_vt`** escribe o que envía o xterm cando a tecla baixa unha vez:

| Tecla | Envía |
|---|---|
| Frechas, Inicio e Fin | `ESC [ A`–`D`, `H` e `F`, ou `ESC O …` no modo aplicación (`app_cursor`). |
| Insert, Supr, RePáx e AvPáx | `ESC [ 2 ~`, `3 ~`, `5 ~` e `6 ~`. |
| F1–F4 | `ESC O P`–`S`, sempre coma no modo aplicación. |
| F5–F12 | `ESC [ n ~`, cos números de `F5_TO_F12`: 15, 17, 18, 19, 20, 21, 23 e 24. |
| Retroceso | `DEL` (`0x7F`); con Ctrl, `BS` (`0x08`). Windows escribe ao revés, `BS` sen Ctrl e `DEL` con el, pero os terminais VT, coma o Windows Terminal, envían `DEL`. |
| Tab e Maiús+Tab | `HT` e `ESC [ Z`. |
| Intro | `CR`; co modo *LNM* (`line_feed_new_line`), `CR LF`; con Ctrl, `LF`. |
| Esc | `ESC`. |
| Pausa | `SUB` (`0x1A`), o Ctrl+Z, coma no Windows Terminal: Pausa non ten secuencia VT. |
| Ctrl+Inter | `ETX` (`0x03`), coma Ctrl+C. |
| As demais | O seu carácter, en `push_vt_character`. |

- **`modifiers`** é o parámetro das modificadoras do xterm: 1, máis 1 por Maiús, 2 por Alt e 4 por Ctrl. Con algunha, as frechas e as teclas con `~` lévano: Ctrl+frecha esquerda é `ESC [ 1 ; 5 D`.
- **`with_alt`** pon un ESC diante, coma fan os terminais VT con Alt, nas teclas que non teñen o parámetro.
- `high_surrogate` cóllese ao principio con `take`: se esta tecla non é a segunda metade dun carácter, a primeira pérdese, porque xa non ten con que xuntarse.
- **Unha tecla que trae un carácter que non é seu** envía o carácter, non a súa secuencia ([`own_character`](#os-axudantes-das-secuencias)). Pasa cando unha tecla morta non se xunta coa seguinte: Windows escribe os dous caracteres coa segunda tecla, o primeiro chega con ela e o segundo chega despois, só, coa tecla do seu código de exploración. Despois de ´, Intro trae o ´, e o `CR` chega só: sen esta comprobación irían dous `CR`. E se a seguinte é o 7 do teclado numérico, o 7 chega coa tecla Inicio, e iría `ESC [ H`.

```rust
fn push_vt_character(
    &mut self,
    event: &KeyEvent,
    high_surrogate: Option<u16>,
    out: &mut Vec<u8>,
) {
    let (alt, ctrl) = (event.controls.alt(), event.controls.ctrl());
    let unit = event.character;
    // Windows writes a space for Ctrl+Space, and nothing for some keys
    // with Ctrl, such as Ctrl+2: a VT terminal sends control codes.
    if ctrl && (unit == 0 || unit == u16::from(b' ')) {
        // But AltGr writes characters: with a key that writes nothing
        // with it, as a dead key, it sends nothing.
        if unit == 0 && event.controls.alt_gr() {
            return;
        }
        if let Some(code) = control_code(event.key) {
            if alt {
                out.push(ESC);
            }
            out.push(code);
        }
        return;
    }
    let character = match unit {
        0 => return,
        0xd800..=0xdbff => {
            self.high_surrogate = Some(unit);
            return;
        }
        0xdc00..=0xdfff => {
            // A second half without its first one is lost.
            let Some(high) = high_surrogate else {
                return;
            };
            let code = 0x10000 + (u32::from(high - 0xd800) << 10) + u32::from(unit - 0xdc00);
            char::from_u32(code)
        }
        _ => char::from_u32(u32::from(unit)),
    };
    let Some(character) = character else {
        return;
    };
    // AltGr writes its characters with Ctrl and Alt down: they go as
    // they are. With Alt, a VT terminal sends ESC first.
    let alt_gr = ctrl && alt && character > ' ' && character != '\x7f';
    if alt && !alt_gr {
        out.push(ESC);
    }
    push_utf8(out, character);
}
```

**`push_vt_character`** envía o carácter da tecla en UTF-8:

- **Ctrl sen carácter.** Windows dá o código de control de Ctrl cunha letra (Ctrl+C escribe `0x03`), pero nada para Ctrl+2 e un espazo para Ctrl+Espazo. Para esas, [`control_code`](#os-axudantes-das-secuencias) dá o que envía un terminal VT, e se a tecla non ten ningún, non se envía nada.
- **AltGr sen carácter** non envía nada. AltGr é o Ctrl esquerdo e o Alt dereito ([`alt_gr`](#controlkeys)), pero serve para escribir caracteres. Unha tecla que con AltGr non escribe nada é unha tecla morta, coma o ~ de AltGr+4 no teclado español, cuxo carácter vai coa tecla seguinte, ou unha sen carácter con AltGr, coma a A. Ctrl+Alt co Alt esquerdo segue enviando os códigos de control: Ctrl+Alt+A é `ESC` e `0x01`.
- **As dúas metades.** A primeira (`0xD800`–`0xDBFF`) gárdase; a segunda (`0xDC00`–`0xDFFF`) xúntase con ela no carácter enteiro. Unha segunda metade sen a primeira, ou unha primeira que non vai seguida da segunda, pérdese.
- **AltGr** escribe os seus caracteres con Ctrl e Alt premidos ([Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows)): un carácter imprimíbel con Ctrl e Alt é de AltGr e vai tal cal. Con Alt só, vai un ESC diante: Alt+f é `ESC f`.
- [`push_utf8`](#os-axudantes-das-secuencias) escribe o carácter en UTF-8.

### Os eventos de foco

```rust
pub fn focus_sequence(focused: bool, modes: &Modes) -> &'static [u8] {
    match (modes.focus_events, focused) {
        (false, _) => b"",
        (true, true) => b"\x1b[I",
        (true, false) => b"\x1b[O",
    }
}
```

**`focus_sequence`** di que enviarlle ao programa cando a xanela gaña ou perde o foco: `ESC [ I` ou `ESC [ O` se o pediu (`?1004`, que segue `alacritty_terminal`), e nada se non. Úsaa a vista do terminal ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

### Os axudantes das secuencias

```rust
fn push_win32(event: &KeyEvent, out: &mut Vec<u8>) {
    out.extend_from_slice(b"\x1b[");
    for number in [
        event.key,
        event.scan,
        event.character,
        u16::from(event.down),
        event.controls.bits(),
    ] {
        push_decimal(out, number);
        out.push(b';');
    }
    // The ConPTY stores a 0 as it is, and a program could lose the key.
    push_decimal(out, event.repeat.max(1));
    out.push(b'_');
}

fn push_cursor_key(out: &mut Vec<u8>, last: u8, modifiers: u16, application: bool) {
    if modifiers > 1 {
        out.extend_from_slice(b"\x1b[1;");
        push_decimal(out, modifiers);
    } else if application {
        out.extend_from_slice(b"\x1bO");
    } else {
        out.extend_from_slice(b"\x1b[");
    }
    out.push(last);
}

fn push_tilde_key(out: &mut Vec<u8>, number: u16, modifiers: u16) {
    out.extend_from_slice(b"\x1b[");
    push_decimal(out, number);
    if modifiers > 1 {
        out.push(b';');
        push_decimal(out, modifiers);
    }
    out.push(b'~');
}
```

- **`push_win32`** escribe o evento coma `ESC [ Vk ; Sc ; Uc ; Kd ; Cs ; Rc _`, sempre cos seis números, coma o Windows Terminal. As repeticións van polo menos a 1: o ConPTY garda un 0 tal cal, e un programa que conta as repeticións non vería a tecla.
- **`push_cursor_key`**, para as frechas, Inicio, Fin e F1–F4: `ESC [ X`, ou `ESC O X` no modo aplicación; con modificadoras, `ESC [ 1 ; m X`, tamén no modo aplicación.
- **`push_tilde_key`**, para Insert, Supr, RePáx, AvPáx e F5–F12: `ESC [ n ~`, e `ESC [ n ; m ~` con modificadoras.

```rust
fn control_code(key: u16) -> Option<u8> {
    Some(match key {
        // Ctrl+Space and Ctrl+2, which is Ctrl+@: NUL.
        vk::SPACE | 0x32 => 0x00,
        // Ctrl+3 to Ctrl+7, which are Ctrl+[, \, ], ^ and _: ESC to US.
        0x33..=0x37 => (key - 0x33) as u8 + ESC,
        // Ctrl+8: DEL.
        0x38 => 0x7f,
        // Ctrl+A to Ctrl+Z, when the layout gives no character with them,
        // as with Ctrl+Alt: 1 to 26.
        0x41..=0x5a => (key - 0x40) as u8,
        _ => return None,
    })
}

pub(crate) fn push_decimal(out: &mut Vec<u8>, value: u16) {
    let mut digits = [0; 5];
    let mut start = digits.len();
    let mut rest = value;
    loop {
        start -= 1;
        digits[start] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    out.extend_from_slice(&digits[start..]);
}
```

- **`control_code`** dá o código de control de Ctrl cunha tecla que non o ten en Windows: Ctrl+Espazo e Ctrl+2, `NUL`; Ctrl+3 a Ctrl+7, de `ESC` a `US` (son Ctrl+[, \, ], ^ e _ no teclado estadounidense); Ctrl+8, `DEL`; e Ctrl+A–Z, do 1 ao 26, para cando a distribución non dá ningún, coma con Ctrl+Alt.
- **`push_decimal`** escribe as cifras de atrás cara a diante nun *array* de 5, as que ten como moito un `u16`, sen `format!`, que reservaría un texto. Desde a subfase 1.8 é `pub(crate)`, visíbel en todo o crate pero non fóra del: tamén escribe os números dos informes do rato en SGR ([`mouse.rs`](#cratesmeiga-coresrcmousers)).
- O módulo **`vk`** ten os códigos de tecla virtual que a codificación VT trata á parte, e `F5_TO_F12`, os números das teclas de función, que saltan algúns por razóns históricas. `meiga-core` non depende de `windows-sys`, así que define os seus.

```rust
fn own_character(key: u16, unit: u16) -> bool {
    match key {
        vk::BACK => matches!(unit, 0 | 0x08 | 0x7f),
        vk::TAB => matches!(unit, 0 | 0x09),
        vk::RETURN => matches!(unit, 0 | 0x0a | 0x0d),
        vk::ESCAPE => matches!(unit, 0 | 0x1b),
        vk::CANCEL => matches!(unit, 0 | 0x03),
        vk::PAUSE | vk::PRIOR..=vk::DOWN | vk::INSERT | vk::DELETE | vk::F1..=vk::F12 => unit == 0,
        _ => true,
    }
}

fn push_utf8(out: &mut Vec<u8>, character: char) {
    let mut utf8 = [0; 4];
    out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
}
```

- **`own_character`** di se o carácter dun evento é da súa tecla, ou se non trae ningún: só entón envían a súa secuencia as teclas que a teñen ([`push_vt`](#keyencoder)). Retroceso escribe `BS`, e con Ctrl `DEL`; Tab, `HT`; Intro, `CR`, e con Ctrl `LF`; Esc, `ESC`; e Ctrl+Inter, `ETX`. As frechas, Inicio, Fin, Insert, Supr, RePáx, AvPáx, Pausa e as teclas de función non escriben ningún. As outras teclas non teñen secuencia, e envían sempre o seu carácter.
- **`push_utf8`** escribe o carácter en UTF-8 nun *array* de 4 bytes, sen reservar memoria. Úsano `encode`, para o carácter de Alt cando sobe, e `push_vt_character`.

**Probas.** Doce:

| Proba | Que comproba |
|---|---|
| `win32_input_mode_sends_the_whole_key_events` | Eventos que enviou o Windows Terminal co teclado español: a A ao baixar e ao subir, AltGr+2 (@), Maiús+Intro, a frecha esquerda, unha tecla morta e Maiús só; as repeticións, e un 0 que vai coma 1; e os números máis grandes. |
| `vt_sends_the_characters_in_utf8_when_keys_go_down` | a, ñ e €, en UTF-8; unha tecla morta non envía nada, e a seguinte, á; nin as teclas que soben nin Maiús só envían nada. |
| `vt_joins_the_two_halves_of_a_character` | 😀 en dous eventos dá os seus 4 bytes, tamén cos eventos de subir no medio e con Alt; unha metade soa pérdese. |
| `vt_sends_the_character_of_an_alt_code_when_alt_goes_up` | Alt+0225: Alt e as teclas do teclado numérico non envían nada ao baixar, e Alt envía á ao subir; sen Bloq Num, nada. |
| `vt_cursor_keys_follow_the_application_mode_and_the_modifiers` | As frechas, Inicio e Fin, co modo aplicación e coas modificadoras. |
| `vt_editing_and_function_keys_send_their_sequences` | Insert, Supr, RePáx, AvPáx, F1, F4, F5, F10 e F12, con e sen modificadoras. |
| `vt_backspace_tab_enter_and_escape_send_control_codes` | Retroceso, Tab, Intro (tamén co modo *LNM*), Esc, Pausa e Ctrl+Inter, con Maiús, Ctrl e Alt. |
| `vt_keys_with_a_character_that_is_not_theirs_send_the_character` | Despois de ´, Intro, que trae o ´, e o `CR` que chega só envían `´` e `CR`; con Retroceso, `´` e `DEL`; e co 7 do teclado numérico, que chega coa tecla Inicio, `´7`. |
| `vt_ctrl_sends_control_codes` | Ctrl+C, Ctrl+Espazo, Ctrl+2, Ctrl+3, Ctrl+7, Ctrl+8 e Ctrl+Alt+A, tamén con AltGr e o Ctrl dereito; unha tecla sen código de control non envía nada. |
| `vt_alt_sends_escape_first_but_altgr_does_not` | Alt+a é `ESC a`; AltGr+2 e Ctrl+Alt+2 envían `@`, e o `[` de AltGr, `[`, sen ESC; e o ~ de AltGr+4, unha tecla morta, non envía nada ata a tecla seguinte. |
| `vt_repeats_the_keys_that_repeated` | Unha tecla repetida tres veces envía tres veces o seu, tamén unha frecha; con 0 repeticións, unha. |
| `focus_events_only_go_if_the_program_asks_for_them` | `ESC [ I` e `ESC [ O` co modo, e nada sen el. |

## `crates/meiga-core/src/mouse.rs`

Desde a subfase 1.8: o rato, coma o conta meiga e coma o recibe un programa que o pide ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). A xanela dá os eventos do rato en píxeles, cos botóns e as teclas modificadoras ([Código › meiga-win › `event.rs`](meiga-win.md#cratesmeiga-winsrceventrs)), e o rato da vista decide se seleccionan texto ou van ao programa ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)). Este módulo ten o que non depende de Windows: a cela debaixo do punteiro, os clics seguidos, as partes da roda e, desde a subfase 1.9, as filas do historial que move ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)), e os informes, no formato que pediu o programa. Coma `input.rs`, só traballa con números e bytes: as súas probas non necesitan ningún rato.

**Os informes e o ConPTY.** Un programa pide os clics (`?1000`), tamén os movementos cun botón premido (`?1002`) ou todos os movementos (`?1003`), e o formato dos informes: o do primeiro xterm, o mesmo en UTF-8 (`?1005`) ou SGR (`?1006`) ([Tecnoloxías › O rato nos programas](../tecnoloxias.md#45-o-rato-nos-programas-modos-e-formatos)). O ConPTY pásalles os informes tal cal aos programas que len a entrada coma un terminal, coma os de WSL: por iso meiga os escribe no formato que pediu o programa. Para os programas da consola de Windows, o ConPTY pide el mesmo SGR, e converte os informes en eventos do rato de Windows.

```rust
pub const WHEEL_DELTA: i32 = 120;

const MAX_X10: u16 = 127 - 33;

const MAX_UTF8: u16 = 0x7ff - 33;
```

- **`WHEEL_DELTA`**: o xiro que Windows conta coma unha parte enteira (*notch*) da roda: 120. As mensaxes da roda dan o xiro en 120 avos dunha parte. Úsano [`WheelTurns` e, desde a subfase 1.9, `WheelRows`](#a-roda-wheelturns-wheelrows-e-arrow_keys).
- **`MAX_X10`** e **`MAX_UTF8`**: a columna e a fila máis grandes, desde 0, que caben nun informe no formato do primeiro xterm e no de UTF-8 ([`report`](#os-informes-report)).

### `Button`, `Modifiers`, `Wheel` e `MouseAction`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wheel {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseAction {
    Press(Button),
    Release(Button),
    Move(Option<Button>),
    Wheel(Wheel),
}
```

| Tipo | Que é |
|---|---|
| `Button` | Un dos tres botóns: o esquerdo, o do medio e o dereito. |
| `Modifiers` | As teclas modificadoras que estaban premidas co evento: Maiús, Alt e Ctrl, sen distinguir a esquerda da dereita, porque os informes non o fan. As mensaxes do rato de Windows levan Maiús e Ctrl, e a xanela engade Alt ([Tecnoloxías › O rato en Windows](../tecnoloxias.md#44-o-rato-en-windows)). |
| `Wheel` | Cara a onde vai unha parte da roda: adiante é `Up`, e a inclinación cara á dereita, `Right`. |
| `MouseAction` | O que fixo o rato, coma o recibe un programa: premer ou soltar un botón, moverse a outra cela, cun botón premido ou sen ningún (`Move(None)`), ou xirar a roda unha parte. |

- `Button` e `Modifiers` están en `meiga-core`, e non en `meiga-win`, porque tamén os levan os eventos da xanela (`Event::MouseDown`, `MouseUp`, `MouseMoved` e `Wheel`): a xanela e os informes falan dos mesmos botóns, sen conversións.
- Todos son `Copy`: uns poucos bytes, que van por valor.

### Que pediu o programa: `is_tracked` e `wants`

```rust
pub const fn is_tracked(modes: &Modes) -> bool {
    modes.mouse_clicks || modes.mouse_drag || modes.mouse_motion
}

pub const fn wants(action: MouseAction, modes: &Modes) -> bool {
    match action {
        MouseAction::Press(_) | MouseAction::Release(_) | MouseAction::Wheel(_) => {
            is_tracked(modes)
        }
        MouseAction::Move(Some(_)) => modes.mouse_drag || modes.mouse_motion,
        MouseAction::Move(None) => modes.mouse_motion,
    }
}
```

- **`is_tracked`** di se o programa pediu o rato, con calquera dos tres modos de [`Modes`](#changes-e-modes): `mouse_clicks` (`?1000`), `mouse_drag` (`?1002`) ou `mouse_motion` (`?1003`). Mentres tanto, os clics van ao programa en vez de seleccionar texto, salvo con Maiús, e o punteiro é a frecha.
- **`wants`** di se o programa pediu `action`: os clics, as solturas e a roda, con calquera dos tres; os movementos cun botón premido, con `?1002` ou `?1003`; e os movementos sen ningún, só con `?1003`. `alacritty_terminal` só deixa activo un dos tres, o último que se pediu: `?1003` non activa tamén `mouse_drag`, e por iso os movementos cun botón miran os dous.

### Os informes: `report`

```rust
pub fn report(
    action: MouseAction,
    at: GridPoint,
    modifiers: Modifiers,
    modes: &Modes,
    out: &mut Vec<u8>,
) -> bool {
    if !wants(action, modes) {
        return false;
    }
    let button = |button: Button| match button {
        Button::Left => 0,
        Button::Middle => 1,
        Button::Right => 2,
    };
    let mut code: u8 = match action {
        MouseAction::Press(pressed) => button(pressed),
        // Without SGR, a release does not say which button it was.
        MouseAction::Release(released) if modes.sgr_mouse => button(released),
        MouseAction::Release(_) => 3,
        MouseAction::Move(down) => 32 + down.map_or(3, button),
        MouseAction::Wheel(Wheel::Up) => 64,
        MouseAction::Wheel(Wheel::Down) => 65,
        MouseAction::Wheel(Wheel::Left) => 66,
        MouseAction::Wheel(Wheel::Right) => 67,
    };
    for (down, value) in [
        (modifiers.shift, 4),
        (modifiers.alt, 8),
        (modifiers.ctrl, 16),
    ] {
        if down {
            code += value;
        }
    }
    if modes.sgr_mouse {
        out.extend_from_slice(b"\x1b[<");
        push_decimal(out, u16::from(code));
        out.push(b';');
        push_decimal(out, at.column.saturating_add(1));
        out.push(b';');
        push_decimal(out, at.row.saturating_add(1));
        out.push(if matches!(action, MouseAction::Release(_)) {
            b'm'
        } else {
            b'M'
        });
        return true;
    }
    let max = if modes.utf8_mouse { MAX_UTF8 } else { MAX_X10 };
    if at.column > max || at.row > max {
        return false;
    }
    out.extend_from_slice(b"\x1b[M");
    // At most 67 + 4 + 8 + 16 + 32 = 127: it is always one byte.
    out.push(32 + code);
    for value in [at.column, at.row] {
        let value = u32::from(value) + 33;
        match char::from_u32(value) {
            Some(character) if value >= 0x80 => {
                let mut utf8 = [0; 4];
                out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
            }
            // Up to 127, the byte, in both formats.
            _ => out.push(value as u8),
        }
    }
    true
}
```

**`report`** engade a `out` o informe de `action` na cela `at`, coas teclas `modifiers`, no formato que pediu o programa, e di se o engadiu: non o fai se o programa non pediu `action`, nin se a cela non cabe no formato.

1. **O número** do que fixo o rato, o mesmo nos tres formatos ([Tecnoloxías › O rato nos programas](../tecnoloxias.md#45-o-rato-nos-programas-modos-e-formatos)): premer o botón esquerdo, o do medio ou o dereito, 0, 1 e 2; soltalo, o botón en SGR, e 3 nos outros formatos, que non din cal; moverse, 32 máis o botón, ou 35 sen ningún (`map_or(3, button)`); e a roda, de 64 a 67. Maiús suma 4, Alt 8 e Ctrl 16.
2. **En SGR**: `ESC [ < número ; columna ; fila M`, en decimal, coa columna e a fila desde 1, e `m` en vez de `M` ao soltar. Non ten límite: `saturating_add` só evita desbordar na columna ou na fila 65 535.
3. **Nos outros dous formatos**: `ESC [ M` e tres valores, cada un con 32 máis, para que non sexan caracteres de control: o número, a columna e a fila. A columna e a fila contan desde 1, así que levan 33 máis ca a cela. Ata 127, cada valor é un byte; desde 128, só no formato de UTF-8, un carácter en UTF-8, de dous bytes.

- **Os límites.** O primeiro xterm escribía cada valor nun byte, ata 255: a columna 223. Pero o ConPTY le a entrada coma UTF-8, e un byte solto desde 128 chegaría ao programa coma outro carácter. Así que, nese formato, meiga só escribe os valores ata 127: `MAX_X10`, 127 − 33 = 94, as 95 primeiras columnas e filas. No de UTF-8, os caracteres de ata dous bytes chegan ata `0x7FF`: `MAX_UTF8`, 0x7FF − 33 = 2 014, as 2 015 primeiras. Fóra delas, o informe non se envía ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).
- **O número sempre é un byte**: como moito 67 da roda, máis 28 das tres modificadoras, máis 32, 127. Por iso `code` é un `u8`, e `32 + code` nunca desborda.
- `Some(character) if value >= 0x80` é un brazo de `match` cunha garda: só os valores desde 128 van en UTF-8, e o resto, tal cal (`value as u8`, que xa cabe). Ningún valor chega ás metades de UTF-16 (`0xD800`–`0xDFFF`), que non son caracteres: `char::from_u32` sempre dá `Some`.
- Os números de SGR escríbeos [`push_decimal`](#os-axudantes-das-secuencias), de `input.rs`, sen reservar memoria. Quen chama reutiliza `out`, e envía cada informe nunha escritura, porque o ConPTY le cada escritura enteira ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)).

### A cela debaixo do punteiro: `cell_at`

```rust
pub fn cell_at(at: Point, cell: Size, grid: GridSize) -> (GridPoint, Side) {
    let width = i64::from(cell.width.max(1));
    let height = i64::from(cell.height.max(1));
    let (x, y) = (i64::from(at.x), i64::from(at.y));
    let column = (x.max(0) / width).min(i64::from(grid.columns.max(1)) - 1);
    let row = (y.max(0) / height).min(i64::from(grid.rows.max(1)) - 1);
    let side = if x >= width * i64::from(grid.columns) || x.max(0) % width > width / 2 {
        Side::Right
    } else {
        Side::Left
    };
    // Both are below the columns and the rows of the grid.
    (GridPoint::new(column as u16, row as u16), side)
}
```

**`cell_at`** dá a cela dunha grella de `grid` celas de `cell` píxeles que está debaixo do punto `at`, en píxeles desde a esquina superior esquerda da grella, e o lado da cela onde está ([`Side`](#word_delimiters-selectionkind-e-side)).

- **A cela**: a columna é `x` entre o ancho da cela, e a fila, `y` entre o alto. Un punto fóra da grella dá a cela máis próxima (`max(0)` e `min`): unha selección que se arrastra fóra da xanela chega ata o bordo.
- **O lado**: a metade esquerda chega ata a metade da cela, incluída; cunha cela de 10 píxeles, do 0 ao 5. Á dereita da grella, o lado dereito da última columna: unha selección arrastrada alén do final da fila colle a súa última cela.
- As contas fanse en `i64`, onde caben unha coordenada `i32`, un tamaño `u32` e o seu produto. `max(1)` evita dividir entre 0 mentres non se sabe o tamaño da cela, e que unha grella sen columnas ou sen filas dea −1. Despois de `min`, a columna e a fila caben nun `u16`: o `as u16` non corta nada.

### Os clics seguidos: `ClickSettings` e `Clicks`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClickSettings {
    pub time: Duration,
    pub width: u32,
    pub height: u32,
}

impl Default for ClickSettings {
    fn default() -> Self {
        Self {
            time: Duration::from_millis(500),
            width: 4,
            height: 4,
        }
    }
}
```

**`ClickSettings`**: cando toma Windows dous clics por un dobre clic: o tempo do dobre clic (`GetDoubleClickTime`) e o tamaño do rectángulo arredor do primeiro clic onde ten que caer o segundo (`SM_CXDOUBLECLK` e `SM_CYDOUBLECLK`). Os valores por defecto son os de Windows: 500 ms e 4 × 4 píxeles. meiga le os de Windows ao arrancar, e outra vez cada vez que a xanela recupera o foco, porque poden cambiar mentres corre ([Código › meiga-win › `window.rs`](meiga-win.md#cratesmeiga-winsrcwindowrs), [Tecnoloxías › O rato en Windows](../tecnoloxias.md#44-o-rato-en-windows)).

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct Clicks {
    last: Option<LastClick>,
}

#[derive(Clone, Copy, Debug)]
struct LastClick {
    button: Button,
    at: Point,
    when: Instant,
    count: u8,
}

impl Clicks {
    …
    pub fn press(
        &mut self,
        button: Button,
        at: Point,
        now: Instant,
        settings: &ClickSettings,
    ) -> u8 {
        let count = match self.last {
            Some(last)
                if last.button == button
                    && now.saturating_duration_since(last.when) <= settings.time
                    && at.x.abs_diff(last.at.x) <= settings.width / 2
                    && at.y.abs_diff(last.at.y) <= settings.height / 2 =>
            {
                last.count % 3 + 1
            }
            _ => 1,
        };
        self.last = Some(LastClick {
            button,
            at,
            when: now,
            count,
        });
        count
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}
```

**`Clicks`** conta os clics dun botón: o segundo seguido é un dobre clic, e o terceiro, un triplo clic. Windows só detecta os dobres, e só nas xanelas dunha clase con `CS_DBLCLKS`: meiga cóntaos todos, coas mesmas medidas ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

- **`press`** conta un clic de `button` en `at`, no instante `now`: 1, ou un máis ca o anterior se foi do mesmo botón, dentro do tempo do dobre clic e preto del, como moito a metade do rectángulo cara a cada lado: 2 píxeles, co de Windows. Despois do 3 volta o 1 (`last.count % 3 + 1`): o cuarto clic seguido é outra vez un clic só. O rato da vista selecciona celas cun clic, palabras con dous e liñas con tres.
- Cada clic compárase co anterior: o tempo e a distancia cóntanse desde el. `LastClick`, privado, lémbrao: o botón, onde e cando foi, e a súa conta.
- `now` vén de fóra, coma en [`FramePacer`](#cratesmeiga-coresrcpacingrs): as probas usan instantes inventados. `saturating_duration_since` dá 0 se `now` fose anterior, en vez de fallar, e `abs_diff` dá a distancia entre dous `i32` coma un `u32`, sen desbordar.
- **`reset`** esquece o último clic: o seguinte é un clic só.

### A roda: `WheelTurns`, `WheelRows` e `arrow_keys`

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct WheelTurns {
    vertical: i32,
    horizontal: i32,
}

impl WheelTurns {
    …
    pub fn add(&mut self, delta: i16, horizontal: bool) -> i32 {
        let total = if horizontal {
            &mut self.horizontal
        } else {
            &mut self.vertical
        };
        let delta = i32::from(delta);
        if (*total > 0 && delta < 0) || (*total < 0 && delta > 0) {
            *total = 0;
        }
        *total += delta;
        let notches = *total / WHEEL_DELTA;
        *total -= notches * WHEEL_DELTA;
        notches
    }
    …
}
```

**`WheelTurns`** suma os xiros da roda ata facer partes enteiras: algúns ratos e *touchpads* xiran menos dunha parte cada vez ([Tecnoloxías › O rato en Windows](../tecnoloxias.md#44-o-rato-en-windows)).

- **`add`** suma un xiro de `delta`, en 120 avos dunha parte, positivo adiante ou cara á dereita, e devolve as partes enteiras que completa, con signo. O que sobra queda para o seguinte xiro: tres anacos de 50 dan 0, 0 e 1 parte, e quedan 30.
- **Un xiro no outro sentido empeza de cero**: o que sobraba no sentido anterior esquécese.
- A división de enteiros corta cara a 0, así que o que sobra ten o signo do total e nunca chega a unha parte.
- A roda e a inclinación cara aos lados suman por separado (`vertical` e `horizontal`). `delta` é un `i16`, coma o trae a mensaxe de Windows, e o total, un `i32`. `reset` esquece o que sobraba.
- Por cada parte, o rato da vista envíalle ao programa un informe, se pediu o rato, ou frechas, na pantalla alternativa. Desde a subfase 1.9, con Ctrl, cada parte é un punto de zoom; e, na pantalla principal, se a roda despraza unha pantalla, unha pantalla do historial. Se despraza liñas, as filas cóntaas `WheelRows`.

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct WheelRows {
    total: i64,
}

impl WheelRows {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, delta: i16, rows: u32) -> i32 {
        let delta = i64::from(delta) * i64::from(rows);
        if (self.total > 0 && delta < 0) || (self.total < 0 && delta > 0) {
            self.total = 0;
        }
        self.total += delta;
        let whole = self.total / i64::from(WHEEL_DELTA);
        self.total -= whole * i64::from(WHEEL_DELTA);
        i32::try_from(whole).unwrap_or(if whole < 0 { i32::MIN } else { i32::MAX })
    }
}
```

**`WheelRows`**, desde a subfase 1.9, suma os xiros da roda en filas do historial, a un número de filas por parte: as liñas que din as preferencias de Windows ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)).

- **`add`** suma un xiro de `delta`, en 120 avos dunha parte, positivo adiante, a `rows` filas por parte, e devolve as filas enteiras que completa, con signo: positivas cara arriba, coma en [`Scroll::Rows`](#o-historial-e-a-súa-vista).
- **En 120 avos dunha fila** (`total`): cada xiro suma `delta` por `rows`, así que unha parte enteira son `rows` filas, e os anacos dunha parte moven unha fila en canto chegan a ela: con 3 filas por parte, cada 40. Contando partes enteiras, coma `WheelTurns`, a vista moveríase de `rows` en `rows` filas ao completar cada parte, e cun *touchpad*, que xira menos dunha parte cada vez, iría a saltos.
- **Un xiro no outro sentido empeza de cero**, coma en `WheelTurns`.
- As contas fanse en `i64`, onde caben un `i16` por un `u32` e as súas sumas. As filas, nun `i32`, quedan no máximo ou no mínimo (`unwrap_or`) se non caben, o que só pasaría cun número de filas por parte enorme.
- O rato da vista úsao na pantalla principal, cando a roda despraza liñas, e move a vista as filas que devolve ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)).

```rust
pub const fn arrow_keys(wheel: Wheel) -> [KeyEvent; 2] {
    let (key, scan) = match wheel {
        Wheel::Up => (0x26, 0x48),
        Wheel::Down => (0x28, 0x50),
        Wheel::Left => (0x25, 0x4b),
        Wheel::Right => (0x27, 0x4d),
    };
    let down = KeyEvent {
        key,
        scan,
        character: 0,
        down: true,
        controls: ControlKeys::ENHANCED,
        repeat: 1,
    };
    let mut up = down;
    up.down = false;
    [down, up]
}
```

**`arrow_keys`** dá a frecha de `wheel`, ao baixar e ao subir, para os programas da pantalla alternativa que non pediron o rato, coma `less`: desprázanse coas frechas. É o modo `?1007` (`alternate_scroll` en `Modes`), activo ao principio, coma no xterm e no Windows Terminal ([Tecnoloxías › O rato nos programas](../tecnoloxias.md#45-o-rato-nos-programas-modos-e-formatos)).

- Son eventos de tecla coma os do teclado ([`KeyEvent`](#keyevent)): o código de tecla virtual de cada frecha (`VK_UP` é `0x26`…), o seu código de exploración, e `ENHANCED`, porque son as frechas de fóra do teclado numérico ([`ControlKeys`](#controlkeys)). O mesmo [`KeyEncoder`](#keyencoder) das teclas convérteos no que le o programa: co win32-input-mode, cada evento enteiro; en VT, a secuencia da frecha, `ESC [ A`…, ou `ESC O A`… no modo aplicación.
- As frechas de cada parte son as liñas das preferencias de Windows, ou as filas da pantalla se a roda despraza unha pantalla; cara aos lados, os caracteres. Cóntaas o rato da vista ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)).
- `let mut up = down;` copia o evento, que é `Copy`, e despois cambia un campo da copia: nunha `const fn` tamén se pode.

**Probas.** Nove:

| Proba | Que comproba |
|---|---|
| `the_program_gets_only_what_it_asked_for` | Sen modos, nada, e `is_tracked` é falso; con `?1000`, o clic (`ESC [ M` e tres bytes), pero non os movementos; con `?1002`, os movementos cun botón, pero non sen el; e con `?1003`, tamén sen botón. |
| `sgr_reports_say_the_button_and_the_cell_from_1` | En SGR, na columna 120 e na fila 30 (119 e 29 desde 0): premer o botón esquerdo e o do medio, soltar o dereito (co botón e `m`), moverse con e sen botón, a roda adiante e cara á dereita, as tres modificadoras, que suman 28, e Ctrl coa roda cara atrás. E sen límite: a columna 1 000 e a fila 301. |
| `the_first_format_does_not_say_which_button_went_up_and_has_a_limit` | No primeiro formato, soltar o botón dereito é 3, sen o botón; a columna 94 desde 0 é o byte 127; e a 95, de columna ou de fila, non se envía. |
| `utf8_reports_reach_further` | En UTF-8, a columna 94 é o byte 127, coma no primeiro formato; a 95, o carácter 128, en dous bytes; a última que cabe, `MAX_UTF8`, o carácter `0x7FF`; e a seguinte non se envía. |
| `the_cell_under_the_pointer_and_its_side` | Con celas de 10 × 20 píxeles nunha grella de 8 × 4: a metade esquerda chega ata a metade da cela, incluída; e fóra da grella, por riba, á esquerda, á dereita e por baixo, a cela máis próxima, co lado dereito á dereita da grella. |
| `clicks_in_a_row_count_up_to_three` | Tres clics seguidos dan 1, 2 e 3, o segundo 2 píxeles máis aló, e o cuarto, 1; tarde, lonxe ou con outro botón, 1; e despois de `reset`, 1. |
| `the_wheel_adds_up_whole_notches` | Unha parte enteira e tres cara atrás; un *touchpad* que dá unha parte en tres anacos de 50; o que sobra esquécese ao cambiar de sentido; a roda e a inclinación por separado; e `reset`. |
| `the_wheel_moves_the_history_by_rows_also_with_parts_of_a_notch` | Desde a subfase 1.9, con 3 filas por parte: unha parte adiante dá 3 filas, e dúas cara atrás, −6; un *touchpad* que xira de 30 en 30 move unha fila cada 40 (0, 1 e, con 20 máis, outra); e os 30 que sobran esquécense ao cambiar de sentido: −30 e −10 dan unha fila cara abaixo. |
| `the_wheel_on_the_alternate_screen_sends_arrows` | A frecha de arriba baixa e sobe, coma tecla estendida; e `KeyEncoder` converte a de abaixo en `ESC [ B`, e no modo aplicación, en `ESC O B`. |

## `crates/meiga-core/src/paste.rs`

Desde a subfase 1.8: o que le o programa cando se pega texto ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md), [Tecnoloxías › Pegar no terminal](../tecnoloxias.md#47-pegar-no-terminal-o-filtro-e-o-bracketed-paste)). Pegar é teclear: o texto vai á entrada do programa, e un carácter de control sería unha tecla, coma Esc ou Ctrl+C, e cada fin de liña, un Intro. O texto chega do portapapeis de Windows ([Código › meiga-win › `clipboard.rs`](meiga-win.md#cratesmeiga-winsrcclipboardrs)), e o módulo que copia e pega ([Código › meiga › `clipboard.rs`](meiga.md#cratesmeigasrcclipboardrs)) pásao por eiquí, cos modos do terminal, antes de envialo. Coma no Windows Terminal:

- o texto perde os caracteres de control, salvo o tabulador e os fins de liña, para que non poida enviar unha tecla coma Esc nin rematar o *bracketed paste* antes de tempo;
- cada fin de liña pasa a ser un CR, a tecla Intro;
- e, se o programa non pediu o *bracketed paste* (`?2004`), un texto dunha soa liña perde os brancos do final, para que un comando copiado co seu fin de liña non se execute só.

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paste {
    pub text: String,
    pub bracketed: bool,
}

impl Paste {
    pub fn new(clipboard: &str, modes: &Modes) -> Self {
        let bracketed = modes.bracketed_paste;
        let clipboard = if bracketed {
            clipboard
        } else {
            trim(clipboard)
        };
        let mut text = String::with_capacity(clipboard.len());
        let mut after_cr = false;
        for character in clipboard.chars() {
            match character {
                // CR and LF, CRLF and LF CR... each line ends with one CR.
                '\n' if after_cr => {}
                '\r' | '\n' => text.push('\r'),
                '\t' => text.push('\t'),
                // C0, DEL and C1.
                '\0'..='\x1f' | '\x7f'..='\u{9f}' => {}
                _ => text.push(character),
            }
            after_cr = character == '\r';
        }
        Self { text, bracketed }
    }
    …
}
```

**`Paste`** é o texto listo para pegar (`text`), e se vai marcado coma texto pegado (`bracketed`): se o programa pediu o *bracketed paste*. **`new`** faino desde o texto do portapapeis (`clipboard`), co filtro do Windows Terminal (`FilterStringForPaste`):

| Carácter | Que pasa |
|---|---|
| CR, LF e CR LF | Un CR, o que envía a tecla Intro: un LF só chegaría ao PSReadLine coma Ctrl+Intro. `after_cr` lembra se o anterior foi un CR, para que o LF de CR LF non engada outro. LF e CR, nesa orde, son dous fins de liña. |
| O tabulador | Queda. |
| Os outros de control: C0 (`0x00`–`0x1F`), DEL (`0x7F`) e C1 (`U+0080`–`U+009F`) | Desaparecen. Un `ESC [ 201 ~` dentro do texto perde o ESC, e o resto é texto: non remata o pegado antes de tempo. E algúns programas len os C1 coma secuencias: `U+009B` é CSI, coma `ESC [`. |
| O resto | Queda: letras, acentos, emoji… |

- Sen *bracketed paste*, o texto pasa antes por `trim`, que quita os brancos do final dunha soa liña (máis abaixo). Co *bracketed paste*, queda enteiro: decide o programa.
- `'\0'..='\x1f' | '\x7f'..='\u{9f}'` son dous rangos de caracteres nun patrón. Os brazos compróbanse en orde: o CR, o LF e o tabulador, que tamén están no primeiro rango, xa se trataron antes.
- `String::with_capacity(clipboard.len())`: o texto nunca medra, porque cada carácter queda, desaparece ou, un LF, pasa a ser un CR do mesmo tamaño. Unha reserva chega.

```rust
pub fn is_multiline(&self) -> bool {
    self.text.contains('\r')
}

pub fn is_empty(&self) -> bool {
    self.text.is_empty() && !self.bracketed
}

pub fn write_to(&self, out: &mut Vec<u8>) {
    if self.bracketed {
        out.extend_from_slice(b"\x1b[200~");
    }
    out.extend_from_slice(self.text.as_bytes());
    if self.bracketed {
        out.extend_from_slice(b"\x1b[201~");
    }
}
```

- **`is_multiline`**: se ten máis dunha liña, é dicir, algún CR. Sen *bracketed paste*, unha shell executa cada liña ao lela: entón meiga pregunta antes de pegar, nunha caixa de mensaxe coas primeiras liñas (`!paste.bracketed && paste.is_multiline()`, en [Código › meiga › `clipboard.rs`](meiga.md#cratesmeigasrcclipboardrs)). Unha soa liña co seu fin de liña non pregunta: sen *bracketed paste*, `trim` xa lle quitou o fin de liña.
- **`is_empty`**: se non hai nada que enviar. Co *bracketed paste*, un texto baleiro vai igual, só coas marcas: algúns programas, ao recibilo, len unha imaxe do portapapeis. Sen el, un texto baleiro, ou só con brancos, non envía nada.
- **`write_to`** engade a `out` o que le o programa: o texto, entre `ESC [ 200 ~` e `ESC [ 201 ~` co *bracketed paste*. O ConPTY pásalles as marcas tal cal aos programas que len a entrada coma un terminal, coma bash en WSL, e quítallas aos da consola de Windows, coma o PSReadLine, que reciben o texto coma teclas ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

```rust
fn trim(text: &str) -> &str {
    let trimmed = text.trim_end_matches(['\t', '\n', '\x0b', '\x0c', '\r', ' ']);
    if trimmed.contains(['\n', '\r']) {
        text
    } else {
        trimmed
    }
}
```

- **`trim`** quita os brancos do final (o tabulador, LF, VT, FF, CR e o espazo) só se o texto ten unha liña: se despois de quitalos queda algún fin de liña, ten varias, e devólveo como estaba. Os brancos do principio quedan.
- Devolve un anaco do mesmo texto (`&str`), sen copialo. `trim_end_matches` e `contains` aceptan coma patrón un *array* de caracteres, coma en [`shell.rs`](#cratesmeiga-coresrcshellrs).

**Probas.** Catro:

| Proba | Que comproba |
|---|---|
| `each_end_of_line_becomes_a_cr` | CR LF, LF e CR dan un CR cada un, e LF CR, dous; un texto con CR ten varias liñas, e un sen el, unha. |
| `control_characters_go_away_but_tabs_and_text_stay` | Co *bracketed paste*, un `ESC [ 201 ~` dentro do texto perde o ESC e non remata o pegado; BEL, DEL e os C1 `U+0085` e `U+009B` desaparecen; o tabulador, ñ, € e 😀 quedan. |
| `without_bracketed_paste_a_single_line_loses_the_blanks_at_its_end` | Sen *bracketed paste*, `  ls -l \t\r\n` queda en `  ls -l`, cos espazos do principio; unha liña só de brancos non envía nada; varias liñas quedan coma están, cun CR ao final de cada unha; e co *bracketed paste*, unha liña conserva os seus brancos e o seu fin de liña. |
| `an_empty_clipboard_only_goes_with_bracketed_paste` | Un portapapeis baleiro non envía nada sen *bracketed paste*, e só as marcas con el. |

## `crates/meiga-core/src/model/mod.rs`

Desde a subfase 2.1, o modelo das xanelas e as súas tabs: que tabs ten cada xanela, en que orde, cal está activa, e que pasa ao abrir, pechar, mover ou activar unha, sen ningunha interface ([ADR 0018](../adr/0018-o-modelo-das-tabs.md)). Na 2.1 só o usan as súas probas e o exemplo `tabs`; desde a 2.2, o programa terá unha sesión e unha vista en cada tab.

```rust
mod window;

#[cfg(test)]
mod tests;

pub use window::{Close, Switch, Tab, Window};
```

- O módulo ten dous arquivos máis: `window.rs`, coa xanela e as súas tabs, e `tests.rs`, coas probas, que só se compilan con `cargo test` (`#[cfg(test)]`).
- `pub use` publica os tipos de `window.rs` coma se estivesen eiquí: fóra do crate úsanse coma `meiga_core::model::Window`, sen saber en que arquivo están.
- O comentario do módulo (`//!`) explica as tres preguntas que o modelo responde por separado ([ADR 0018](../adr/0018-o-modelo-das-tabs.md), decisión 9): a tab activa, a que marca a barra (`Window::active`); a enfocada, a que recibe o teclado (`Window::focused`); e as visíbeis, as que se debuxan (`Window::is_visible`).

### `TabId` e `WindowId`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(take(&NEXT))
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn from_raw(number: u64) -> Self {
        Self(number)
    }
}

fn take(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, Ordering::Relaxed)
}
```

- **`TabId`** identifica unha tab mentres meiga corre. É un `u64` dentro dun tipo propio (un *newtype*): o compilador non deixa confundilo cun número calquera nin cun `WindowId`. O campo é privado: fóra do módulo, un identificador só se fai co número doutro, con `from_raw`.
- **`next`** dá o seguinte número dun contador `static`, que existe unha soa vez en todo o programa e empeza en 1. É privada: só o modelo dá números novos, cando crea unha tab.
- **`take`** súmalle 1 ao contador e devolve o valor de antes nunha soa operación atómica: dous fíos que abran tabs á vez nunca reciben o mesmo número, e unha tab pechada non devolve o seu. Chega a orde de memoria `Relaxed`, porque só importa que os números non se repitan ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)).
- **`get`** dá o número, por exemplo para envialo nunha mensaxe a outro fío (2.2), e **`from_raw`**, desde a revisión da 2.1, volta facer o identificador co número que chega: as mensaxes de Windows levan números, e non tipos de Rust. Un número inventado non fai dano: o modelo dálle a cada tab nova un número do seu contador e só el pon tabs na xanela, así que `from_raw` non pode facer que dúas tabs compartan un, e un número que non é dunha tab da xanela non atopa nada (`Error::NoSuchTab`).
- Os `derive`: `Clone` e `Copy` copian o identificador coma un número; `Debug` móstrao nas mensaxes das probas; `PartialEq` e `Eq` compáranos; e `Hash` deixa usalos de chave nun mapa, coma fan as probas.
- **`WindowId`** é igual para as xanelas, co seu propio contador. Polo de agora só o garda cada `Window` (`Window::id`), porque meiga ten unha xanela: prepara as varias xanelas (F6).

### `Error`

```rust
pub enum Error {
    NoSuchTab(TabId),
    OutOfRange { index: usize, count: usize },
}
```

Por que unha xanela rexeitou un cambio. A xanela queda como estaba.

- **`NoSuchTab`:** a xanela non ten esa tab, porque xa se pechou ou porque é doutra xanela.
- **`OutOfRange`:** a posición, contada desde 0, está despois da última tab. `index` é a posición pedida, e `count`, cantas tabs hai.

`Display` escribe o erro en inglés, coma as outras mensaxes de meiga: `the window has no tab 7`, `position 3 is out of range for 3 tabs` ou, cunha tab, `for 1 tab`. E implementa `std::error::Error`, o *trait* dos erros de Rust, así que pode ir xunto con outros erros, por exemplo nun `Box<dyn Error>`.

## `crates/meiga-core/src/model/window.rs`

A xanela e as súas tabs.

### `Tab`, `Switch` e `Close`

```rust
pub struct Tab<T> {
    id: TabId,
    content: T,
}

pub struct Switch {
    pub from: TabId,
    pub to: TabId,
}

#[must_use = "the program has to close the content of the tab and show the tab that is active now"]
pub enum Close<T> {
    Removed {
        content: T,
        activated: Option<TabId>,
    },
    LastTab,
}
```

- **`Tab<T>`** é unha tab: o seu identificador e o seu contido, do tipo que escolla o programa ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)). Os campos son privados: fóra do módulo só se len, con `id()` e `content()`. Só o modelo crea tabs, ao crear a xanela e ao abrir unha ([Abrir e pechar](#abrir-e-pechar)).
- **`Switch`:** a tab activa cambiou, de `from` a `to`. Desde a 2.2, o programa redebuxará todo e cambiará a prioridade dos dous fíos lectores.
- **`Close<T>`,** o que fixo pechar unha tab: `Removed`, a tab saíu da xanela, co seu contido e, se era a activa, a tab que ocupa o seu lugar (`activated`); ou `LastTab`, era a única, segue na xanela, e a xanela ten que pecharse con ela.
- **`#[must_use]`** en `Close`, desde a revisión da 2.1: o compilador avisa se quen pecha unha tab ignora a resposta, que pode traer a sesión da tab e a tab que pasa a ser a activa, ou pedir que se peche a xanela. Por ir no tipo, avisa tamén cando o erro sae cun `?`: en `window.close(id)?;`, o `?` usa o `Result`, pero o `Close` que queda dentro ignórase, e iso si o ve o compilador.
- `Close` deriva `PartialEq` e `Eq`, se o contido os ten: as probas comparan a resposta enteira cun `assert_eq!`.

### `Window`: os datos e as consultas

```rust
pub struct Window<T> {
    id: WindowId,
    tabs: Vec<Tab<T>>,
    active: usize,
}
```

- **`tabs`:** as tabs, na orde da barra, nun vector, contiguas en memoria. Nunca está baleiro.
- **`active`:** a posición da tab activa en `tabs`. Gárdase a posición, e non o identificador, porque o programa consultará a activa en cada tecla e en cada fotograma: así é un acceso directo ([ADR 0018](../adr/0018-o-modelo-das-tabs.md), decisión 11).
- Os campos son privados, e ningún método deixa a xanela sen tabs nin a activa fóra delas. Son as súas **invariantes**, as regras que sempre se cumpren, e as probas compróbanas despois de cada cambio.

| Método | Que fai |
|---|---|
| `new(content)` | Crea unha xanela cunha tab, a activa, con ese contido. |
| `try_new_with(content)` | Igual, pero o contido faino unha función que recibe o identificador da tab; se falla, devolve o seu erro, e non hai xanela ([Abrir e pechar](#abrir-e-pechar)). |
| `id()` | O identificador da xanela. |
| `tabs()` | As tabs, na orde da barra, nunha *slice* para lelas. |
| `tab_count()` | Cantas tabs ten: polo menos unha. Chámase así, e non `len`, porque Clippy pide un `is_empty` xunto a cada `len`, e unha xanela nunca está baleira. |
| `index_of(id)` | A posición dunha tab, desde 0, ou `None` se non está. |
| `get(id)` e `get_mut(id)` | O contido dunha tab, para lelo ou para cambialo. |
| `active()` e `active_index()` | A tab activa, e a súa posición. |
| `focused()` e `focused_mut()` | A tab enfocada, a que recibe o teclado, e o seu contido, para cambialo. Hoxe, a activa. |
| `is_visible(id)` | Se unha tab se debuxa. Hoxe, só a activa. |

- Buscar polo identificador (`index_of`, `get` e `get_mut`) percorre o vector ata atopala, con `position` e `find`: con decenas de tabs, uns nanosegundos.
- `active()` le o vector na posición gardada (`&self.tabs[self.active]`), que as invariantes manteñen dentro del.
- `focused` e `is_visible` respóndense coa activa: o modelo non garda nada máis ata a vista dividida (decisión 9). Desde a 2.2, o programa usará `focused` para o teclado e `is_visible` para saber que debuxar, aínda que hoxe dean o mesmo ca `active`.
- Non hai un `tabs_mut`: cunha *slice* mutábel, o programa podería cambiar a orde das tabs sen que a xanela o soubese, e a posición da activa apuntaría a outra tab. O contido cámbiase con `get_mut` e `focused_mut`, que só dan o contido.

### Abrir e pechar

```rust
pub fn open(&mut self, content: T) -> TabId {
    self.push(Tab::new(content))
}

pub fn try_open_with<E>(
    &mut self,
    content: impl FnOnce(TabId) -> Result<T, E>,
) -> Result<TabId, E> {
    Ok(self.push(Tab::try_with(content)?))
}

fn push(&mut self, tab: Tab<T>) -> TabId {
    let id = tab.id;
    self.tabs.push(tab);
    id
}
```

**Abrir** pon a tab nova ao final da barra (decisión 6, decidido contigo) e devolve o seu identificador. Non a activa: o botón + activaraa despois con `activate`, e a restauración da sesión (7.2) abrirá as tabs sen activalas.

```rust
fn try_with<E>(content: impl FnOnce(TabId) -> Result<T, E>) -> Result<Self, E> {
    let id = TabId::next();
    Ok(Self {
        id,
        content: content(id)?,
    })
}
```

**Abrir cun contido que sabe o seu identificador,** desde a revisión da 2.1: `try_open_with` e `try_new_with`, a da xanela nova, reciben unha función que fai o contido, e `Tab::try_with` dálle o identificador da tab antes de que a tab exista. Desde a 2.2, esa función creará a sesión, cuxo fío lector enviará o identificador cos seus avisos, así que o necesita ao nacer.

- `impl FnOnce(TabId) -> Result<T, E>` acepta calquera función ou *closure* que reciba un `TabId` e devolva o contido ou un erro ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)). `FnOnce` chámase unha vez, así que pode levar dentro o que move ao contido. O tipo do erro, `E`, escólleo quen chama: o modelo non sabe que é unha sesión nin como falla.
- Se a función falla, o `?` devolve o erro antes de `push`, e a xanela queda como estaba; a xanela nova non chega a existir. O identificador pérdese: o contador non o devolve, coma cando se pecha unha tab, e con 64 bits non se esgota.
- `open` e `new` usan `Tab::new`, co contido xa feito, e as catro maneiras de crear unha tab rematan igual: só `push` e `with_tab`, a da xanela nova, poñen tabs na xanela, sempre recén creadas e co seu identificador novo.

```rust
pub fn close(&mut self, id: TabId) -> Result<Close<T>, Error> {
    let index = self.find(id)?;
    if self.tabs.len() == 1 {
        return Ok(Close::LastTab);
    }
    let tab = self.tabs.remove(index);
    let activated = match index.cmp(&self.active) {
        Ordering::Less => {
            self.active -= 1;
            None
        }
        Ordering::Greater => None,
        Ordering::Equal => {
            self.active = index.min(self.tabs.len() - 1);
            Some(self.tabs[self.active].id)
        }
    };
    Ok(Close::Removed {
        content: tab.content,
        activated,
    })
}
```

**Pechar:**

1. `find` busca a posición da tab ou, se non está, devolve `Error::NoSuchTab`; o `?` devolve o erro no momento, sen cambiar nada.
2. Se é a única tab, non se quita: `Close::LastTab`, e a xanela pecharase con ela (decisión 4).
3. Se non, `remove` sácaa do vector, e as que estaban á súa dereita corren un posto cara á esquerda. A posición da activa axústase comparando a da tab pechada coa da activa (`cmp`):
   - **á esquerda da activa** (`Less`): a activa correu un posto, e a súa posición baixa un;
   - **á dereita** (`Greater`): nada cambia;
   - **era a activa** (`Equal`): a tab da súa dereita ocupou a súa posición e pasa a ser a activa, coma en Chrome (decisión 5, decidido contigo). Se era a última, a posición quedou despois da nova última, e `min` colle esta, a que estaba á súa esquerda.
4. A resposta leva o contido da tab, para que o programa peche a súa sesión, e a tab activada, se cambiou.

### Mover

```rust
pub fn move_tab(&mut self, id: TabId, to: usize) -> Result<(), Error> {
    let from = self.find(id)?;
    let count = self.tabs.len();
    if to >= count {
        return Err(Error::OutOfRange { index: to, count });
    }
    if from < to {
        self.tabs[from..=to].rotate_left(1);
    } else {
        self.tabs[to..=from].rotate_right(1);
    }
    self.active = moved(self.active, from, to);
    Ok(())
}
```

- `move` é unha palabra reservada de Rust, así que o método chámase `move_tab`.
- Unha posición despois da última devolve `Error::OutOfRange` antes de cambiar nada.
- **Rotar** unha parte do vector move a tab sen sacala nin metela, e sen reservar memoria: cara á dereita, `rotate_left(1)` sobre as posicións de `from` a `to` leva a primeira, a tab que se move, ao final, e corre as demais un posto cara á esquerda; cara á esquerda, `rotate_right(1)` sobre as de `to` a `from` fai o contrario. Para mover unha tab á súa propia posición, a parte ten unha tab, e nada cambia.

```rust
const fn moved(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}
```

**`moved`** di onde acaba a tab que estaba en `index`: a que se move, en `to`; as que estaban entre as dúas posicións, un posto á esquerda se a tab foi cara á dereita, ou á dereita se foi cara á esquerda; e as demais, onde estaban. A xanela úsaa coa posición da activa, que así segue sendo a mesma tab (decisión 7). É unha `const fn`, porque só fai contas.

### Activar

```rust
pub fn activate(&mut self, id: TabId) -> Result<Option<Switch>, Error> {
    let index = self.find(id)?;
    Ok(self.activate_at(index))
}

#[must_use = "the program has to show the tab that is active now"]
pub fn activate_next(&mut self) -> Option<Switch> {
    self.activate_at((self.active + 1) % self.tabs.len())
}

#[must_use = "the program has to show the tab that is active now"]
pub fn activate_previous(&mut self) -> Option<Switch> {
    let count = self.tabs.len();
    self.activate_at((self.active + count - 1) % count)
}

fn activate_at(&mut self, index: usize) -> Option<Switch> {
    if index == self.active {
        return None;
    }
    let from = self.tabs[self.active].id;
    self.active = index;
    Some(Switch {
        from,
        to: self.tabs[index].id,
    })
}
```

- **`activate`** activa unha tab polo seu identificador; **`activate_next`** e **`activate_previous`**, a seguinte e a anterior. O resto da división (`%`) dá a volta: desde a última, a seguinte é a primeira, e desde a primeira, a anterior é a última. Para a anterior, súmase `count` antes de restar 1, para non baixar de 0. Cunha soa tab, as dúas dan a mesma tab, e nada cambia.
- **`activate_at`** fai o cambio e responde cun `Switch`, de que tab a que tab, ou con `None` se xa era a activa.
- **`#[must_use]`** en `activate_next` e `activate_previous` fai que o compilador avise se quen chama ignora a resposta: desde a 2.2, quen active unha tab terá que mostrala. `activate` devolve un `Result`, que o compilador tampouco deixa ignorar enteiro; pero, se o erro sae cun `?` (`window.activate(id)?;`), o `Option<Switch>` que queda dentro pódese ignorar sen aviso. Un `#[must_use]` en `Switch` non o evitaría: o aviso dun tipo non pasa a través de `Option`. `Close` si o leva, porque se devolve sen `Option` ([`Tab`, `Switch` e `Close`](#tab-switch-e-close)). O exemplo `tabs`, que non debuxa, descarta a resposta a propósito, con `let _ =` e con `map(|_| ())`.

## `crates/meiga-core/src/model/tests.rs`

As probas do modelo: vinte e unha, que levan uns milisegundos.

```rust
fn check<T>(window: &Window<T>) {
    assert!(window.tab_count() >= 1, "a window without tabs");
    assert_eq!(window.tabs().len(), window.tab_count());
    assert!(window.active_index() < window.tab_count());
    let active = window.active().id();
    assert_eq!(window.tabs()[window.active_index()].id(), active);
    assert_eq!(window.focused().id(), active);
    let ids: HashSet<TabId> = window.tabs().iter().map(Tab::id).collect();
    assert_eq!(ids.len(), window.tab_count(), "a repeated identifier");
    for (index, tab) in window.tabs().iter().enumerate() {
        assert_eq!(window.index_of(tab.id()), Some(index));
        assert_eq!(window.is_visible(tab.id()), tab.id() == active);
    }
}
```

- **`check`** comproba as invariantes: polo menos unha tab, a activa entre elas, a enfocada igual ca a activa, ningún identificador repetido, a posición de cada tab igual ca a que dá `index_of`, e só a activa visíbel. Cada proba chámaa despois de cada cambio, tamén dos que a xanela rexeita.
- **`window(&["a", "b", "c"])`** crea unha xanela cunha tab por nome, a primeira activa, e devolve tamén os seus identificadores; tamén comproba as invariantes, ao crear a xanela e despois de abrir cada tab. **`names`** dá os contidos de esquerda a dereita, para comparalos cunha lista. **`removed`** e **`switched`** escriben as respostas de pechar e de activar.

Vinte probas comproban cada regra:

- unha xanela nova ten unha tab, e é a activa; unha tab ábrese ao final, e non se activa;
- un contido pode saber o identificador da súa tab, ao crear a xanela e ao abrir unha, e un contido que non se pode facer non abre nada; un identificador volta do seu número, e o número 0, que o contador nunca dá, non atopa ningunha tab;
- os identificadores nunca se repiten, nin noutra xanela nin despois de pechar unha tab;
- pechar a activa activa a da dereita ou, se era a última, a da esquerda; pechar outra deixa a activa, tamén cando a pechada estaba á súa esquerda e a activa corre un posto; e a única tab queda, e a xanela péchase con ela;
- mover corre as tabs do medio, e a activa segue sendo a mesma tab: cando se move ela, cando outra a cruza desde a esquerda ou desde a dereita, e cando outra se move sen cruzala;
- unha posición despois da última, unha tab xa pechada e unha tab doutra xanela rexéitanse, e a xanela queda igual;
- activar responde o cambio, ou nada se xa era a activa; a seguinte e a anterior dan a volta; e cunha soa tab non cambian nada;
- o teclado e a pantalla seguen a activa (`focused`, `focused_mut` e `is_visible`), e `get_mut` cambia o contido doutra tab;
- e as mensaxes dos erros.

**A proba ao chou** (`random_changes_keep_the_rules_and_match_a_plain_list`) fai 500 cambios en cada unha de 20 xanelas, 10 000 en total: abrir, con `open` ou con `try_open_with`, e ás veces despois dun intento cun contido que non se pode facer; pechar; mover (ás veces a unha posición despois da última); activar unha tab, a seguinte ou a anterior; e pechar ou activar unha tab xa pechada. Os cambios saen dunha secuencia *xorshift* fixa, coma nas probas de `damage.rs` e do escáner: se falla, falla sempre igual. Fai os mesmos cambios nunha lista simple, cos contidos das tabs e o contido da activa, coas regras escritas da forma máis directa. Despois de cada cambio comproba as invariantes, que a xanela e a lista coinciden, e que as respostas son as que tocan. Ao final comproba que a barra pasou por unha tab, por dúas e por dez ou máis, para que os extremos se proben tamén.

Comprobouse tamén ao revés, con catro copias do modelo estragadas a propósito: unha activa a tab da esquerda ao pechar a activa, outra non corre a activa ao pechar unha tab á súa esquerda, outra axusta mal a activa ao mover, e a última non dá a volta coa seguinte. Despois da revisión, outras dúas: unha dálle ao contido outro identificador ca o da súa tab, e outra fai mal o identificador co seu número. As seis fallan as probas.

## `crates/meiga-core/src/term/mod.rs`

O terminal: a fachada sobre `alacritty_terminal` ([ADR 0003](../adr/0003-nucleo-vt-alacritty-terminal.md) e [ADR 0012](../adr/0012-fachada-do-terminal.md)). meiga dálle o que escribe o programa e le del o que hai que mostrar e o que hai que responder, en tipos propios. Desde a subfase 1.6 ten tamén a paleta coa que se debuxan as celas, e responde ás preguntas polas cores e polo tamaño en píxeles ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Desde a 1.8 ten a selección, que vive na grella de `alacritty_terminal` e que a copia do que cambia marca nas celas ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). Desde a 1.9, cando cambia o tamaño, coloca a pantalla onde a pon o ConPTY e garda á parte a pantalla principal mentres se ve a alternativa, e move a vista do historial ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). Só este módulo usa os tipos de `alacritty_terminal`: cambialo por outro núcleo, ou actualizalo a unha versión cunha API distinta, só cambia este módulo. Como encaixa co resto, en [Arquitectura › O terminal](../arquitectura.md#o-terminal-da-saída-ao-que-se-mostra), [› Da saída á pantalla](../arquitectura.md#da-saída-á-pantalla-o-aviso-a-copia-e-o-fotograma), [› O rato, a selección e o portapapeis](../arquitectura.md#o-rato-a-selección-e-o-portapapeis) e [› O historial, o tamaño e o zoom](../arquitectura.md#o-historial-o-tamaño-e-o-zoom).

```rust
mod handler;
pub mod recording;
mod resize;
mod scanner;
mod screen;
mod selection;

pub use screen::{Cell, CellFlags, Color, Columns, Cursor, CursorShape, Screen};
pub use selection::{SelectionKind, Side};
```

- Os outros arquivos do cartafol `src/term/` son os submódulos de `term`.
- `recording` é público: o exemplo `replay` e as probas usan as gravacións como `meiga_core::term::recording`. `scanner`, `screen` e, desde a subfase 1.8, `selection` son privados, coma, desde a 1.9, `handler` e `resize`, pero os tipos de `screen` e de `selection` reexpórtanse con `pub use`: fóra úsanse como `meiga_core::term::Screen` ou `meiga_core::term::SelectionKind`, sen saber en que arquivo están.
- Desde a 1.9, `use alacritty_terminal::grid::{self, Dimensions, Grid};` trae tamén o módulo `grid` (`self`): o `Scroll` de `alacritty_terminal` escríbese `grid::Scroll`, porque `Scroll` é o tipo de meiga ([O historial e a súa vista](#o-historial-e-a-súa-vista)). E `use handler::Hooked;` trae o `Handler` que recibe a saída do programa ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)).

### Constantes

```rust
pub const SCROLLBACK_LINES: usize = 10_000;

pub const MIN_SIZE: GridSize =
    GridSize::new(term::MIN_COLUMNS as u16, term::MIN_SCREEN_LINES as u16);

const DEFAULT_CURSOR: CursorStyle = CursorStyle {
    shape: ansi::CursorShape::Beam,
    blinking: true,
};
```

- **`SCROLLBACK_LINES`**: as liñas de historial que se gardan por riba da pantalla, só na pantalla principal: a alternativa non ten. Desde a subfase 1.9, a vista móstraas ([O historial e a súa vista](#o-historial-e-a-súa-vista)); ata a 1.8, gardábanse sen poder velas. `alacritty_terminal` non as reserva ao crear o terminal: a memoria do historial medra a medida que se enche, de 1 000 en 1 000 liñas. Cada cela ocupa 24 bytes, tamén as baleiras: un historial cheo de 100 columnas son 24 MB ([ADR 0016 › Consecuencias](../adr/0016-o-historial-o-tamano-e-o-zoom.md#consecuencias)). Desde a 1.9, úsaa tamén [`new_primary`](#a-pantalla-principal-á-parte), para a pantalla principal nova despois dun reinicio na alternativa.
- **`MIN_SIZE`**: a grella máis pequena que admite `alacritty_terminal`, as súas constantes `MIN_COLUMNS` e `MIN_SCREEN_LINES`: 2 columnas e 1 fila. Un carácter ancho necesita dúas columnas: nunha grella dunha soa columna, `alacritty_terminal` falla (*panic*) ao escribilo. `as u16` converte os dous valores, que son `usize`, porque nunha constante non se pode usar `try_from`; son 2 e 1, así que caben.
- **`DEFAULT_CURSOR`**, desde a subfase 1.6: o cursor cando o programa non escolle ningún, unha barra que pestanexa, coma no Windows Terminal ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Vai na configuración de `alacritty_terminal` (`default_cursor_style`), que o usa mentres o programa non pide outro, e outra vez despois de `ESC [ 0 SP q` ou dun reinicio (RIS). Se o programa só pide un cursor fixo ou que pestanexe (`?12l` e `?12h`), a barra segue, e só cambia o pestanexo. É privada porque `CursorStyle` é un tipo de `alacritty_terminal`: fóra da fachada, o cursor é un [`Cursor`](#cratesmeiga-coresrctermscreenrs). Que pestanexe de verdade decídeo a vista do terminal, segundo Windows ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)).

### `Changes` e `Modes`

```rust
pub struct Changes {
    pub responses: bool,
    pub title: bool,
    pub bell: bool,
    pub clipboard: bool,
}
```

- **`Changes`**: o que pasou ao ler a saída, ademais dos cambios na pantalla: hai respostas para o programa (`take_responses`), cambiou o título (`title`), soou a campá ou hai texto para o portapapeis (`take_clipboard`). Devólveno `feed` e `end_sync`, e quen chama só ten que facer algo máis cando un campo é `true`.
- **`Modes`**: os modos que activou o programa e que cambian o que meiga lle envía: como se codifican as teclas (win32-input-mode, as frechas e o teclado numérico en modo aplicación), os eventos de foco, o pegado entre marcas (*bracketed paste*), que eventos do rato quere e en que formato, se a roda envía frechas, se está na pantalla alternativa, se se ve o cursor, e se o salto de liña volta tamén á primeira columna. Os números de cada modo están en [Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control). `win32_input` dío o escáner, porque `alacritty_terminal` non coñece ese modo; o resto, `alacritty_terminal`. Desde a subfase 1.8, os modos do rato deciden se os clics seleccionan texto ou van ao programa, e en que formato ([`mouse.rs`](#cratesmeiga-coresrcmousers)), e o *bracketed paste*, como se pega o texto ([`paste.rs`](#cratesmeiga-coresrcpasters)). Desde a 1.9, na pantalla alternativa (`alt_screen`), que non ten historial, as teclas do historial van ao programa ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

### Os eventos: `Pending` e `Listener`

```rust
#[derive(Default)]
struct Pending {
    changes: Changes,
    responses: Vec<u8>,
    questions: Vec<(usize, Question)>,
    title: Option<String>,
    clipboard: Option<String>,
}

enum Question {
    Color(usize, Arc<dyn Fn(ansi::Rgb) -> String + Send + Sync>),
    TextAreaSize,
}

struct Listener(Arc<Mutex<Pending>>);
```

Mentres le a saída, `alacritty_terminal` avisa do que pasa chamando a `send_event` do seu `EventListener`, un trait que implementa `Listener`. `send_event` recibe `&self`, unha referencia que non deixa cambiar nada, así que os eventos van a unha cola, `Pending`, que o `Listener` comparte co `Terminal` nun `Arc<Mutex<…>>`:

- **`Arc`** é un punteiro con contador de referencias que se pode compartir entre fíos: o `Terminal` e o seu `Listener` apuntan á mesma cola, que se destrúe cando se destrúen os dous.
- **`Mutex`** permite cambiar o que garda a través dunha referencia compartida, bloqueándoo mentres tanto. Ninguén máis o bloquea, así que nunca hai que esperar. Fai falta porque o terminal ten que poder pasar dun fío a outro (`Send`): `RefCell`, a alternativa sen bloqueo, non o permite dentro dun `Arc`.

| Evento | Que fai meiga |
|---|---|
| `PtyWrite` | Unha resposta para o programa: os atributos do dispositivo (DA1 e DA2), o estado e a posición do cursor (DSR), o tamaño en celas… Engádese a `responses`. |
| `ColorRequest(index, answer)` | Desde a subfase 1.6: unha pregunta por unha cor (OSC 4, 10, 11 ou 12 con `?`). Anótase en `questions` (`Question::Color`), coa posición na que irá a súa resposta: o final de `responses` nese momento. |
| `TextAreaSizeRequest` | Desde a 1.6: unha pregunta polo tamaño da grella en píxeles (`CSI 14 t`). Anótase igual (`Question::TextAreaSize`). |
| `Title` | O título que lle dá o programa á xanela (OSC 0 e 2). Un título baleiro é como non ter ningún: a tab mostrará o seu. |
| `ResetTitle` | O programa recuperou un título gardado (`ESC [ 23 t`) que era ningún. |
| `Bell` | A campá (BEL). |
| `ClipboardStore(Clipboard, …)` | Texto para o portapapeis (OSC 52). Windows non ten selección primaria: só se garda o do portapapeis. |
| Os demais | Ignóranse. Ler o portapapeis nunca se permite (`Osc52::OnlyCopy`), así que nunca se pide. `CursorBlinkingChange` tampouco fai falta: cada copia compara o cursor, tamén o seu pestanexo. O resto son para o bucle de eventos do Alacritty, que meiga non usa. |

**As preguntas que `alacritty_terminal` non sabe responder.** As cores dependen da paleta de meiga, e o tamaño en píxeles, do tamaño dunha cela, que coñece quen debuxa: `alacritty_terminal` envíaas como eventos, en vez de respondelas. O `Listener` só ten a cola, así que as anota en `questions`, e o terminal respóndeas cando remata de ler ([As preguntas](#as-preguntas-polas-cores-e-polo-tamaño)). Cada `Question` garda o que fai falta para respondela:

- **`Color(index, answer)`**: o número da cor nas cores de `alacritty_terminal` (de 0 a 255, as da paleta; 256, 257 e 258, o texto, o fondo e o cursor) e a función que escribe a resposta, que trae o evento. É un `Arc<dyn Fn…>` porque así a dá `alacritty_terminal`; e `Send` e `Sync` deixan que a cola, e o terminal con ela, siga podendo pasar dun fío a outro.
- **`TextAreaSize`** non garda nada: a resposta sae do tamaño da grella e do dunha cela. O evento trae tamén unha función que a escribiría, pero meiga non a usa (ver máis abaixo).

```rust
struct TermSize(GridSize);

impl Dimensions for TermSize { … }
```

`alacritty_terminal` pide o tamaño a través do trait `Dimensions`: as filas que se ven (`screen_lines`), as columnas e as filas en total. `TermSize` envolve un `GridSize` para implementalo: implementalo directamente en `GridSize`, que é público, mostraría `alacritty_terminal` fóra da fachada. O historial non conta: `total_lines` é igual a `screen_lines`. Ata a subfase 1.5 chamábase `Size`; na 1.6 cambiou de nome porque o módulo usa agora o `Size` de [`geometry.rs`](#cratesmeiga-coresrcgeometryrs), o tamaño dunha cela en píxeles, e os dous nomes chocarían. Desde a 1.9, úsano tamén os cambios de tamaño de [`term/resize.rs`](#cratesmeiga-coresrctermresizers).

### `Terminal`

```rust
pub struct Terminal {
    term: Term<Listener>,
    primary: Option<Grid<TermCell>>,
    region_below_top: bool,
    parser: Processor,
    scanner: Scanner,
    pending: Arc<Mutex<Pending>>,
    size: GridSize,
    title: Option<String>,
    theme: Palette,
    cell: Size,
    damaged: Vec<LineDamageBounds>,
    shown_selection: Option<SelectionRange>,
}
```

| Campo | Que é |
|---|---|
| `term` | O terminal de `alacritty_terminal`: a grella, o historial, o cursor e os modos. |
| `primary` | Desde a subfase 1.9: a pantalla principal, coa súa grella e o seu historial, mentres está á parte porque o tamaño cambiou cando se vía a alternativa; `None` o resto do tempo. Recibe o tamaño do terminal cando o programa volta a ela ([`term/resize.rs`](#cratesmeiga-coresrctermresizers)). `TermCell` é o `Cell` de `alacritty_terminal`, co nome cambiado, porque `Cell` é o de meiga. |
| `region_below_top` | Desde a revisión do código da subfase 1.9: a rexión de desprazamento do programa empeza máis abaixo da primeira fila da pantalla. `alacritty_terminal` non di onde empeza a súa, e [`Hooked`](#hooked) séguea eiquí, para deixar a vista do historial onde está ([O historial e a súa vista](#o-historial-e-a-súa-vista)). |
| `parser` | O parser de `vte` (`Processor`), que le os bytes e chama aos métodos de `term`. Garda o estado entre lecturas: unha secuencia partida entre dous bloques, ou un carácter UTF-8 a medias. Tamén garda a saída dunha actualización sincronizada mentres non remata. |
| `scanner` | O escáner propio ([`scanner.rs`](#cratesmeiga-coresrctermscannerrs)). |
| `pending` | A cola dos eventos, compartida co `Listener`. |
| `size` | O tamaño da grella. |
| `title` | O último título que deu o programa. |
| `theme` | Desde a subfase 1.6: as cores do tema, `Palette::MEIGA` ata que haxa temas. As que cambia o programa non van eiquí: gárdaas `alacritty_terminal` ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `cell` | Desde a 1.6: o tamaño dunha cela en píxeles, para responder a `CSI 14 t`; 0 × 0 mentres non se sabe. |
| `damaged` | As filas danadas, nun `Vec` que se reutiliza en cada copia para non reservar memoria. |
| `shown_selection` | Desde a subfase 1.8: a selección que mostra a pantalla, como a deixou a última copia, coas súas celas na grella (`SelectionRange`). `alacritty_terminal` non dana as celas da selección cando esta cambia: a copia compáraa coa nova ([`copy_damage`](#copy_damage-copiar-o-que-cambiou)). |

```rust
const _: () = {
    const fn send<T: Send>() {}
    send::<Terminal>();
};
```

**Unha comprobación ao compilar.** Desde a subfase 1.6, o fío lector da sesión alimenta o terminal e o fío da xanela copia del ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)): `Terminal` ten que poder pasar dun fío a outro, ser `Send`. `send::<Terminal>()` só compila se o é, e un `const _` avalíase ao compilar, sen deixar nada no programa. Se un cambio mete no terminal algo que non é `Send`, como un `Rc`, o erro sae eiquí, e non no crate que o usa, lonxe da causa.

```rust
pub fn new(size: GridSize) -> Self {
    let size = at_least_min_size(size);
    let pending = Arc::new(Mutex::new(Pending::default()));
    let config = Config {
        scrolling_history: SCROLLBACK_LINES,
        default_cursor_style: DEFAULT_CURSOR,
        osc52: Osc52::OnlyCopy,
        kitty_keyboard: false,
        semantic_escape_chars: selection::WORD_DELIMITERS.to_owned(),
        ..Config::default()
    };
    Self {
        term: Term::new(config, &TermSize(size), Listener(Arc::clone(&pending))),
        primary: None,
        region_below_top: false,
        parser: Processor::new(),
        …
        theme: Palette::MEIGA,
        cell: Size::new(0, 0),
        damaged: Vec::new(),
        shown_selection: None,
    }
}

pub fn feed(&mut self, bytes: &[u8]) -> Changes {
    self.scanner.scan(bytes);
    let mut hooked = Hooked {
        term: &mut self.term,
        primary: &mut self.primary,
        region_below_top: &mut self.region_below_top,
    };
    self.parser.advance(&mut hooked, bytes);
    self.take_changes()
}
```

- **`new`** crea un terminal de polo menos `MIN_SIZE`, coa configuración de meiga:
  - 10 000 liñas de historial;
  - desde a subfase 1.6, o cursor por defecto, `DEFAULT_CURSOR`;
  - `Osc52::OnlyCopy`: os programas poden escribir no portapapeis, pero non lelo;
  - sen o protocolo de teclado de kitty: meiga usa o win32-input-mode, que é o que pide o ConPTY ([`input.rs`](#cratesmeiga-coresrcinputrs));
  - desde a subfase 1.8, os caracteres que rematan unha palabra nun dobre clic (`semantic_escape_chars`), os de meiga, [`WORD_DELIMITERS`](#word_delimiters-selectionkind-e-side): `alacritty_terminal` úsaos para estender as seleccións de palabras. `to_owned` copia o texto nun `String`, que é o que garda a configuración;
  - o resto, cos valores por defecto (`..Config::default()`).

  Ten as cores de meiga (`Palette::MEIGA`), ningunha selección e, desde a subfase 1.9, ningunha pantalla principal á parte (`primary: None`) e a rexión de desprazamento na pantalla enteira (`region_below_top: false`), e aínda non sabe o tamaño da cela: 0 × 0 ata que llo dean con `set_cell_size`. `Processor::new()` reserva xa o búfer das actualizacións sincronizadas, de 2 MiB, que Windows conta como memoria privada aínda que non se use. Que facer con el decidirase na subfase 2.2, coa memoria por tab ([ADR 0012 › Consecuencias](../adr/0012-fachada-do-terminal.md#consecuencias)).
- **`feed`** le o que escribiu o programa: primeiro o escáner e despois o parser, cos mesmos bytes, sen copialos nin cambialos. Devolve os `Changes`.
  - Desde a subfase 1.9, o parser non lle dá a saída directamente ao `Term`, senón a un [`Hooked`](#hooked): pásalle todas as ordes e, cando o programa deixa a pantalla alternativa ou reinicia o terminal, devolve no momento a pantalla principal que deixou á parte un cambio de tamaño, antes da saída que vén despois. Tamén deixa a vista do historial onde está cando se despraza unha rexión que non empeza arriba ([O historial e a súa vista](#o-historial-e-a-súa-vista)). `Hooked` toma prestados tres campos do terminal, `term`, `primary` e `region_below_top`, mentres le o parser, que é outro: o compilador permite as catro referencias mutábeis porque son campos distintos.

| Método | Que fai |
|---|---|
| `size` | O tamaño da grella. |
| `set_theme(theme)` | Desde a subfase 1.6: cambia as cores do tema. As que cambiou o programa seguen coa súa. Polo de agora só o usan as probas: os temas chegan na subfase 8.3. |
| `set_cell_size(cell)` | Desde a 1.6: o tamaño dunha cela en píxeles, para responder a `CSI 14 t`. Mentres non o ten, non responde. `meiga.exe` dállo ao crear o terminal, co tamaño da cela da fonte. |
| `palette` | Desde a 1.6: as cores coas que se debuxan as celas ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `take_responses(into)` | Move as respostas ao final de `into`, para envialas ao programa. `Vec::append` deixa a cola baleira pero coa súa memoria: a seguinte resposta non reserva nada. |
| `take_clipboard` | O último texto que o programa copiou ao portapapeis; despois, ningún. Se copiou varios antes de recollelos, só queda o último, como no portapapeis. Desde a subfase 1.8, a xanela recólleo e pono no portapapeis de Windows, só se ten o foco e está en primeiro plano ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md), [Código › meiga › `app.rs`](meiga.md#cratesmeigasrcapprs)). |
| `title` | O título que deu o programa, se deu algún. |
| `modes` | Os modos, traducidos de `TermMode` (uns *bitflags*) aos campos de `Modes`. `win32_input` pregúntallo ao escáner. |
| `resize(size)` | Cambia o tamaño da grella, con polo menos `MIN_SIZE`, e todo queda danado. Ata a subfase 1.8, `alacritty_terminal` repartía as liñas longas da pantalla principal (*reflow*) e colocaba a pantalla á súa maneira, e a selección só se ía se cambiaba o número de columnas. Desde a 1.9, a pantalla queda onde a pon o ConPTY, e a selección vaise sempre ([O tamaño](#o-tamaño-resize)). |
| `sync_deadline` | Cando hai que mostrar a actualización sincronizada (`?2026`) que empezou o programa, aínda que non a remate: `vte` gárdaa como moito 150 ms desde o último `?2026h`, ou ata encher o búfer. Mentres tanto, a saída non chega á pantalla. |
| `end_sync` | Mostra a saída da actualización sincronizada que agarda, e devolve os `Changes`. É para cando vence: o terminal non ten temporizador. Desde a subfase 1.6, remátaa a xanela: antes de cada fotograma consulta `sync_deadline`, e o temporizador dos fotogramas vence a esa hora, tamén coa xanela minimizada, que non debuxa ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)). Desde a 1.9, coma `feed`: a saída pasa polo `Hooked`. |

Desde a subfase 1.8, `Terminal` ten tamén os métodos da selección, noutro arquivo do módulo: `select`, `select_to`, `clear_selection`, `has_selection` e `selection_text` ([`term/selection.rs`](#cratesmeiga-coresrctermselectionrs)). Desde a 1.9, ten tamén os do historial: `scroll`, `scroll_to` e `history` ([O historial e a súa vista](#o-historial-e-a-súa-vista)).

### O tamaño: `resize`

Desde a subfase 1.9 ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). A vista do terminal chámao co tamaño da grella que enche a xanela, como moito unha vez por intervalo dos fotogramas, e despois cambia o tamaño do ConPTY co *lock* do terminal aínda collido: ningunha saída do programa chega entre os dous ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

```rust
pub fn resize(&mut self, size: GridSize) {
    let size = at_least_min_size(size);
    if size == self.size {
        return;
    }
    self.size = size;
    self.region_below_top = false;
    if self.term.mode().contains(TermMode::ALT_SCREEN) {
        if self.primary.is_none() {
            self.primary = Some(resize::set_aside_primary(&mut self.term));
        }
        // Neither ConPTY nor alacritty_terminal reflow the alternate
        // screen.
        self.term.resize(TermSize(size));
        self.term.selection = None;
    } else {
        resize::resize(&mut self.term, size);
    }
}
```

- **Co mesmo tamaño, nada**, tamén despois de `at_least_min_size`: nin garda á parte a pantalla principal nin quita a selección.
- **Na pantalla principal**, [`resize::resize`](#resize-a-pantalla-onde-a-pon-o-conpty) reparte o texto e coloca a pantalla onde a pon o ConPTY.
- **Na pantalla alternativa**, a primeira vez, [`set_aside_primary`](#a-pantalla-principal-á-parte) garda á parte a principal, sen repartila. Despois, `term.resize` cambia a alternativa, que nin o ConPTY nin `alacritty_terminal` reparten: córtase ou énchese pola dereita e por abaixo. `alacritty_terminal` reparte tamén a grella que ocupa o sitio da principal, que, baleira e sen historial, non custa nada. Os seguintes cambios xa atopan a principal á parte (`is_none`), e esta recibe o tamaño do terminal unha vez, cando o programa volta a ela, coma no ConPTY ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)).
- **A selección vaise sempre**, coma no Windows Terminal: `alacritty_terminal` quítaa se cambian as columnas, pero se só cambian as filas, móvea co texto. Na pantalla principal, quítaa `resize::resize`.
- **A rexión de desprazamento volta ser a pantalla enteira**: `alacritty_terminal` faino sempre que cambia o tamaño, nas dúas pantallas. Desde a revisión do código, `region_below_top`, que a segue, volta a `false` ([`Hooked`](#hooked)).

### O historial e a súa vista

Desde a subfase 1.9: o historial, as filas que saíron da pantalla principal por arriba, vese coa roda, coas teclas e coa barra de desprazamento ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md), [Tecnoloxías › O historial](../tecnoloxias.md#49-o-historial-de-alacritty_terminal-a-vista-e-o-reflow)). A vista é o que se mostra: a pantalla, ou unhas filas máis arriba. `alacritty_terminal` gárdaa na súa grella, coma as filas por riba da pantalla onde empeza (`display_offset`), e a copia do que cambia copia as filas que se ven ([`copy_row`](#os-axudantes)). Fóra da fachada, móvese e lese con tipos de meiga.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scroll {
    Rows(i32),
    PageUp,
    PageDown,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct History {
    pub rows: usize,
    pub offset: usize,
}
```

- **`Scroll`**: canto mover a vista:

  | Variante | Ata onde |
  |---|---|
  | `Rows(filas)` | Unhas filas cara arriba, cara ás máis vellas, ou cara abaixo, cun número negativo. |
  | `PageUp` e `PageDown` | Unha pantalla cara arriba ou cara abaixo. |
  | `Top` | Á primeira fila do historial. |
  | `Bottom` | Á pantalla, onde escribe o programa. |

- **`History`**: o historial e a parte del que mostra a vista: `rows`, as filas de historial por riba da pantalla, e `offset`, cantas filas por riba da pantalla empeza a vista, 0 cando mostra a pantalla. Na pantalla alternativa, sen historial, os dous son 0. Con el debúxase o polgar da barra de desprazamento ([Código › meiga › `scrollbar.rs`](meiga.md#cratesmeigasrcscrollbarrs), [Tecnoloxías › A barra de desprazamento](../tecnoloxias.md#52-a-barra-de-desprazamento-e-o-zoom)).
- `alacritty_terminal` ten o seu `Scroll`, coas mesmas cinco variantes (`Delta` en vez de `Rows`), pero non sae da fachada ([ADR 0012](../adr/0012-fachada-do-terminal.md)): neste módulo é `grid::Scroll`.

```rust
pub fn scroll(&mut self, scroll: Scroll) -> bool {
    let before = self.term.grid().display_offset();
    self.term.scroll_display(match scroll {
        Scroll::Rows(rows) => grid::Scroll::Delta(rows),
        Scroll::PageUp => grid::Scroll::PageUp,
        Scroll::PageDown => grid::Scroll::PageDown,
        Scroll::Top => grid::Scroll::Top,
        Scroll::Bottom => grid::Scroll::Bottom,
    });
    self.term.grid().display_offset() != before
}

pub fn scroll_to(&mut self, offset: usize) -> bool {
    let before = self.term.grid().display_offset();
    let offset = offset.min(self.term.grid().history_size());
    // Both fit in an i32: the history has at most SCROLLBACK_LINES rows.
    self.scroll(Scroll::Rows(offset as i32 - before as i32))
}

pub fn history(&self) -> History {
    let grid = self.term.grid();
    History {
        rows: grid.history_size(),
        offset: grid.display_offset(),
    }
}
```

| Método | Que fai |
|---|---|
| `scroll(scroll)` | Move a vista, sen pasar da primeira fila do historial nin da pantalla, e di se se moveu: entón hai que debuxar. Úsano as teclas e a roda, a barra cando pasa páxinas e a selección que se despraza soa; e as teclas que van ao programa, e pegar, levan con el a vista á pantalla (`Bottom`). |
| `scroll_to(offset)` | Leva a vista a `offset` filas por riba da pantalla, como moito ao principio do historial, e di se se moveu. Úsao a barra mentres se arrastra o polgar. |
| `history` | O historial e onde está a vista. Con el, cada fotograma pon ao día o polgar da barra. |

- `scroll_display` de `alacritty_terminal` fai o traballo: non pasa dos extremos e, se move a vista, dana a pantalla enteira. A seguinte copia compara entón todas as filas que se ven, e só marca as celas que cambian ([`copy_damage`](#copy_damage-copiar-o-que-cambiou)).
- `scroll_to` move a vista a diferenza entre `offset` e onde está. As dúas contas caben nun `i32`: o historial ten como moito `SCROLLBACK_LINES` filas, e `offset` xa non pasa del (`min`).
- Na pantalla alternativa, que non ten historial, a vista nunca se move.

- **A saída non move a vista** ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)): mentres mostra o historial, as filas que entran nel súbena tamén, e segue mostrando as mesmas filas. Faino `alacritty_terminal` ao desprazar a grella. Compróbao a proba `the_output_does_not_move_the_view_of_the_history`.
- **Tampouco cunha rexión de desprazamento.** Cunha rexión que non empeza na primeira fila (`ESC [ superior ; inferior r`), as filas que saen dela non entran no historial, pero `alacritty_terminal` sube igual a vista: con cada fila que se despraza, a vista mostraría a de enriba, e podería pasar do principio do historial, onde non hai filas que mostrar. Desde a revisión do código, [`Hooked`](#manter-a-vista-view_to_keep-e-keep_view) déixaa onde estaba. Compróbano as probas `the_rows_that_leave_a_region_below_the_top_do_not_move_the_view` e `a_new_size_a_reset_or_deccolm_make_the_region_the_whole_screen_again`.

### A paleta: o tema e os cambios do programa

```rust
pub fn palette(&self) -> Palette {
    let mut palette = self.theme.clone();
    let colors = self.term.colors();
    for (index, color) in palette.colors.iter_mut().enumerate() {
        if let Some(changed) = colors[index] {
            *color = rgb(changed);
        }
    }
    for (name, color) in [
        (NamedColor::Foreground, &mut palette.foreground),
        (NamedColor::Background, &mut palette.background),
        (NamedColor::Cursor, &mut palette.cursor),
    ] {
        if let Some(changed) = colors[name] {
            *color = rgb(changed);
        }
    }
    palette
}
```

Desde a subfase 1.6. Un programa pode cambiar calquera cor da paleta (OSC 4), a do texto (OSC 10), a do fondo (OSC 11) e a do cursor (OSC 12), e voltar á orixinal (OSC 104, 110, 111 e 112) ([Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). `alacritty_terminal` garda eses cambios en `term.colors()`, unha táboa cun `Option` para cada unha das 256 cores e para as que teñen nome (o texto, o fondo, o cursor…): `None` mentres o programa non a cambia, e outra vez `None` cando volta á orixinal.

- **`palette`** dá a paleta coa que se debuxa: a do tema, coas cores que cambiou o programa por riba. A táboa de `alacritty_terminal` admite como índice un número ou un `NamedColor`, e os nomes van despois das 256 cores. Desde a subfase 1.8, a cor da selección é sempre a do tema: non ten nome nesa táboa, e ningunha secuencia a cambia.
- **Por que o tema vai aparte.** Se os cambios do programa se escribisen no tema, voltar á cor orixinal non sabería cal era, e cambiar de tema borraría o que cambiou o programa. Así, `set_theme` só cambia o tema, e as cores do programa seguen por riba.
- `for (name, color) in [(…, &mut palette.foreground), …]` percorre un *array* de pares, cada un cunha referencia mutábel a un campo distinto da mesma paleta: o compilador permíteo porque non se solapan.
- Devolve unha paleta nova en cada chamada, sen reservar memoria: son 780 bytes na pila. `copy_damage` chámaa en cada copia, e `take_changes`, se hai preguntas.
- `rgb` converte cada cor de `alacritty_terminal` nun `Rgb` de meiga ([Os axudantes](#os-axudantes)).

### As preguntas polas cores e polo tamaño

```rust
fn take_changes(&mut self) -> Changes {
    let mut pending = lock(&self.pending);
    if !pending.questions.is_empty() {
        let palette = self.palette();
        let questions = mem::take(&mut pending.questions);
        let responses = mem::take(&mut pending.responses);
        let mut answered = Vec::with_capacity(responses.len() + 32 * questions.len());
        let mut from = 0;
        for (at, question) in &questions {
            let at = (*at).clamp(from, responses.len());
            answered.extend_from_slice(&responses[from..at]);
            from = at;
            if let Some(answer) = self.answer(question, &palette) {
                answered.extend_from_slice(answer.as_bytes());
                pending.changes.responses = true;
            }
        }
        answered.extend_from_slice(&responses[from..]);
        pending.responses = answered;
    }
    …
}
```

Desde a subfase 1.6, o terminal responde tamén ás preguntas que lle deixa `alacritty_terminal` ([Os eventos](#os-eventos-pending-e-listener)): as das cores e a do tamaño en píxeles. `feed` e `end_sync` rematan os dous en `take_changes`, que as responde antes de devolver os `Changes`, coas cores que hai cando remata de ler. As que chegan dentro dunha actualización sincronizada respóndense cando esta remata: no `feed` que trae o seu fin, ou en `end_sync`.

**A orde.** As respostas teñen que ir na mesma orde ca as preguntas. Moitos programas preguntan pola cor do fondo e despois polos atributos do terminal (DA1, `ESC [ c`), á que responden todos: se esta resposta chega primeiro, entenden que o terminal non sabe responder a outra ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control), [ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Por iso cada pregunta garda onde vai a súa resposta: os bytes que había en `responses` cando chegou. `take_changes` constrúe as respostas outra vez, en orde: as de `alacritty_terminal` ata a posición da primeira pregunta, a súa resposta, as seguintes ata a posición da segunda, e así ata o final. Compróbao a proba `questions_about_colours_are_answered_with_the_palette`, cunha cor entre dúas preguntas DA1.

- `clamp(from, responses.len())`: as posicións só medran e nunca pasan do final, porque mentres se le a saída as respostas só se engaden ao final; `clamp` garante que, aínda así, os rangos nunca fallan.
- `pending.changes.responses` só pasa a `true` se houbo algunha resposta: unha pregunta que o terminal non sabe responder non avisa de nada.
- Reserva sitio para 32 bytes máis por pregunta, o bastante para unha resposta habitual. Só pasa cando houbo preguntas, poucas veces, así que non importa reservar un `Vec` novo.

```rust
fn answer(&self, question: &Question, palette: &Palette) -> Option<String> {
    match question {
        Question::Color(index, answer) => {
            let color = match *index {
                index @ 0..256 => palette.colors[index],
                index if index == NamedColor::Foreground as usize => palette.foreground,
                index if index == NamedColor::Background as usize => palette.background,
                index if index == NamedColor::Cursor as usize => palette.cursor,
                _ => return None,
            };
            Some(answer(ansi::Rgb {
                r: color.r,
                g: color.g,
                b: color.b,
            }))
        }
        Question::TextAreaSize => (!self.cell.is_empty()).then(|| {
            let height = u64::from(self.size.rows) * u64::from(self.cell.height);
            let width = u64::from(self.size.columns) * u64::from(self.cell.width);
            format!("\x1b[4;{height};{width}t")
        }),
    }
}
```

- **Unha cor**: a da paleta, se o número vai de 0 a 255, ou a do texto, a do fondo ou a do cursor. Con outro número, que `alacritty_terminal` non envía, non hai resposta. A resposta escríbea a función que veu coa pregunta, coma a pregunta: co mesmo número e co mesmo final (BEL ou `ESC \`), e cada compoñente coas súas dúas cifras hexadecimais repetidas, catro en total: `#16181D` é `rgb:1616/1818/1d1d`.
- **O tamaño en píxeles**, `ESC [ 4 ; alto ; ancho t`: as filas polo alto dunha cela, e as columnas polo seu ancho. É o da grella, sen as marxes que sobran na xanela. `then` dá `Some` co resultado da *closure* só se a condición se cumpre: sen o tamaño da cela, `None`, porque unha resposta de 0 píxeles sería falsa.
- **Por que non a resposta de `alacritty_terminal`.** A súa función calcula o mesmo, pero multiplicando números de 16 bits (os de `WindowSize`), e o produto pode non caber neles: un *panic* nas compilacións de depuración, e unha resposta errada en *release*. meiga multiplica en `u64`, onde un `u16` por un `u32` sempre cabe.

### `copy_damage`: copiar o que cambiou

```rust
pub fn copy_damage(&mut self, screen: &mut Screen) {
    let resized = screen.size() != self.size;
    if resized {
        screen.resize(self.size);
    }
    self.damaged.clear();
    let full = match self.term.damage() {
        TermDamage::Full => true,
        TermDamage::Partial(lines) => {
            self.damaged.extend(lines);
            resized
        }
    };
    let selection = self.selection_range();
    let grid = self.term.grid();
    if full {
        for row in 0..self.size.rows {
            copy_row(grid, screen, row, selection.as_ref());
        }
    } else {
        for damage in &self.damaged {
            match u16::try_from(damage.line) {
                Ok(row) if row < self.size.rows => {
                    copy_row(grid, screen, row, selection.as_ref());
                }
                _ => {}
            }
        }
        if selection != self.shown_selection {
            let display_offset = grid.display_offset();
            for range in [&self.shown_selection, &selection].into_iter().flatten() {
                for row in selection::shown_rows(range, display_offset, self.size.rows) {
                    copy_row(grid, screen, row, selection.as_ref());
                }
            }
        }
    }
    self.shown_selection = selection;
    let cursor = self.cursor();
    if cursor != screen.cursor() {
        screen.mark_cursor_changed();
        screen.set_cursor(cursor);
        screen.mark_cursor_changed();
    }
    screen.set_palette(&self.palette());
    self.term.reset_damage();
}
```

Copia na `Screen` o que cambiou desde a copia anterior ([ADR 0012](../adr/0012-fachada-do-terminal.md)), desde a subfase 1.6 a paleta ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)), e desde a 1.8 a selección ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)):

1. Se a `Screen` ten outro tamaño, cámbiao: todas as súas celas quedan en branco e cambiadas, e cópianse todas as filas.
2. `term.damage()` dá as filas danadas desde a última copia: `Full`, todas, ou `Partial`, a lista das filas danadas. `alacritty_terminal` dá `Full` cando cambia moito: ao borrar a pantalla, ao desprazala (cada salto de liña na última fila), ao cambiar á pantalla alternativa ou de tamaño, ao mover a vista do historial (desde a subfase 1.9), nun reinicio…
3. Cada fila danada cópiase **enteira** con `copy_row`, que compara cela a cela e só marca as que cambiaron. **Por que enteira:** `alacritty_terminal` dana as celas que escribe, pero non sempre outras da mesma fila que cambian con elas. Ao escribir sobre a metade dereita dun carácter ancho, por exemplo, borra tamén a esquerda, e esa non queda danada. Atopouno a proba aleatoria do terminal. Copiar e comparar unha fila custa pouco, e marcar só as celas distintas evita debuxar o que non cambiou, tamén co dano `Full`: despois de borrar a pantalla, as filas que xa estaban en branco non se marcan.
4. **A selección**, desde a subfase 1.8. `selection_range` dá as súas celas na grella, e `copy_row` marca as que están nela con `CellFlags::SELECTED` ([Os axudantes](#os-axudantes)). Pero `alacritty_terminal` tampouco dana as celas da selección cando esta cambia: se non é a que mostra a pantalla (`shown_selection`), cópianse outra vez as filas das dúas, a vella e a nova ([`shown_rows`](#cratesmeiga-coresrctermselectionrs)), e `copy_row` só marca as celas que entran ou saen dela. Así, arrastrar o rato só debuxa as celas onde cambia a selección. Cando a selección se move co texto, porque a pantalla se desprazou, `alacritty_terminal` xa dá `Full`. Ao final, `shown_selection` garda a nova, tamén despois dunha copia enteira.
5. **O cursor.** Se cambiou (a posición, a forma ou o pestanexo), márcanse as súas celas onde estaba, antes de cambialo, e onde está, despois, con [`mark_cursor_changed`](#cratesmeiga-coresrctermscreenrs): dúas celas, por se está sobre un carácter ancho, e só unha na última columna, e só se se debuxa. A vista do terminal debuxa o cursor por riba delas. Oculto (`Hidden`), non marca nada: un *spinner* que oculta o cursor só redebuxa os seus caracteres, e unha copia sen saída nova non marca nada.
6. **A paleta.** `set_palette` compara a paleta do terminal coa da `Screen`: se cambiou unha das 256 cores, a do texto, a do fondo ou a da selección, cambian todas as celas; se só cambiou a do cursor, só as do cursor. **Por que aparte:** cando o programa cambia unha cor, `alacritty_terminal` dana a pantalla enteira e as filas cópianse, pero as celas gardan o número da súa cor, que é o mesmo: comparadas, ningunha cambiou. E cando só cambia a do cursor, non dana nada. Compróbao a proba `a_change_of_the_palette_changes_every_cell`.
7. `reset_damage` esquece o dano, para a seguinte copia.

- `Ok(row) if row < self.size.rows => …` é un brazo de `match` cunha garda (`if`): só se copian as filas que existen.
- `[&self.shown_selection, &selection].into_iter().flatten()` percorre as dúas seleccións, a vella e a nova, e salta as que son `None`: unha referencia a un `Option` percórrese coma unha lista de cero ou un elementos. Unha fila que está nas dúas cópiase dúas veces, pero a segunda xa non cambia nada.
- `selection.as_ref()` dá un `Option<&SelectionRange>`, o que recibe `copy_row`: unha referencia, sen copiar a selección. Ao final, a selección pasa a ser `shown_selection`. A comparación `!=` mira as celas, non os puntos onde se fixo a selección: arrastrar unha selección de palabras dentro da mesma palabra, por exemplo, non copia nada.

### `cursor` e `take_changes`

- **`cursor`** le o cursor da grella (`grid.cursor.point`) e do estado do terminal. Desde a subfase 1.6 é público: cando o programa remata, a sesión mira se o cursor está ao principio dunha fila, para escribir nunha fila propia a liña co código de saída ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)).
  - a forma é `Hidden` se o programa o ocultou (`?25l`); se non, a que pediu (`cursor_style()`), que tamén di se pestanexa, ou `DEFAULT_CURSOR` se non pediu ningunha;
  - sobre a metade dereita dun carácter ancho, pasa á columna anterior, para debuxalo sobre o carácter enteiro, pero só se esa cela é a súa metade esquerda: coma en `copy_row`, unha metade dereita soa é unha cela en branco, e o cursor debúxase nela;
  - `point_to_viewport` converte a posición na grella, que inclúe o historial, na fila que se ve: se queda fóra, porque desde a subfase 1.9 a vista pode mostrar o historial, o cursor non se debuxa.

  **Por que non `renderable_content()`**, que dá o cursor como o debuxa Alacritty: sobre unha metade dereita, pásao á columna anterior sen comprobar que exista. Borrar caracteres (`ESC [ P`) pode deixar unha metade dereita soa na primeira columna, co cursor enriba, e entón a resta baixa de cero: nas compilacións de depuración, que comproban os desbordamentos tamén nas dependencias, é un *panic*; en *release*, a columna dá a volta e o cursor desaparece. Atopouno a revisión do código, e compróbao a proba `the_cursor_on_a_lone_right_half_stays_in_its_cell`.
- **`take_changes`**, desde a subfase 1.6, responde primeiro as preguntas pendentes ([As preguntas](#as-preguntas-polas-cores-e-polo-tamaño)). Despois colle os `Changes` da cola e déixaa baleira: `mem::take` devolve o valor e deixa no seu sitio o valor por defecto. O título só se move se cambiou.

### Os axudantes

- **`rgb`**, desde a subfase 1.6, converte unha cor de `alacritty_terminal` nun `Rgb` de meiga. Úsana `palette` e `color`.
- **`lock`** bloquea a cola. Se un fío fallase (*panic*) mentres a tiña bloqueada, o `Mutex` quedaría «envelenado» (*poisoned*), e `lock()` daría un erro desde entón. Ningún cambio da cola pode quedar a medias, así que `unwrap_or_else(PoisonError::into_inner)` cóllea igual.
- **`at_least_min_size`**: o tamaño, con polo menos 2 columnas e 1 fila.
- **`copy_row`** copia unha fila que se ve e marca as celas que cambiaron. Desde a subfase 1.8 recibe tamén a selección (`selection`), e as celas que están nela levan `CellFlags::SELECTED`:

  ```rust
  let selected = selection.map_or(Columns::default(), |range| {
      selected_cells(range, point.line, sources, columns)
  });
  …
  for ((cell, source), column) in cells.iter_mut().zip(sources).zip(0..columns) {
      let char = if source.flags.contains(Flags::WIDE_CHAR) {
          CellChar::WideLeft(source.c)
      } else if source.flags.contains(Flags::WIDE_CHAR_SPACER) {
          wide.map_or(CellChar::Blank, CellChar::WideRight)
      } else if source.flags.contains(Flags::LEADING_WIDE_CHAR_SPACER)
          || matches!(source.c, ' ' | '\t')
      {
          CellChar::Blank
      } else {
          CellChar::Narrow(source.c)
      };
      wide = source.flags.contains(Flags::WIDE_CHAR).then_some(source.c);
      let mut flags = flags(source.flags);
      if selected.contains(column) {
          flags = flags | CellFlags::SELECTED;
      }
      …
  }
  ```

  - `viewport_to_point` converte a fila que se ve na liña da grella, contando o historial (`display_offset`), e `&grid[point.line][..]` son as súas celas. Desde a subfase 1.9, a vista pode mostrar o historial, e as súas filas cópianse coma as da pantalla.
  - Percorre á vez as celas da `Screen` e as de `alacritty_terminal` (`zip`), e converte cada unha:

    | Cela de `alacritty_terminal` | `CellChar` |
    |---|---|
    | Co *flag* `WIDE_CHAR` | `WideLeft(c)`. |
    | Co *flag* `WIDE_CHAR_SPACER` | `WideRight(c)`, co carácter da cela anterior se esta é a súa metade esquerda; se non, `Blank`. |
    | Co *flag* `LEADING_WIDE_CHAR_SPACER`, un espazo ou un tabulador | `Blank`. |
    | Calquera outra | `Narrow(c)`. |

  - **As metades soltas.** Inserir ou borrar caracteres (`ESC [ @` e `ESC [ P`) move as celas dunha fila e pode separar as dúas metades dun carácter ancho: unha metade dereita pode quedar sen a esquerda, mesmo na columna 0. `wide` lembra o carácter da cela anterior, se é unha metade esquerda; sen ela, a metade dereita é unha cela en branco.
  - **`LEADING_WIDE_CHAR_SPACER`**: cando un carácter ancho non cabe ao final dunha fila, vai á seguinte, e a última cela queda baleira con este *flag*.
  - **O tabulador**: `alacritty_terminal` garda `\t` na cela onde empezou, para poder copiar o texto con tabuladores; vese como unha cela en branco.
  - Cada cela nova compárase coa que había, e só se cambia se é distinta. `changed` acumula as columnas, da primeira á última que cambiaron, e `mark_changed` engádeas aos cambios da fila.
  - **A selección.** `selected` son as columnas desta fila que están na selección, ningunha se non hai selección (`map_or`). `SELECTED` engádese aos estilos da cela, así que unha cela que entra ou sae da selección é distinta da que había, e márcase coma calquera outro cambio. `let mut flags = flags(source.flags);` chama á función `flags` e garda o resultado nunha variábel co mesmo nome, que oculta a función ata o final da volta do bucle.
- **`selected_cells`**, desde a subfase 1.8, dá esas columnas: as que colle a selección na liña da grella `line` ([`selected_columns`](#cratesmeiga-coresrctermselectionrs)), e un carácter ancho enteiro se a selección colle só unha das súas metades:

  ```rust
  fn selected_cells(
      range: &SelectionRange,
      line: Line,
      sources: &[TermCell],
      columns: u16,
  ) -> Columns {
      let mut selected = selection::selected_columns(range, line, columns);
      if selected.is_empty() {
          return selected;
      }
      let flags_at = |column: u16| {
          sources
              .get(usize::from(column))
              .map_or(Flags::empty(), |cell| cell.flags)
      };
      if selected.start > 0 && flags_at(selected.start).contains(Flags::WIDE_CHAR_SPACER) {
          selected.start -= 1;
      }
      if selected.end < columns && flags_at(selected.end - 1).contains(Flags::WIDE_CHAR) {
          selected.end += 1;
      }
      selected
  }
  ```

  - Se a selección empeza na metade dereita dun carácter ancho (`WIDE_CHAR_SPACER`), empeza unha columna antes; e se remata na metade esquerda (`WIDE_CHAR`), unha columna despois. O texto da selección xa leva o carácter enteiro: así a pantalla mostra o mesmo que se copia ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).
  - `flags_at` le os *flags* dunha cela, ou ningún se a columna non existe: `get` nunca falla.
- **`color`**: as 16 cores con nome (`Named`) son as 16 primeiras da paleta, de `Indexed(0)` a `Indexed(15)`. Os outros nomes son as cores por defecto do texto e do fondo, e outras que só usa o renderizador do Alacritty (a do cursor, as atenuadas…), que ningunha secuencia lle dá a unha cela: `Default`. `Indexed` pasa igual, e `Spec` (*truecolor*), con `rgb`. `Ok(index @ 0..16)` é un patrón: o valor ten que estar entre 0 e 15, e `@` gárdao en `index`.
- **`flags`** traduce os estilos cunha táboa (`MAP`) de 11 pares, só os que debuxa meiga: os *flags* internos de `alacritty_terminal`, como os das metades dos caracteres anchos, non pasan. `fold` xúntaos con `|`. `SELECTED` non está na táboa: non é un estilo de `alacritty_terminal`, e pono `copy_row`.

As probas están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs).

## `crates/meiga-core/src/term/resize.rs`

Desde a subfase 1.9: o tamaño novo da pantalla principal, coa pantalla onde a colocan o anfitrión do ConPTY e o Windows Terminal, para que as filas do programa e as do terminal sexan as mesmas ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). Chámano [`Terminal::resize`](#o-tamaño-resize) e, cando o programa volta á pantalla principal, [`Hooked`](#cratesmeiga-coresrctermhandlerrs).

**O problema.** Cando cambia o tamaño, o ConPTY volta repartir as liñas longas do seu búfer (*reflow*), que non ten historial: o texto queda arriba, e medra cara abaixo. Non lle escribe nada ao terminal: supón que colocou o texto coma el, e a seguinte vez que un programa lle pide os datos do búfer, pregúntalle onde ten o cursor (`ESC [ 6 n`) e colle esa posición ([Tecnoloxías › Cambiar o tamaño do ConPTY](../tecnoloxias.md#48-cambiar-o-tamaño-do-conpty-o-seu-reflow-e-a-pregunta-polo-cursor)). `alacritty_terminal` reparte a pantalla e o historial xuntos, e mantén abaixo o cursor: cando a pantalla medra, trae liñas do historial, e cando as liñas se parten, empurra as de arriba ao historial, aínda que haxa filas baleiras debaixo ([Tecnoloxías › O historial](../tecnoloxias.md#49-o-historial-de-alacritty_terminal-a-vista-e-o-reflow)). Entón, o que escribe despois o programa cae noutras filas, e queda lixo na pantalla.

**A solución.** Despois do *reflow* de `alacritty_terminal`, a pantalla móvese onde a pon o Windows Terminal (`Terminal::UserResize`): a súa primeira fila é aquela onde foi parar a primeira de antes, salvo que o texto e o cursor xa non caiban debaixo dela; entón, a súa última fila é a máis baixa con texto, ou a do cursor se está máis abaixo. O cursor sempre queda na pantalla, e as liñas que pasaron ao historial quedan nel. Para saber onde foron parar a primeira fila e o cursor, `resize` marca as súas celas antes de repartir: non se pode fiar do cursor de `alacritty_terminal`, que, cando moitas filas debaixo del se parten, sae da pantalla por arriba e queda na primeira fila, que xa non é a súa.

**Na pantalla alternativa**, o ConPTY non reparte a principal: dálle o último tamaño unha vez, cando o programa volta a ela. meiga garda á parte a pantalla principal ata entón, e así tampouco reparte o seu historial mentres corre un programa a pantalla completa.

**Como se comproba.** Ademais das probas de [`tests.rs`](#cratesmeiga-coresrctermtestsrs), `check-resize.ps1` compara na CI, despois de cada cambio de tamaño, a pantalla de meiga co búfer da consola do ConPTY, que le un script de PowerShell dentro de meiga con `ReadConsoleOutputCharacterW`: a 70 × 20, 45 × 14, 90 × 18 e 60 × 24 celas; na pantalla alternativa, a 50 × 16; e, despois dun cambio de tamaño coa caixa de mensaxe aberta, a 66 × 15 ([Código › ferramentas](ferramentas.md#toolscheck-resizeps1)).

### As marcas: `MARK` e `CURSOR`

```rust
const MARK: Flags = Flags::from_bits_retain(0x8000);

const CURSOR: char = '\u{FFFF}';
```

- **`MARK`**: un *flag* das celas que `alacritty_terminal` 0.26.0 non usa, o último dos 16 bits de `Flags` (`0x8000`): os seus, os estilos e as marcas dos caracteres anchos e das liñas partidas, chegan ata `0x4000`. Marca as dúas celas que segue o cambio de tamaño: a primeira da pantalla e a do cursor. `from_bits_retain` crea o valor con ese bit aínda que non sexa un *flag* coñecido, e pódese usar nunha constante. Nunca chega á pantalla: `resize` quita as marcas antes de rematar, e `copy_row` só copia os estilos da súa táboa ([Os axudantes](#os-axudantes)).
- **`CURSOR`**: o carácter da cela do cursor mentres está marcada, U+FFFF, un dos que Unicode reserva para usos internos (*noncharacters*), que os programas non escriben. Fai que a cela estea ocupada. Para `alacritty_terminal`, unha cela cun espazo, sen cores nin estilos, está baleira, tamén con `MARK`, e o *reflow* non conserva as celas baleiras do final das filas que parte: a do cursor podería perderse. Con `CURSOR`, o *reflow* consérvaa coma texto, e pasa á fila seguinte cos brancos de antes dela, coma fai o ConPTY cos brancos antes do cursor. Ao final, a cela recupera o seu carácter.

### `resize`: a pantalla onde a pon o ConPTY

```rust
pub(super) fn resize(term: &mut Term<Listener>, size: GridSize) {
    let at_bottom = term.grid().display_offset() == 0;
    let rows = i32::from(size.rows);
    let old_rows = term.grid().screen_lines() as u16;
    let marks = Marks::set(term.grid_mut());
    // The width first, with the same rows: alacritty_terminal removes no row
    // when it reflows.
    if usize::from(size.columns) != term.grid().columns() {
        term.resize(TermSize(GridSize::new(size.columns, old_rows)));
    }
    if let Some(cursor) = marks.find(term.grid()).cursor {
        term.grid_mut().cursor.point = cursor;
    }
    if size.rows < old_rows {
        // alacritty_terminal removes the rows at the bottom: before that, the
        // screen starts where it has to, so that they are the ones that
        // ConPTY removes too.
        let top = window_top(term.grid(), rows, marks.find(term.grid()).top);
        move_screen(term.grid_mut(), top);
        term.resize(TermSize(size));
    } else {
        term.resize(TermSize(size));
        let top = window_top(term.grid(), rows, marks.find(term.grid()).top);
        move_screen(term.grid_mut(), top);
        // Back to the size, without the rows at the bottom.
        let grid = term.grid_mut();
        let columns = grid.columns();
        grid.resize::<Color>(false, usize::from(size.rows), columns);
    }
    let found = marks.find(term.grid());
    marks.remove(term.grid_mut(), found);
    term.selection = None;
    let grid = term.grid_mut();
    grid.scroll_display(if at_bottom {
        Scroll::Bottom
    } else {
        // Where it was, as long as the history still reaches it.
        Scroll::Delta(0)
    });
}
```

**`resize`** cambia o tamaño da pantalla principal de `term` a `size`, que non é o que ten: nin [`Terminal::resize`](#o-tamaño-resize) nin [`bring_back_primary`](#a-pantalla-principal-á-parte) o chaman co mesmo tamaño. Paso a paso:

1. **As marcas.** Lembra se a vista mostraba a pantalla (`at_bottom`), e marca a primeira cela da pantalla e a do cursor ([`Marks::set`](#seguir-as-celas-marks-e-found)).
2. **Primeiro o ancho, coas mesmas filas.** Se cambian as columnas, `alacritty_terminal` reparte a pantalla e o historial coas filas de antes: ao repartir, non quita ningunha fila, e as que non caben na pantalla quedan no historial. Se cambiase tamén a altura na mesma chamada, `alacritty_terminal` quitaría ou traería filas primeiro, á súa maneira, co cursor abaixo, antes de saber onde foi parar o texto.
3. **O cursor, desde as marcas.** O cursor pasa á cela marcada con `CURSOR`, que pode estar no historial. `alacritty_terminal` sobe o cursor unha fila por cada fila nova debaixo del e, se moitas filas debaixo del se parten e o empurran por riba da pantalla, déixao na primeira fila, que xa non é a súa. Compróbao a proba `the_cursor_keeps_its_row_when_many_rows_below_it_wrap`.
4. **As filas.** [`window_top`](#onde-empeza-a-pantalla-window_top-e-last_text_row) di onde ten que empezar a pantalla, e [`move_screen`](#mover-a-pantalla-move_screen) móvea alí:
   - **con menos filas**, a pantalla móvese primeiro, e despois `alacritty_terminal` quita as filas que sobran abaixo: así son as mesmas que quita o ConPTY;
   - **con máis filas, ou coas mesmas**, primeiro cambia o tamaño: `alacritty_terminal` medra trazendo filas do historial, co cursor abaixo. Despois, a pantalla móvese: as filas que trouxo voltan ao historial, e entran filas baleiras abaixo. Se ten que empezar máis arriba, no historial, `move_screen` faina máis alta, e `Grid::resize` sen *reflow* (`false`) devólvea ao seu tamaño, sen as filas de abaixo. Coas mesmas filas, o primeiro `term.resize` non fai nada: o ancho xa cambiou.
5. **O final.** As marcas vanse, e a cela do cursor recupera o seu carácter ([`Marks::remove`](#seguir-as-celas-marks-e-found)). A selección vaise, coma no Windows Terminal: `alacritty_terminal` só a quita se cambian as columnas. E a vista queda abaixo se estaba abaixo, ou onde estaba, mentres o historial chegue a ela: `Delta(0)` non a move ningunha fila, pero axústaa ao historial, que pode ser máis curto. Compróbao a proba `a_resize_keeps_the_view_inside_the_history`.

- `old_rows` cabe nun `u16`, porque as filas da grella veñen dun `GridSize`; e `rows` é un `i32`, coma as liñas de `alacritty_terminal` (`Line`), negativas no historial.
- `Scroll`, neste arquivo, é o de `alacritty_terminal`, e o último `scroll_display` é o da grella, non o do `Term`: non dana nada, pero o cambio de tamaño xa danou a pantalla enteira.
- `::<Color>`, en `Grid::resize` e en `Grid::scroll_up`, di o tipo co que a grella compara as celas cando borra filas (`ResetDiscriminant`): nas celas de `alacritty_terminal`, a cor do fondo.

### A pantalla principal á parte

```rust
pub(super) fn set_aside_primary(term: &mut Term<Listener>) -> Grid<Cell> {
    let lines = term.screen_lines();
    let columns = term.columns();
    // alacritty_terminal only gives the primary screen through swap_alt,
    // which clears the grid that becomes the alternate screen: the alternate
    // screen goes out before, and comes back after.
    let alternate = mem::replace(term.grid_mut(), Grid::new(lines, columns, 0));
    term.swap_alt();
    let primary = mem::replace(term.grid_mut(), Grid::new(lines, columns, 0));
    term.swap_alt();
    *term.grid_mut() = alternate;
    primary
}
```

**`set_aside_primary`** garda á parte a pantalla principal, mentres se ve a alternativa, e devólvea. No seu sitio deixa unha grella baleira sen historial (`Grid::new(lines, columns, 0)`), que segue o tamaño do terminal: ata que o programa volta á pantalla principal, un cambio de tamaño só cambia a alternativa e esa grella, que, baleira, se reparte sen custo.

- **Por que dous `swap_alt`.** `alacritty_terminal` garda a pantalla que non se ve nun campo privado, e só a dá `swap_alt`, que cambia dunha pantalla á outra. Pero, cando cambia á alternativa, borra a grella que pasa a ser a alternativa. Así que primeiro sae a alternativa, cunha grella baleira no seu sitio; o primeiro `swap_alt` mostra a principal, que sae tamén, con outra grella baleira no seu sitio; o segundo volta á pantalla alternativa, e borra a primeira grella baleira; e ao final a alternativa volta ao seu sitio. O terminal segue na pantalla alternativa, e cada `swap_alt` quita a selección e dana a pantalla enteira.
- `mem::replace` pon un valor no sitio doutro e devolve o que había: as grellas saen do `Term` sen copialas.
- Unha copia de meiga que non garda á parte a pantalla principal falla `check-resize.ps1`: `The primary screen, after the alternate one changed size: … line 1 is '26 …' in meiga and '25 …' in the console.`

```rust
pub(super) fn bring_back_primary(term: &mut Term<Listener>, primary: Grid<Cell>) -> bool {
    let size = GridSize::new(term.columns() as u16, term.screen_lines() as u16);
    let fits = primary.columns() == term.columns() && primary.screen_lines() == term.screen_lines();
    *term.grid_mut() = primary;
    if !fits {
        resize(term, size);
    }
    !fits
}
```

**`bring_back_primary`** devolve ao `Term` a pantalla principal que estaba á parte, cando o programa acaba de deixar a alternativa ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)), no sitio da grella baleira, que xa ten o tamaño do terminal. Coma fai entón o ConPTY, recibe ese tamaño, unha vez: se non é o que tiña, [`resize`](#resize-a-pantalla-onde-a-pon-o-conpty) reparte o texto e coloca a pantalla, coma se o tamaño cambiase nese momento. Os tamaños do medio non contan. Compróbao a proba `on_the_alternate_screen_the_primary_one_gets_the_last_size_when_the_program_leaves`.

- O tamaño do terminal lese antes de devolver a principal: o `Term` mide a grella que ten. Os `as u16` non cortan nada: as columnas e as filas viñeron dun `GridSize`.
- Desde a revisión do código, di se o tamaño cambiou: entón `alacritty_terminal` puxo a rexión de desprazamento na pantalla enteira, e [`Hooked`](#os-métodos-de-handler) fai o mesmo coa súa copia.

```rust
pub(super) fn new_primary(term: &mut Term<Listener>) {
    let lines = term.screen_lines();
    let columns = term.columns();
    *term.grid_mut() = Grid::new(lines, columns, SCROLLBACK_LINES);
}
```

**`new_primary`** pon unha pantalla principal nova despois dun reinicio (`ESC c`) na pantalla alternativa, cando a principal estaba á parte. O reinicio de `alacritty_terminal` volta á pantalla principal e baléiraa, co seu historial, pero a que baleira é a grella sen historial que ocupaba o seu sitio. A principal á parte vaise ([`Hooked`](#cratesmeiga-coresrctermhandlerrs)), e no sitio da grella sen historial queda unha nova, baleira e con historial (`SCROLLBACK_LINES`), coma despois dun reinicio na pantalla principal. `Grid::new` non reserva o historial: medra a medida que se enche. Compróbao a proba `a_reset_on_the_alternate_screen_leaves_an_empty_primary_screen_with_history`.

### Seguir as celas: `Marks` e `Found`

```rust
struct Marks {
    cursor_char: char,
    pending_wrap: bool,
    cursor_first: bool,
}

struct Found {
    top: Option<i32>,
    cursor: Option<Point>,
}
```

| Campo | Que é |
|---|---|
| `Marks::cursor_char` | O carácter que tiña a cela do cursor, que volta a ela ao final. |
| `Marks::pending_wrap` | O cursor esperaba despois da última columna para pasar á fila seguinte (`input_needs_wrap`): o seguinte carácter vai á fila de abaixo. |
| `Marks::cursor_first` | O cursor estaba na primeira cela da pantalla: as dúas marcas son unha. |
| `Found::top` | A fila da cela que era a primeira da pantalla, na pantalla ou no historial, se a atopou. |
| `Found::cursor` | A cela do cursor, se a atopou. |

Os tres métodos de `Marks` (`impl Marks`):

```rust
fn set(grid: &mut Grid<Cell>) -> Self {
    let first = Point::new(Line(0), Column(0));
    let cursor = grid.cursor.point;
    let pending_wrap = grid.cursor.input_needs_wrap;
    // Writing through the index of the row makes the cell occupied.
    grid[first].flags.insert(MARK);
    let cell = &mut grid[cursor];
    cell.flags.insert(MARK);
    Self {
        cursor_char: mem::replace(&mut cell.c, CURSOR),
        pending_wrap,
        cursor_first: cursor == first,
    }
}
```

**`set`** marca a primeira cela da pantalla con `MARK`, e a do cursor con `MARK` e `CURSOR`, e garda o que fai falta para desfacelo. `mem::replace` pon `CURSOR` na cela e devolve o carácter que tiña.

- **As celas ocupadas.** Cada fila de `alacritty_terminal` lembra ata que columna ten celas escritas (`occ`), e cando a borra só mira esas. Escribir a través do índice da fila (`grid[…]`) conta a cela coma escrita: se a fila se borra, a marca vaise con ela.

```rust
fn find(&self, grid: &Grid<Cell>) -> Found {
    let mut found = Found {
        top: None,
        cursor: None,
    };
    let top = -(grid.history_size() as i32);
    let rows = grid.screen_lines() as i32;
    for line in (top..rows).rev() {
        for (column, cell) in grid[Line(line)].into_iter().enumerate() {
            if !cell.flags.contains(MARK) {
                continue;
            }
            let is_cursor = cell.c == CURSOR;
            if is_cursor {
                found.cursor = Some(Point::new(Line(line), Column(column)));
            }
            if !is_cursor || self.cursor_first {
                found.top = Some(line);
            }
        }
        if found.top.is_some() && found.cursor.is_some() {
            break;
        }
    }
    found
}
```

**`find`** busca as celas marcadas de abaixo arriba, desde a última fila da pantalla ata a primeira do historial, e para cando atopou as dúas: case sempre están preto da pantalla. A do cursor é a que ten `CURSOR`; a outra, a que era a primeira, ou a do cursor se eran a mesma (`cursor_first`). `resize` chámaa tres veces, porque cada paso move as filas: despois do *reflow*, polo cursor; antes de mover a pantalla, pola primeira fila; e ao final, para quitar as marcas.

- `grid[Line(line)]` é unha fila, e `into_iter` percorre as súas celas; `enumerate` dá a columna de cada unha.
- `-(grid.history_size() as i32)` é a primeira fila do historial: ten como moito `SCROLLBACK_LINES`, e cabe nun `i32`.

```rust
fn remove(self, grid: &mut Grid<Cell>, found: Found) {
    if let Some(line) = found.top {
        for cell in &mut grid[Line(line)] {
            cell.flags.remove(MARK);
        }
    }
    let Some(point) = found.cursor else {
        return;
    };
    let cell = &mut grid[point];
    cell.c = self.cursor_char;
    cell.flags.remove(MARK);
    if !(0..grid.screen_lines() as i32).contains(&point.line.0) {
        return;
    }
    let last_column = Column(grid.columns() - 1);
    let cursor = &mut grid.cursor;
    // The next character goes after the one of the cell, maybe on the
    // same row now.
    if self.pending_wrap && point.column < last_column {
        cursor.point = Point::new(point.line, point.column + 1);
        cursor.input_needs_wrap = false;
    } else {
        cursor.point = point;
        cursor.input_needs_wrap = self.pending_wrap;
    }
}
```

**`remove`** quita as marcas: a da primeira cela, en toda a súa fila, porque o *reflow* pode levala a outra columna, cando a liña empezaba no historial e se xunta co seu anaco ao ensanchar; e a da cela do cursor, que recupera o seu carácter. Despois, se esa cela está na pantalla, o cursor pasa a ela:

- se esperaba despois da última columna, e a súa cela xa non está na última, vai á cela seguinte, e xa non espera: o seguinte carácter vai despois do da cela, quizais agora na mesma fila;
- se non, queda na cela, e espera se esperaba. Coma no xterm, un cursor que esperaba segue esperando aínda que a fila se estreite; no ConPTY, non.

Compróbao a proba `the_cursor_keeps_its_place_after_the_last_character_of_a_full_row`. `remove` recibe `self` por valor: despois, as marcas xa non serven. `let Some(point) = found.cursor else { return; };` (*let-else*) sae se non atopou a cela do cursor.

### Onde empeza a pantalla: `window_top` e `last_text_row`

```rust
fn window_top(grid: &Grid<Cell>, rows: i32, first: Option<i32>) -> i32 {
    let cursor = grid.cursor.point.line.0;
    let bottom = last_text_row(grid).map_or(cursor, |last| max(last, cursor));
    let mut top = bottom - rows + 1;
    if let Some(first) = first {
        top = max(top, first);
    }
    min(top, cursor)
}

fn last_text_row(grid: &Grid<Cell>) -> Option<i32> {
    let rows = grid.screen_lines() as i32;
    (0..rows).rev().find(|&line| {
        grid[Line(line)]
            .into_iter()
            .any(|cell| !matches!(cell.c, ' ' | '\t'))
    })
}
```

**`window_top`**: onde ten que empezar unha pantalla de `rows` filas, coma unha fila da pantalla de agora ou, negativa, do historial. Coma no Windows Terminal:

1. a fila máis baixa que ten que caber, `bottom`, é a última con texto ou a do cursor, a que estea máis abaixo;
2. a pantalla empeza onde foi parar a primeira fila, `first`, salvo que así `bottom` non caiba: entón, tan abaixo como faga falta (`bottom - rows + 1`);
3. e nunca debaixo do cursor: se o texto que hai debaixo del non cabe, a pantalla empeza na súa fila, e as filas que non caben vanse por abaixo.

Sen `first`, se a marca non apareceu, a pantalla remata en `bottom`.

**`last_text_row`**: a fila máis baixa da pantalla con texto, coma a mide o ConPTY: os espazos e os tabuladores non contan, aínda que teñan cor. Para `alacritty_terminal`, un espazo con cor ocupa a cela.

### Mover a pantalla: `move_screen`

```rust
fn move_screen(grid: &mut Grid<Cell>, top: i32) {
    let rows = grid.screen_lines();
    match top.cmp(&0) {
        Ordering::Greater => {
            let count = top as usize;
            grid.scroll_up::<Color>(&(Line(0)..Line(rows as i32)), count);
            grid.cursor.point.line -= count;
            grid.saved_cursor.point.line = max(grid.saved_cursor.point.line - count, Line(0));
        }
        Ordering::Less => {
            let columns = grid.columns();
            grid.resize::<Color>(false, rows + (-top) as usize, columns);
        }
        Ordering::Equal => {}
    }
}
```

**`move_screen`** move a pantalla para que a súa primeira fila sexa `top`:

- **máis abaixo**, con `top` positivo: `Grid::scroll_up`, coa pantalla enteira coma rexión, sobe as `top` primeiras filas ao historial, e entran filas baleiras abaixo, coma cando o programa escribe na última fila. O cursor sobe con elas, e tamén o cursor gardado (`ESC 7`), como moito ata a primeira fila;
- **máis arriba**, con `top` negativo, unha fila do historial: `Grid::resize` sen *reflow* (`false`) fai a pantalla `-top` filas máis alta, e énchea coas filas do historial que hai enriba dela, co cursor máis abaixo. Despois, `resize` devólvea ao tamaño novo, quitando as filas de abaixo.

Na súa rama, `top` e `-top` son positivos: os `as usize` non perden nada.

**Onde meiga e o ConPTY non coinciden.** Hai casos raros, que corrixe a seguinte escritura do programa ([ADR 0016 › Consecuencias](../adr/0016-o-historial-o-tamano-e-o-zoom.md#consecuencias)): unha liña que empezaba no historial xúntase coa súa parte da primeira fila ao ensanchar, coma no Windows Terminal, e no ConPTY non, porque non ten historial; os espazos con cor ao final dunha liña ocupan celas en `alacritty_terminal`, e o ConPTY quítaos ao repartir; e un cursor que esperaba despois da última columna segue esperando, coma no xterm, e no ConPTY non.

As probas están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs), coas do terminal: cinco do cambio de tamaño e tres da pantalla principal á parte.

## `crates/meiga-core/src/term/handler.rs`

Desde a subfase 1.9: o terminal de `alacritty_terminal` coma o ve o parser. O parser de `vte` chama a un `Handler` por cada orde da saída ([Tecnoloxías › `alacritty_terminal` e `vte`](../tecnoloxias.md#34-alacritty_terminal-e-vte-o-núcleo-do-terminal)). Ata a 1.8 era o propio `Term`; desde a 1.9 é `Hooked`, que lle pasa ao `Term` todas as ordes e, cando o programa deixa a pantalla alternativa ou reinicia o terminal, devolve a pantalla principal que un cambio de tamaño deixou á parte ([`term/resize.rs`](#cratesmeiga-coresrctermresizers), [ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). Desde a revisión do código, tamén deixa a vista do historial onde está cando se despraza unha rexión que non empeza arriba ([Manter a vista](#manter-a-vista-view_to_keep-e-keep_view)).

**Por que dentro do parser.** A pantalla principal ten que voltar no momento en que o programa deixa a alternativa: a saída que vén despois de `?1049l`, na mesma lectura ou na mesma actualización sincronizada (`?2026`), xa é para ela, e ten que caer nas súas filas, co tamaño novo. `feed` só volta cando o parser leu o bloque enteiro; `vte` garda a saída dunha actualización sincronizada e dálla ao `Handler` dunha vez, cando remata ou cando vence (`end_sync`); e ningún evento de `alacritty_terminal` avisa do cambio de pantalla. Despois de ler, sería tarde. Compróbao a proba `the_output_after_leaving_the_alternate_screen_reaches_the_primary_one`, nos tres casos.

### `Hooked`

```rust
pub(super) struct Hooked<'a> {
    pub term: &'a mut Term<Listener>,
    pub primary: &'a mut Option<Grid<Cell>>,
    pub region_below_top: &'a mut bool,
}
```

- **`Hooked`**: o terminal, a pantalla principal que un cambio de tamaño deixou á parte e, desde a revisión do código, se a rexión de desprazamento do programa empeza máis abaixo da primeira fila da pantalla: os campos `term`, `primary` e `region_below_top` do [`Terminal`](#terminal), prestados mentres le o parser. `feed` e `end_sync` crean un en cada lectura: son tres referencias, sen copiar nada.
- `'a` é o tempo de vida dos tres préstamos: `Hooked` non pode durar máis ca eles. Os campos son públicos, pero o tipo só se ve no módulo `term` (`pub(super)`).
- `Cell`, neste arquivo, é a cela de `alacritty_terminal`.

### Manter a vista: `view_to_keep` e `keep_view`

Desde a revisión do código da subfase 1.9. Cando se despraza unha rexión que non empeza na primeira fila da pantalla, as filas que saen dela non entran no historial, pero `alacritty_terminal` sube igual a vista ([O historial e a súa vista](#o-historial-e-a-súa-vista)): se a vista non está na pantalla, `Grid::scroll_up` súmalle as filas que despraza, entren ou non no historial (só se borra a rexión enteira, non a move). As ordes que poden desprazar así gardan antes onde está a vista e, despois, déixana alí.

```rust
impl Hooked<'_> {
    #[inline]
    fn view_to_keep(&self, below_top: bool) -> Option<usize> {
        if !below_top {
            return None;
        }
        let offset = self.term.grid().display_offset();
        (offset != 0).then_some(offset)
    }

    #[inline]
    fn keep_view(&mut self, view: Option<usize>) {
        let Some(offset) = view else {
            return;
        };
        let moved = self.term.grid().display_offset();
        if moved != offset {
            // Both are at most SCROLLBACK_LINES: they fit in an i32.
            self.term
                .scroll_display(Scroll::Delta(offset as i32 - moved as i32));
        }
    }
}
```

- **`view_to_keep(below_top)`**: onde está a vista, cantas filas por riba da pantalla, se a orde despraza desde unha fila que non é a primeira (`below_top`) e a vista non está na pantalla; se non, `None`. Coa vista na pantalla, `alacritty_terminal` non a move, e non hai nada que manter. Fóra dese caso, o custo é ler un `bool`.
- **`keep_view(view)`**: despois da orde, se a vista se moveu, lévaa onde estaba coa diferenza (`Delta`), negativa: cara abaixo. As dúas contas caben nun `i32`, porque a vista non pasa de `SCROLLBACK_LINES` filas. `scroll_display` dana a pantalla enteira, pero só cando a vista estaba arriba e a rexión se desprazou. A vista volta a unha fila que xa mostraba, así que tampouco pasa do principio do historial.
- **Orde a orde.** Ao final da lectura sería tarde: na mesma lectura, outras ordes poden meter filas no historial, que si teñen que subir a vista, e despois xa non se sabe canto subiu por cada unha.

### Seguir a rexión: `follow_region` e `follow_column_mode`

```rust
#[inline]
fn follow_region(&mut self, top: usize, bottom: Option<usize>) {
    if top < bottom.unwrap_or_else(|| self.term.screen_lines()) {
        *self.region_below_top = top > 1;
    }
}

#[inline]
fn follow_column_mode(&mut self, mode: PrivateMode) {
    if matches!(mode, PrivateMode::Named(NamedPrivateMode::ColumnMode)) {
        self.follow_region(1, None);
    }
}
```

Desde a revisión do código da subfase 1.9. `alacritty_terminal` garda a rexión de desprazamento nun campo privado, `scroll_region`, e non di onde empeza: `region_below_top` séguea.

- **`follow_region`** fai coa copia o mesmo ca `alacritty_terminal` coa rexión cando o programa a pon desde a fila `top`, onde a primeira é 1, ata `bottom` ou, sen `bottom`, ata a última fila: se `top` non está por riba desa fila, `alacritty_terminal` ignora a orde, e a copia tampouco cambia. Se non, a rexión empeza máis abaixo da primeira fila se `top > 1`. `unwrap_or_else` só le o alto da pantalla se fai falta.
- **`follow_column_mode`**: DECCOLM (`?3h` e `?3l`) cambiaba o terminal a 132 ou a 80 columnas. `alacritty_terminal` non cambia as columnas, pero borra a pantalla e pon a rexión na pantalla enteira, coma `CSI 1 r`, desde dentro do `Term`, sen pasar polo `Handler`. `matches!` compara o modo co patrón de DECCOLM. Cunha soa fila, `1` non está por riba da última, e `alacritty_terminal` deixa a rexión que había: `follow_region(1, None)` tamén. Atopouno a segunda volta da revisión do código.
- **A copia segue todos os cambios.** `alacritty_terminal` só cambia a rexión en catro sitios: ao crear o terminal, ao cambiar o seu tamaño, que a pon na pantalla enteira, no reinicio e en `set_scrolling_region`, que tamén chama DECCOLM. Deixar a pantalla alternativa non a cambia. `region_below_top` segue os catro: `false` ao crear o [`Terminal`](#terminal), en [`Terminal::resize`](#o-tamaño-resize) e cando `bring_back_primary` cambia o tamaño, en `reset_state`, e `follow_region` con `set_scrolling_region` e con DECCOLM ([Os métodos de `Handler`](#os-métodos-de-handler)).

### Os métodos de `Handler`

```rust
macro_rules! pass {
    ($(fn $name:ident(&mut self $(, $argument:ident: $type:ty)*);)*) => {
        $(
            #[inline]
            fn $name(&mut self $(, $argument: $type)*) {
                <Term<Listener> as Handler>::$name(self.term $(, $argument)*);
            }
        )*
    };
}

// A method that Handler adds in a new version would do nothing here, without
// any error: the lint asks for all of them.
#[deny(clippy::missing_trait_methods)]
impl Handler for Hooked<'_> {
    pass! {
        fn set_title(&mut self, title: Option<String>);
        fn set_cursor_style(&mut self, style: Option<CursorStyle>);
        fn set_cursor_shape(&mut self, shape: CursorShape);
        fn goto(&mut self, line: i32, column: usize);
        fn goto_line(&mut self, line: i32);
        fn goto_col(&mut self, column: usize);
        fn insert_blank(&mut self, count: usize);
        fn move_up(&mut self, rows: usize);
        fn move_down(&mut self, rows: usize);
        fn identify_terminal(&mut self, intermediate: Option<char>);
        fn device_status(&mut self, kind: usize);
        fn move_forward(&mut self, columns: usize);
        fn move_backward(&mut self, columns: usize);
        fn move_down_and_cr(&mut self, rows: usize);
        fn move_up_and_cr(&mut self, rows: usize);
        fn backspace(&mut self);
        fn carriage_return(&mut self);
        fn bell(&mut self);
        fn substitute(&mut self);
        fn set_horizontal_tabstop(&mut self);
        fn scroll_down(&mut self, rows: usize);
        fn insert_blank_lines(&mut self, count: usize);
        fn erase_chars(&mut self, count: usize);
        fn delete_chars(&mut self, count: usize);
        fn move_backward_tabs(&mut self, count: u16);
        fn move_forward_tabs(&mut self, count: u16);
        fn save_cursor_position(&mut self);
        fn restore_cursor_position(&mut self);
        fn clear_line(&mut self, mode: LineClearMode);
        fn clear_screen(&mut self, mode: ClearMode);
        fn clear_tabs(&mut self, mode: TabulationClearMode);
        fn set_tabs(&mut self, interval: u16);
        fn reverse_index(&mut self);
        fn terminal_attribute(&mut self, attribute: Attr);
        fn set_mode(&mut self, mode: Mode);
        fn unset_mode(&mut self, mode: Mode);
        fn report_mode(&mut self, mode: Mode);
        fn report_private_mode(&mut self, mode: PrivateMode);
        fn set_keypad_application_mode(&mut self);
        fn unset_keypad_application_mode(&mut self);
        fn set_active_charset(&mut self, index: CharsetIndex);
        fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset);
        fn set_color(&mut self, index: usize, color: Rgb);
        fn dynamic_color_sequence(&mut self, prefix: String, index: usize, terminator: &str);
        fn reset_color(&mut self, index: usize);
        fn clipboard_store(&mut self, clipboard: u8, base64: &[u8]);
        fn clipboard_load(&mut self, clipboard: u8, terminator: &str);
        fn decaln(&mut self);
        fn push_title(&mut self);
        fn pop_title(&mut self);
        fn text_area_size_pixels(&mut self);
        fn text_area_size_chars(&mut self);
        fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>);
        fn set_mouse_cursor_icon(&mut self, icon: CursorIcon);
        fn report_keyboard_mode(&mut self);
        fn push_keyboard_mode(&mut self, mode: KeyboardModes);
        fn pop_keyboard_modes(&mut self, count: u16);
        fn set_keyboard_mode(&mut self, mode: KeyboardModes, behavior: KeyboardModesApplyBehavior);
        fn set_modify_other_keys(&mut self, mode: ModifyOtherKeys);
        fn report_modify_other_keys(&mut self);
        fn set_scp(&mut self, path: ScpCharPath, update: ScpUpdateMode);
    }

    #[inline]
    fn input(&mut self, c: char) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::input(self.term, c);
        self.keep_view(view);
    }

    #[inline]
    fn put_tab(&mut self, count: u16) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::put_tab(self.term, count);
        self.keep_view(view);
    }

    #[inline]
    fn linefeed(&mut self) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::linefeed(self.term);
        self.keep_view(view);
    }

    #[inline]
    fn newline(&mut self) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::newline(self.term);
        self.keep_view(view);
    }

    #[inline]
    fn scroll_up(&mut self, rows: usize) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::scroll_up(self.term, rows);
        self.keep_view(view);
    }

    #[inline]
    fn delete_lines(&mut self, count: usize) {
        let view = self.view_to_keep(self.term.grid().cursor.point.line.0 != 0);
        <Term<Listener> as Handler>::delete_lines(self.term, count);
        self.keep_view(view);
    }

    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        <Term<Listener> as Handler>::set_scrolling_region(self.term, top, bottom);
        self.follow_region(top, bottom);
    }

    #[inline]
    fn set_private_mode(&mut self, mode: PrivateMode) {
        <Term<Listener> as Handler>::set_private_mode(self.term, mode);
        self.follow_column_mode(mode);
    }

    #[inline]
    fn unset_private_mode(&mut self, mode: PrivateMode) {
        <Term<Listener> as Handler>::unset_private_mode(self.term, mode);
        self.follow_column_mode(mode);
        if !self.term.mode().contains(TermMode::ALT_SCREEN)
            && let Some(primary) = self.primary.take()
            && resize::bring_back_primary(self.term, primary)
        {
            *self.region_below_top = false;
        }
    }

    #[inline]
    fn reset_state(&mut self) {
        <Term<Listener> as Handler>::reset_state(self.term);
        *self.region_below_top = false;
        if self.primary.take().is_some() {
            resize::new_primary(self.term);
        }
    }
}
```

- **`pass!`**, unha macro declarativa (`macro_rules!`), escribe os métodos que só lle pasan a orde ao `Term`: recibe as súas firmas, coma no trait, e escribe cada un cun corpo que chama ao mesmo método do `Term`, cos mesmos argumentos. `$name:ident` e `$argument:ident` recollen identificadores, `$type:ty` un tipo, e `$( … )*` repítese para cada método e para cada argumento.
- `<Term<Listener> as Handler>::$name(self.term …)` chama ao método do trait `Handler` do `Term` coa sintaxe completa, que non deixa dúbida de cal é. `self.term` xa é unha referencia mutábel ao `Term`.
- `#[inline]` suxírelle ao compilador que meta cada método no sitio da chamada: o parser é xenérico no seu `Handler`, e así pasar polo `Hooked` non ten por que engadir unha chamada.
- **Todos os métodos.** Os 71 métodos de `Handler` en `vte` 0.15.0 teñen unha implementación por defecto que non fai nada. Se unha versión nova engadise outro, `Hooked` usaría esa, e a orde non chegaría ao `Term`, sen ningún erro. O *lint* de Clippy `missing_trait_methods`, do grupo *restriction*, que non está activo por defecto, avisa de cada método dun trait que unha implementación non escribe; co nivel `deny`, só neste `impl`, é un erro, e a CI falla ([Tecnoloxías › *Lints*](../tecnoloxias.md#7-lints-rustc-e-clippy)). `pass!` escribe 61 deles, e os outros dez fan algo máis.
- **`input`, `put_tab`, `linefeed`, `newline` e `scroll_up`**, desde a revisión do código: as ordes que desprazan a rexión de desprazamento cara arriba, desde a súa primeira fila. Un carácter despois da última columna pasa á fila seguinte (`input`), e tamén un tabulador (`put_tab`); ao final da rexión, iso despráza, coma unha liña nova (`linefeed`, que fan LF, VT, FF, `ESC D` e `ESC E`). `newline` é unha liña nova e, no modo `LNM`, un retorno de carro, pero o parser de `vte` 0.15 non o chama. E `CSI n S` despraza a rexión `n` filas (`scroll_up`). Se a rexión empeza máis abaixo da primeira fila da pantalla (`region_below_top`), a vista queda onde estaba ([Manter a vista](#manter-a-vista-view_to_keep-e-keep_view)).
- **`delete_lines`** (`CSI n M`) borra `n` filas desde a do cursor e sobe as de debaixo, dentro da rexión: despraza desde a fila do cursor. Se non é a primeira da pantalla, non entra nada no historial, aínda que a rexión empece arriba, e a vista queda onde estaba. `cursor.point.line.0` é a fila do cursor na pantalla, desde 0.
- **`set_scrolling_region`** (`ESC [ superior ; inferior r`) pásalle a orde ao `Term`, e [`follow_region`](#seguir-a-rexión-follow_region-e-follow_column_mode) segue a rexión nova.
- **`set_private_mode`** e **`unset_private_mode`**, con DECCOLM (`?3h` e `?3l`), seguen tamén a rexión que pon o `Term` ([`follow_column_mode`](#seguir-a-rexión-follow_region-e-follow_column_mode)).
- **`unset_private_mode`**: despois de que o `Term` desactive o modo, se xa non está na pantalla alternativa e hai unha principal á parte, devólvea con [`bring_back_primary`](#a-pantalla-principal-á-parte), que lle dá o tamaño do terminal. Só pasa con `?1049l`, o único modo co que `alacritty_terminal` deixa a pantalla alternativa. `take` colle a principal e deixa `None` no seu sitio. Se o tamaño da principal cambia, a rexión volta ser a pantalla enteira.
- **`reset_state`**: o reinicio (`ESC c`) de `alacritty_terminal` deixa tamén a pantalla alternativa e baleira a principal, co seu historial, e a rexión volta ser a pantalla enteira. Se a principal estaba á parte, o reinicio baleirou a grella que ocupaba o seu sitio: entón a principal á parte vaise, e [`new_primary`](#a-pantalla-principal-á-parte) pon unha nova, con historial.
- `if … && let Some(primary) = self.primary.take() && resize::bring_back_primary(…)` encadea condicións e un `if let` (*let chains*, da edición 2024): cada unha só se avalía se se cumpre a anterior, así que `take` só se chama fóra da pantalla alternativa, e `bring_back_primary` só se hai unha principal á parte.
- Un programa que activa a pantalla alternativa estando xa nela (`?1049h` dúas veces) non cambia nada en `alacritty_terminal`: a principal segue á parte ata que o programa volta a ela ([ADR 0016 › Consecuencias](../adr/0016-o-historial-o-tamano-e-o-zoom.md#consecuencias)).

## `crates/meiga-core/src/term/screen.rs`

O que mostra o terminal, en tipos de meiga: as celas, as súas cores e estilos, o cursor, a paleta e as celas que cambiaron. Desde a subfase 1.6, a vista do terminal de `meiga.exe` debuxa desde eiquí, sen coñecer `alacritty_terminal` ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)). Desde a 1.8, as celas din tamén se están na selección.

```rust
pub enum Color {
    Default,
    Indexed(u8),
    Rgb(Rgb),
}

impl Color {
    pub fn rgb(self, palette: &Palette, default: Rgb) -> Rgb {
        match self {
            Color::Default => default,
            Color::Indexed(index) => palette.colors[usize::from(index)],
            Color::Rgb(rgb) => rgb,
        }
    }
}
```

- **`Color`**: a cor do texto ou do fondo dunha cela, tal como a deu o programa: a por defecto, unha das 256 da paleta (0–7 as normais, 8–15 as brillantes, 16–231 un cubo de 6 × 6 × 6 e 232–255 grises), ou unha dada polos seus compoñentes (*truecolor*).
  - Desde a subfase 1.6, **`rgb`** convértea na cor que se debuxa, coa paleta da pantalla. A cor por defecto depende de onde está, a do texto ou a do fondo, así que a dá quen chama (`default`). `usize::from(index)`: un `u8` sempre está entre as 256 cores, así que o índice nunca falla.
  - **Por que o número, e non a cor.** Se o programa cambia a paleta, o texto que xa está na pantalla cambia de cor, coma no xterm; e as celas son máis pequenas ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **`CellFlags`**: os estilos dunha cela, un bit cada un nun `u16`: negra, atenuada, cursiva, cinco subliñados (simple, dobre, ondulado, de puntos e de trazos), riscada, inversa (o texto e o fondo intercambian as cores) e oculta (só se debuxa o fondo). `contains`, `is_empty` e o operador `|` (o trait `BitOr`) chegan para combinalos e preguntar por eles.
  - Desde a subfase 1.8, **`SELECTED`**, o bit 11: a cela está na selección, e o seu fondo ten a cor da selección da paleta, co texto na súa cor ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). Non o pon ningún programa: márcao a fachada ao copiar as celas ([`copy_row`](#os-axudantes)). Ir nos estilos fai que unha cela que entra ou sae da selección sexa unha cela que cambiou, sen outro mecanismo.
- **`Cell`**: unha cela: o carácter (`CellChar`, de [`text.rs`](#cratesmeiga-coresrctextrs)), a cor do texto, a do fondo e os estilos. A cela por defecto (`Default`) é unha cela en branco, coas cores por defecto e sen estilos.
- **`CursorShape`**: bloque, subliñado, barra vertical, o contorno dun bloque, ou oculto.
- **`Cursor`**: a fila, a columna, a forma e se pestanexa: por defecto si, salvo que o programa pida un cursor fixo. meiga só o fai pestanexar se Windows o permite ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)).
- **`Columns`**: as columnas dunha fila, de `start` (incluída) a `end` (excluída); ningunha se `end` non está despois de `start`. `union` dá as que cobren as dúas. Desde a subfase 1.8, `contains` di se unha columna está entre elas: `copy_row` pregúntao, cela a cela, coas columnas seleccionadas.

```rust
pub struct Screen {
    size: GridSize,
    cells: Vec<Cell>,
    changed: Vec<Columns>,
    all_changed: bool,
    cursor: Cursor,
    palette: Palette,
}
```

**`Screen`** é unha copia do que mostra o terminal, e das celas que cambiaron desde o último `clear_changes`. Está pensada para gardarse entre fotogramas: o terminal só copia nela o que cambiou, e a súa memoria reutilízase.

- `cells`: as celas, fila tras fila, nun só `Vec`: a da fila `f` e a columna `c` está na posición `f × columnas + c`. Unha soa reserva de memoria, coas filas seguidas.
- `changed`: as columnas que cambiaron en cada fila. É un só intervalo por fila: se cambian as columnas 2 e 7, debúxanse tamén as do medio. Chega para o que fan os programas, que escriben seguido, e non reserva memoria.
- `all_changed`: cambiaron todas as celas, por exemplo porque cambiou o tamaño ou a paleta.
- `palette`, desde a subfase 1.6: as cores coas que se debuxan as celas, as do terminal na última copia. A vista do terminal debuxa desde a súa `Screen` sen o *lock* do terminal ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)): por iso a copia leva tamén as cores.

| Método | Que fai |
|---|---|
| `new` | Unha pantalla baleira, de 0 × 0, coas cores de meiga (`Palette::MEIGA`). É `const` e non reserva nada: a memoria chega co primeiro cambio de tamaño. |
| `size`, `cursor` | O tamaño e o cursor. |
| `palette` | Desde a 1.6: as cores coas que se debuxan as celas, as do tema cos cambios do programa. |
| `row(row)` | As celas dunha fila, ou ningunha se está fóra. |
| `all_changed` | Se cambiaron todas as celas. |
| `changes()` | As filas que cambiaron, coas súas columnas; se cambiaron todas, cada fila enteira. |
| `clear_changes` | Esquece os cambios, despois de debuxalos. |
| `row_text(row)` | O texto dunha fila sen os espazos do final: cada carácter unha vez, tamén os anchos, e as celas en branco como espazos. Úsano as probas e o exemplo `replay`. |
| `mark_cursor_changed` | Desde a 1.6: engade aos cambios as celas do cursor, se se debuxa (máis abaixo). |

Estes cinco só os usa a fachada: son `pub(super)`, visíbeis no módulo pai, `term`, e non fóra del.

| Método | Que fai |
|---|---|
| `resize(size)` | Cambia o tamaño: todas as celas en branco e cambiadas. Reutiliza a memoria se cabe. |
| `cells_mut(row, columns)` | As celas dunha fila, para cambialas. |
| `mark_changed(row, columns)` | Engade columnas aos cambios dunha fila. |
| `set_cursor` | Cambia o cursor. |
| `set_palette` | Desde a 1.6: colle a paleta do terminal, e marca o que cambia con ela (máis abaixo). |

- `changes()` devolve `impl Iterator<…> + '_`, como `cells` en `text.rs`: un iterador que toma prestada a pantalla, sen reservar memoria. `filter_map` salta as filas sen cambios, e `then_some` converte a condición nun `Option`.

```rust
pub fn mark_cursor_changed(&mut self) {
    let Cursor {
        row, column, shape, ..
    } = self.cursor;
    if shape != CursorShape::Hidden {
        let end = column.saturating_add(2).min(self.size.columns);
        self.mark_changed(row, Columns::new(column, end));
    }
}

pub(super) fn set_palette(&mut self, palette: &Palette) {
    if self.palette.colors != palette.colors
        || self.palette.foreground != palette.foreground
        || self.palette.background != palette.background
        || self.palette.selection != palette.selection
    {
        self.all_changed = true;
    } else if self.palette.cursor != palette.cursor {
        self.mark_cursor_changed();
    } else {
        return;
    }
    self.palette.clone_from(palette);
}
```

- **`mark_cursor_changed`** marca as celas do cursor: dúas, por se está sobre un carácter ancho, e só unha na última columna (`min`); ningunha se está oculto. Ata a subfase 1.5, `copy_damage` marcaba el mesmo estas celas. Na 1.6 o código pasou eiquí, e é público, porque a vista do terminal tamén o necesita cando o cursor cambia de aspecto sen moverse: ao pestanexar, e ao gañar ou perder o foco a xanela, cando pasa a ser un rectángulo baleiro ou volta á súa forma ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)). `let Cursor { row, column, shape, .. } = self.cursor;` desestrutura o cursor en tres variábeis; `..` ignora o resto dos campos.
- **`set_palette`** colle a paleta do terminal, en cada copia:
  - se cambiaron as 256 cores, a do texto, a do fondo ou, desde a subfase 1.8, a da selección, cambian todas as celas: calquera pode ter unha cor que cambiou. Tamén cambian as marxes da xanela, que teñen a cor do fondo: con `all_changed`, a vista do terminal redebuxa a xanela enteira. A da selección só cambia co tema (`set_theme`), que polo de agora só cambian as probas;
  - se só cambiou a do cursor, só cambian as celas do cursor: ningunha outra a usa;
  - se non cambiou nada, o caso de case todas as copias, non copia nada.

  `clone_from` copia a paleta nova sobre a que xa había, sen crear outra.

**Probas.** Cinco:

| Proba | Que comproba |
|---|---|
| `a_new_screen_is_empty` | Unha pantalla nova mide 0 × 0, sen filas nin cambios. |
| `after_resizing_every_cell_is_blank_and_changed` | Despois de `resize`, todas as celas están en branco e cambiadas, e `clear_changes` esquece os cambios. |
| `changes_add_up_until_they_are_cleared` | Dous cambios na mesma fila xúntanse nun intervalo (2–4 e 6–7 dan 2–7), e unha fila que non existe ignórase. |
| `the_text_of_a_row_shows_each_character_once` | `row_text` mostra os caracteres anchos unha vez, e as celas en branco como espazos, sen os do final. |
| `flags_combine` | Os estilos combínanse con `\|`, e `contains` comproba todos os que se piden. |

## `crates/meiga-core/src/term/selection.rs`

Desde a subfase 1.8: a selección, o texto que se marca co rato, que mostra a pantalla e que copia o portapapeis ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). É outro submódulo de `term`, cos métodos de `Terminal` que a crean, a moven, a quitan e a len: os tipos da selección de `alacritty_terminal` quedan dentro da fachada, coma os demais ([ADR 0012](../adr/0012-fachada-do-terminal.md)). Quen a move é o rato da vista ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)), e quen leva o seu texto ao portapapeis, o módulo que copia e pega ([Código › meiga › `clipboard.rs`](meiga.md#cratesmeigasrcclipboardrs)).

**A selección vive na grella.** `alacritty_terminal` garda a selección no seu `Term` (`term.selection`), cos puntos da súa grella, que contan tamén o historial. Así segue o texto: cando a pantalla se despraza, sobe con el; e desaparece cando o programa borra a parte da pantalla onde está, cambia á pantalla alternativa ou reinicia o terminal. Escribir enriba das súas celas non a quita. Desde a subfase 1.9, desaparece tamén cando cambia o tamaño ([`resize`](#o-tamaño-resize)); e, como vive na grella, colle tamén filas do historial, e segue o seu texto cando se move a vista. Gardala fóra da grella obrigaría a facer todo isto outra vez ([ADR 0015 › Alternativas consideradas](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md#alternativas-consideradas)).

### `WORD_DELIMITERS`, `SelectionKind` e `Side`

```rust
pub(super) const WORD_DELIMITERS: &str = " \t\"'`()[]{}<>,;|│";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionKind {
    Cells,
    Words,
    Lines,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
```

- **`WORD_DELIMITERS`**: os caracteres que rematan unha palabra nun dobre clic: o espazo e o tabulador, as comiñas (`"`, `'` e `` ` ``), as parénteses, os corchetes, as chaves e os ángulos, e `,`, `;`, `|` e `│`, a liña vertical das táboas debuxadas con liñas de caixa. Unha ruta, un URL ou unha posición nun arquivo, coma `C:\x\f.rs:12:5`, son unha soa palabra. Os de `alacritty_terminal` (`SEMANTIC_ESCAPE_CHARS`) teñen tamén `:`, que partiría esa posición, e non teñen `;`. É `pub(super)`: `Terminal::new` dállos a `alacritty_terminal` na súa configuración.
- **`SelectionKind`**: como medra unha selección desde onde empeza, segundo os clics que a empezan:

  | Variante | Clics | Que selecciona |
  |---|---|---|
  | `Cells` | Un | Desde un lado dunha cela ata un lado doutra. |
  | `Words` | Dous | Palabras enteiras, que rematan nos caracteres de `WORD_DELIMITERS`. Nunha paréntese, un corchete, unha chave ou un ángulo, ata o que o pecha ou o abre; nos outros caracteres de `WORD_DELIMITERS`, ou nun destes sen parella, só a súa cela. |
  | `Lines` | Tres | Liñas enteiras, con todas as filas dunha liña que non colleu nunha. |

- **`Side`**: unha metade dunha cela. Unha selección de celas empeza e remata nun lado dunha cela: unha cela entra nela cando se pasa da súa metade. Dáo [`cell_at`](#a-cela-debaixo-do-punteiro-cell_at), de `mouse.rs`.
- `alacritty_terminal` ten os seus propios tipos para isto, `SelectionType` e o seu `Side`, que é o seu `Direction`. meiga ten os seus para que eses non saian da fachada, e o `From<Side> for Direction` do final do arquivo converte o lado.

### Crear, mover e quitar a selección

```rust
impl Terminal {
    pub fn select(&mut self, kind: SelectionKind, point: GridPoint, side: Side) {
        let point = self.grid_point(point);
        let selection = match kind {
            SelectionKind::Words => self.word_at(point),
            SelectionKind::Cells => Selection::new(SelectionType::Simple, point, side.into()),
            SelectionKind::Lines => Selection::new(SelectionType::Lines, point, side.into()),
        };
        self.term.selection = Some(selection);
    }

    pub fn select_to(&mut self, point: GridPoint, side: Side) {
        let point = self.grid_point(point);
        if let Some(selection) = &mut self.term.selection {
            selection.update(point, side.into());
        }
    }

    pub fn clear_selection(&mut self) -> bool {
        let had = self.has_selection();
        self.term.selection = None;
        had
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }
    …
}
```

| Método | Que fai |
|---|---|
| `select(kind, point, side)` | Empeza unha selección de `kind` no lado `side` da cela `point` da pantalla, en vez da que había. Unha de celas está baleira ata que remata noutro lado; unha de palabras ou de liñas xa ten a palabra ou a liña de `point`. |
| `select_to(point, side)` | Move o final da selección ao lado `side` da cela `point`: a selección vai desde onde empezou ata alí, tamén cara atrás, con palabras ou liñas enteiras se é diso. Sen selección, non fai nada. |
| `clear_selection` | Quita a selección, e di se había unha con celas: así quen chama sabe se ten que debuxar outra vez. |
| `has_selection` | Se hai unha selección con celas: un clic que non se moveu non ten ningunha. |

- **As clases de `alacritty_terminal`** (`SelectionType`): `Simple` para as celas, `Semantic` para as palabras e `Lines` para as liñas. A selección rectangular, `Block`, aínda non se usa.
- `alacritty_terminal` garda a selección coma dous puntos da grella, o do comezo e o do final, cada un co seu lado, e calcula as súas celas cada vez que se piden. Unha selección de celas que empeza e remata no mesmo lado da mesma cela, ou que vai do lado dereito dunha ao esquerdo da seguinte, non ten ningunha.
- `side.into()` converte o `Side` de meiga no de `alacritty_terminal`. `if let Some(selection) = &mut self.term.selection` toma prestada a selección para cambiala no seu sitio.
- O rato da vista chama a `select` cando baixa o botón esquerdo, coa clase dos clics que contou ([`Clicks`](#os-clics-seguidos-clicksettings-e-clicks)), e a `select_to` cando o rato pasa a outra cela ou a outro lado e, desde a subfase 1.9, cando a vista se move mentres a selección medra, coa roda ou soa, para que remate debaixo do punteiro.

```rust
pub fn selection_text(&self) -> Option<String> {
    let mut text = self.term.selection_to_string()?;
    // alacritty_terminal ends a selection of lines with LF; pasted
    // into a shell, it would run the line.
    if text.ends_with('\n') {
        text.pop();
    }
    Some(text).filter(|text| !text.is_empty())
}

pub(super) fn selection_range(&self) -> Option<SelectionRange> {
    self.term
        .selection
        .as_ref()
        .and_then(|selection| selection.to_range(&self.term))
}
```

- **`selection_text`**: o texto da selección, de `selection_to_string` de `alacritty_terminal`: as súas filas rematan con LF, salvo a última e as dunha liña que non colleu nunha fila, e sen os brancos do final; un carácter ancho vai unha vez. `alacritty_terminal` remata con LF a selección de liñas: pegada nunha shell, executaría a liña, así que se quita. Unha selección sen texto, coma unha cela baleira despois do final da liña, dá `None`. O portapapeis recíbeo con CR LF, que pon `write_text` ([Código › meiga-win › `clipboard.rs`](meiga-win.md#cratesmeiga-winsrcclipboardrs)).
- O `?` despois de `selection_to_string()` devolve `None` se non hai selección, ou se non ten celas; e `filter` converte un texto baleiro en `None`.
- **`selection_range`**, só para a fachada (`pub(super)`): as celas da selección na grella, se ten algunha. `to_range` calcúlaas cada vez: estende as palabras e as liñas, e axústaas á grella. É un `SelectionRange` de `alacritty_terminal`, coa primeira e a última cela, incluídas. Úsano `has_selection` e [`copy_damage`](#copy_damage-copiar-o-que-cambiou).

### O dobre clic: `word_at`

```rust
fn word_at(&self, point: Point) -> Selection {
    // As alacritty_terminal does, but only once: it would search for
    // the other bracket at every frame, maybe through all the history.
    if let Some(other) = self.term.bracket_search(point) {
        return cells(point.min(other), point.max(other));
    }
    // alacritty_terminal would also take the words on both sides.
    let cell = &self.term.grid()[point];
    if !cell
        .flags
        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
        && WORD_DELIMITERS.contains(cell.c)
    {
        return cells(point, point);
    }
    Selection::new(SelectionType::Semantic, point, Direction::Left)
}
```

**`word_at`** dá a selección dun dobre clic en `point`:

1. **Nunha paréntese, un corchete, unha chave ou un ángulo**, `bracket_search` busca o que o pecha ou o abre, saltando as parellas que hai dentro, e a selección vai dun ao outro, os dous incluídos. `alacritty_terminal` fai o mesmo nas súas seleccións de palabras, pero buscándoo outra vez cada vez que calcula as celas, en cada fotograma, quizais por todo o historial: meiga búscao unha vez, e garda unha selección de celas.
2. **Nun dos caracteres de `WORD_DELIMITERS`**, coma un espazo ou unha coma, ou nunha paréntese sen parella, só a súa cela: `alacritty_terminal` collería tamén as palabras dos dous lados. A metade dereita dun carácter ancho (`WIDE_CHAR_SPACER`) e a cela baleira que deixa ao final dunha fila un carácter ancho que non colleu (`LEADING_WIDE_CHAR_SPACER`) gardan un espazo, pero non son espazos: nelas, o dobre clic selecciona a palabra.
3. **No resto**, unha selección de palabras de `alacritty_terminal` (`Semantic`), que vai cara aos dous lados ata os caracteres de `WORD_DELIMITERS`, dentro da mesma liña aínda que ocupe varias filas, e que, ao arrastrar, segue collendo palabras enteiras.

```rust
fn cells(start: Point, end: Point) -> Selection {
    let mut selection = Selection::new(SelectionType::Simple, start, Direction::Left);
    selection.update(end, Direction::Right);
    selection
}
```

- **`cells`** fai unha selección de celas desde o lado esquerdo de `start` ata o dereito de `end`: as dúas incluídas.
- `point.min(other)` e `point.max(other)`: os puntos de `alacritty_terminal` compáranse pola liña e despois pola columna, así que o primeiro é o que vai antes no texto.

### As celas da pantalla e as da grella

```rust
fn grid_point(&self, point: GridPoint) -> Point {
    let row = point.row.min(self.size.rows - 1);
    let column = point.column.min(self.size.columns - 1);
    term::viewport_to_point(
        self.term.grid().display_offset(),
        Point::new(usize::from(row), Column(usize::from(column))),
    )
}
```

**`grid_point`** converte a cela da pantalla no punto da grella de `alacritty_terminal`, coma `copy_row`: a fila que se ve, contando o historial que hai por riba (`display_offset`, que desde a subfase 1.9 non é 0 mentres a vista mostra o historial). Unha cela fóra da pantalla pasa a ser a máis próxima: a resta nunca baixa de 0, porque a grella ten polo menos `MIN_SIZE`.

```rust
pub(super) fn selected_columns(range: &SelectionRange, line: Line, columns: u16) -> Columns {
    if line < range.start.line || line > range.end.line {
        return Columns::default();
    }
    // The columns of the grid are the ones of the screen, so they fit in a
    // u16.
    let column = |column: Column| column.0.min(usize::from(columns)) as u16;
    let start = if line == range.start.line {
        column(range.start.column)
    } else {
        0
    };
    let end = if line == range.end.line {
        column(range.end.column).saturating_add(1)
    } else {
        columns
    };
    Columns::new(start, end.min(columns))
}

pub(super) fn shown_rows(
    range: &SelectionRange,
    display_offset: usize,
    rows: u16,
) -> std::ops::Range<u16> {
    let offset = i32::try_from(display_offset).unwrap_or(i32::MAX);
    let row = |line: i32| line.saturating_add(offset).clamp(0, i32::from(rows)) as u16;
    row(range.start.line.0)..row(range.end.line.0.saturating_add(1))
}
```

- **`selected_columns`**: as columnas que a selección colle na fila da grella `line`, dunha fila de `columns` columnas: ningunha fóra das súas filas; desde a súa primeira columna na primeira fila, e desde 0 nas outras; e ata a súa última columna, incluída, na última fila, e ata o final nas outras. Úsaa `copy_row`, a través de [`selected_cells`](#os-axudantes), que engade as outras metades dos caracteres anchos.
- **`shown_rows`**: as filas da pantalla, de `rows`, nas que está a selección, cando a pantalla mostra a grella desde `display_offset` liñas por riba do seu final. As liñas do historial son negativas, e a fila da pantalla é a liña máis `display_offset`; as que quedan fóra da pantalla córtanse (`clamp`). Úsaa `copy_damage`, para copiar outra vez as filas da selección vella e da nova.
- Os `as u16` despois de `min` e de `clamp` non cortan nada: o valor xa cabe. `saturating_add` e `unwrap_or(i32::MAX)` evitan desbordar cos números máis grandes.

As probas da selección están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs), coas do terminal.

## `crates/meiga-core/src/term/scanner.rs`

O escáner que le a saída do programa antes do parser, para atopar as secuencias que `alacritty_terminal` ignora e meiga necesita ([ADR 0007](../adr/0007-integracion-coa-shell.md) e [ADR 0012](../adr/0012-fachada-do-terminal.md)). Só le os bytes: o parser recibe exactamente os mesmos.

Polo de agora segue unha: o **win32-input-mode** (`ESC [ ? 9001 h` e `l`), que o ConPTY activa ao arrancar para que meiga lle envíe eventos de tecla completos ([`input.rs`](#cratesmeiga-coresrcinputrs)). O plan falaba tamén de `?1004`, os eventos de foco, pero ese xa o segue `alacritty_terminal`. As secuencias da integración coa shell (OSC 7, 9;9 e 133) chegarán na subfase 4.1.

**Ler como `vte`.** Para ver un cambio de modo só cando `vte` o vería, tamén cando a secuencia chega partida entre dúas lecturas, o escáner segue os mesmos estados ca el ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control)):

| Estado | Onde está |
|---|---|
| `Ground` | No texto, ou nunha secuencia que non importa: ata o seguinte ESC. |
| `Escape` | Despois de ESC. |
| `CsiEntry` | Despois de `ESC [`. |
| `CsiParam` | Lendo os números dunha secuencia privada, despois de `ESC [ ?`. |
| `CsiIntermediate` | Despois doutro byte intermedio dunha secuencia privada, como `$`. |
| `CsiIgnore` | Unha secuencia privada que `vte` ignora, ata o seu byte final. |

```rust
pub fn scan(&mut self, bytes: &[u8]) {
    let mut rest = bytes;
    loop {
        if self.state == State::Ground {
            let mut escapes = memchr::memchr_iter(ESC, rest);
            let at = loop {
                let Some(at) = escapes.next() else {
                    return;
                };
                if let Some(&[b'[', next]) = rest.get(at + 1..at + 3)
                    && Self::not_private(next)
                {
                    continue;
                }
                break at;
            };
            self.state = State::Escape;
            rest = rest.get(at + 1..).unwrap_or_default();
            continue;
        }
        let Some((&byte, after)) = rest.split_first() else {
            return;
        };
        rest = after;
        self.advance(byte);
    }
}
```

- **O camiño rápido.** No estado `Ground` nada importa ata o seguinte ESC, e `memchr_iter` atópao lendo moitos bytes á vez, con instrucións SIMD. Case todas as secuencias da saída son cores e movementos do cursor: `ESC [` e un díxito ou unha letra. Esas sáltaas sen saír do bucle (`continue`), porque non poden cambiar un modo privado. Só segue byte a byte (`advance`) as que empezan por `ESC [ ?`, e as que chegan partidas antes do byte despois do `[`.
- **Por que dá o mesmo.** `vte` só le un `?` como a marca dun modo privado xusto despois de `ESC [`; os caracteres de control, DEL e os bytes desde 0x80 non contan, e decide o seguinte (`not_private`). Calquera outro byte nesa posición significa que a secuencia non pode cambiar un modo privado, e os bytes que lle quedan ata o seu final non teñen ningún ESC. Lelos como texto, ata o seguinte ESC, deixa o escáner no mesmo estado ca seguir a secuencia.
- `if let Some(&[b'[', next]) = rest.get(at + 1..at + 3) && Self::not_private(next)` encadea un `if let` cunha condición (*let chains*, da edición 2024). `rest.get(at + 1..at + 3)` dá os dous bytes despois do ESC, se os hai, e o patrón `&[b'[', next]` comproba o primeiro e garda o segundo.

**Byte a byte** (`advance`), como `vte`:

- En calquera estado, ESC empeza unha secuencia nova, e CAN (0x18) e SUB (0x1a) cancelan a actual.
- `advance_escape`: `[` pasa a `CsiEntry`, e `c` é un reinicio completo (RIS), que desactiva todos os modos. Os caracteres de control, DEL e os bytes desde 0x80 non rematan o escape: `vte` ignóraos, e o seguinte byte escolle a secuencia. Calquera outro byte volta a `Ground`: é outra secuencia de escape, ou o comezo dunha cadea (OSC, DCS…) que só remata cun ESC ou un carácter de control. Que os bytes desde 0x80 tamén se ignoran atopouno a revisión do código: antes, `ESC`, un byte desde 0x80 e `[ ? 9001 h` non activaban o modo, e `vte` si.
- `advance_csi_entry`: se o byte non é `?`, volta a `Ground`, como no camiño rápido; con `?`, empeza unha secuencia privada (`start_private`).
- `advance_csi_param`: os díxitos forman o número (`digit`), `;` separa parámetros e `:` subparámetros. Un byte intermedio (0x20–0x2f, como `$`) pasa a `CsiIntermediate`, e outra marca despois dos números (`<`, `=`, `>` ou `?`), a `CsiIgnore`. Un byte final (0x40–0x7e) remata a secuencia (`dispatch`).
- `dispatch`: se a secuencia ten un parámetro 9001, o `?` é o seu único byte intermedio e `vte` non a ignora, `h` activa o modo e `l` desactívao.
- **Os límites de `vte`.** Unha secuencia con máis de 32 números (`MAX_VALUES`, contando os subparámetros) ou con máis de 2 bytes intermedios (`MAX_INTERMEDIATES`, contando o `?`) ignórase, e o escáner tamén a ignora (`ignoring`). Só o primeiro número de cada parámetro é un modo: `?9001:1h` activa o modo, e `?1:9001h` non.
- `digit` usa `saturating_mul` e `saturating_add`: un número enorme queda no máximo dun `u16`, en vez de desbordarse.

**Probas.** Dez:

| Proba | Que comproba |
|---|---|
| `it_follows_the_mode_on_and_off` | `?9001h` activa o modo e `?9001l` desactívao; un texto sen secuencias non o cambia. |
| `it_finds_the_mode_among_others` | O modo atópase entre outros parámetros (`?1004;9001h`). |
| `it_follows_the_mode_when_the_sequence_is_split` | O que escribe o ConPTY ao arrancar, partido en dous en todas as posicións posíbeis, e byte a byte. |
| `only_a_private_set_or_reset_changes_the_mode` | Sen `?`, con outra marca, con outro byte intermedio, con dous `?` ou con outro byte final, nada. |
| `only_the_first_number_of_a_parameter_is_a_mode` | `?9001:1h` si; `?1:9001h` non. |
| `control_characters_cancel_or_restart_the_sequence` | CAN cancela a secuencia; ESC empeza outra; un salto de liña execútase e a secuencia segue; e unha cadea OSC remata no ESC seguinte. |
| `a_full_reset_turns_the_mode_off` | `ESC c` desactiva o modo, e `ESC ( c`, que escolle un xogo de caracteres, non. |
| `bytes_over_0x7e_do_not_end_an_escape` | Despois dun ESC, DEL e os bytes desde 0x80 ignóranse, tamén partidos entre dúas lecturas: `ESC`, 0x80 e `[ ? 9001 h` activan o modo, e `ESC`, `á` e `c` son un reinicio. |
| `a_sequence_with_too_many_numbers_is_ignored` | Con 32 números, o modo cambia; con 33, `vte` ignora a secuencia, e o escáner tamén. |
| `it_agrees_with_vte_on_random_output` | **A referencia.** 2 000 saídas aleatorias, de 1 a 100 anacos: bytes soltos dos que importan nas secuencias (ESC, `[`, `?`, `;`, `:`, díxitos, `h`, `l`, `c`, `p`, `$` e `>`), caracteres de control, DEL e bytes desde 0x80; e, un de cada catro, unha secuencia enteira ou o seu comezo (`ESC [ ? 9001 h`, `ESC [ ? 9001 l`, `ESC c`, `ESC [ ?`, unha cadea OSC…), para que o modo cambie a miúdo. A metade empezan co modo activo, para comprobar tamén que se desactiva cando toca. Cada saída lese en anacos aleatorios co escáner e co parser de `vte`, cun `Handler` que segue o modo como o faría `alacritty_terminal` se non o ignorase, e despois de cada anaco os dous teñen que coincidir. Non hai os díxitos 2 nin 6, para que non empece ningunha actualización sincronizada (`?2026`), que `vte` gardaría sen chamar ao `Handler`. Como nas outras probas aleatorias, a secuencia é fixa: un fallo pódese repetir. |

## `crates/meiga-core/src/term/recording.rs`

Gravacións do que escribe un programa no terminal, para reproducilo nas probas e nas medicións sen executar o programa outra vez ([ADR 0012](../adr/0012-fachada-do-terminal.md)). Cada bloque gárdase tal como chegou da pseudoconsola, co momento en que chegou: así pódese reproducir cos mesmos anacos e as mesmas pausas.

O formato, cos números en *little-endian*:

| Bytes | Contido |
|---|---|
| 8 | `MEIGAREC` (`MAGIC`). |
| 2 | A versión do formato: 1 (`VERSION`). |
| 2 + 2 | As columnas e as filas da pseudoconsola. |

E despois, para cada bloque:

| Bytes | Contido |
|---|---|
| 8 | Os microsegundos desde que empezou a gravación. |
| 4 | A lonxitude do bloque. |
| Esa lonxitude | Os bytes do bloque. |

- **`Recorder<W: Write>`** escribe unha gravación en calquera saída que implemente `Write`: un arquivo, con `create`, ou un `Vec<u8>`, nas probas.
  - `new` escribe a cabeceira e empeza a contar o tempo (`Instant::now()`).
  - `record` engade un bloque, co tempo desde o comezo. Un bloque de 4 GiB ou máis non cabe na súa lonxitude (`u32`) e dá un erro.
  - `blocks` e `bytes` contan os bloques e os bytes gravados.
  - `finish` escribe o que quede na memoria e devolve a saída. `create` usa un `BufWriter`, que xunta as escrituras pequenas nunha grande. Destruílo tamén escribe o que quede, pero sen poder informar dun erro.
- **`Recording`** le unha gravación enteira e compróbaa:
  - `read` le un arquivo, e `from_bytes` comproba os datos: o tamaño da cabeceira, a marca `MEIGAREC`, a versión, e que ningún bloque estea cortado. Os erros son `io::Error` do tipo `InvalidData`, cunha mensaxe que di que falla, e en que byte.
  - Garda os datos tal como están e, de cada bloque, só o tempo e onde empeza e remata (`Span`): ler os bloques non os copia.
  - `blocks` dá os bloques en orde (`Block`: o tempo e os bytes, prestados dos datos); `size`, o tamaño da pseudoconsola; `output_len`, os bytes de saída sen as cabeceiras; e `duration`, cando chegou o último bloque.
  - `first_chunk` dá os primeiros bytes como un *array* de tamaño fixo, se os hai, e o patrón `[magic @ .., v0, v1, c0, c1, r0, r1]` sepáraos sen índices: os 8 primeiros en `magic`, e os seguintes un a un.

**Probas.** Catro:

| Proba | Que comproba |
|---|---|
| `a_recording_gives_back_its_blocks_and_size` | Tres bloques, un deles baleiro, gravados nun `Vec` e lidos outra vez: os mesmos bytes, o tamaño, a lonxitude e os tempos en orde. |
| `a_recording_without_blocks_is_valid` | Unha gravación só coa cabeceira é válida, e dura 0. |
| `other_data_is_rejected` | Datos máis curtos ca a cabeceira, sen a marca `MEIGAREC` ou doutra versión dan cadanseu erro. |
| `a_cut_block_is_rejected` | Un bloque cortado, na súa cabeceira ou nos seus bytes, dá `the block at byte 14 is cut`. |

## `crates/meiga-core/src/term/tests.rs`

As trinta e seis probas do terminal, nun arquivo aparte para que `mod.rs` sexa máis curto. É un submódulo de `term` (`#[cfg(test)] mod tests;`, ao final de `mod.rs`), así que ve tamén os seus tipos privados. Oito axudantes: `copy` copia o terminal enteiro nunha `Screen` nova; `responses` recolle as respostas; `narrow` crea unha cela cun carácter, sen cores nin estilos; desde a subfase 1.8, `marks` mostra as celas seleccionadas dunha fila, `#` as que levan `CellFlags::SELECTED` e `.` as outras; desde a 1.9, `texts` dá o texto de cada fila que se ve, `cursor_at`, a fila e a columna do cursor, e `ten_lines`, un terminal de 10 × 3 coas liñas do 1 ao 10, 7 delas no historial; e, desde a revisión do código da 1.9, `assert_the_rows_go_into_the_history`, que, coa vista dúas filas arriba, comproba que dúas liñas novas ao final da pantalla meten dúas filas no historial e que a vista segue mostrando as mesmas. Desde a 1.8, cinco probas comproban a selección, e a proba aleatoria cámbiaa tamén. Desde a 1.9, trece comproban o cambio de tamaño, a pantalla principal á parte e a vista do historial, e a proba aleatoria move tamén a vista.

| Proba | Que comproba |
|---|---|
| `the_output_reaches_the_screen` | O texto e o salto de liña chegan ás celas, e o cursor queda despois do texto: unha barra que pestanexa, porque o programa non escolleu outro cursor. A copia leva as cores de meiga (`Palette::MEIGA`). |
| `only_what_changed_is_copied_again` | Sen saída, unha copia non marca nada; despois de escribir noutra fila, só cambian o texto novo e as celas do cursor, onde estaba e onde está. |
| `the_program_gets_its_answers` | As respostas a DA1, DSR 5, DSR 6, `ESC [ 18 t` e DA2, que leva a versión de `alacritty_terminal` (`2600`); e a `ESC [ 14 t`, ningunha mentres o terminal non sabe o tamaño da cela e, cunha cela de 9 × 20 píxeles nunha grella de 10 × 3, `ESC [ 4 ; 60 ; 90 t`. |
| `questions_about_colours_are_answered_with_the_palette` | As preguntas pola cor 1 da paleta (OSC 4) e polas do texto, o fondo e o cursor (10, 11 e 12) respóndense coas cores de meiga, cada unha co final da súa pregunta, BEL ou `ESC \`; o cursor ten a cor do texto. Despois de cambiar o programa o fondo, a resposta ten a cor nova, e vai entre as dúas respostas a DA1, na orde das preguntas. E cun tema que cambia a cor 1, a resposta é a do tema. |
| `a_change_of_the_palette_changes_every_cell` | Cambiar a cor 1 da paleta (OSC 4) cambia todas as celas, aínda que seguen co mesmo número de cor; voltar á do tema (OSC 104), tamén; cambiar a do cursor (OSC 12) só cambia as dúas celas do cursor; e despois, unha copia sen cambios non marca nada. |
| `the_title_and_the_bell` | O título de OSC 0 e 2, sen os espazos dos extremos; gardalo e recuperalo (`ESC [ 22 t` e `ESC [ 23 t`); un título baleiro é ningún; e a campá. |
| `osc_52_only_copies` | O texto en base64 de OSC 52 chega ao portapapeis; ler o portapapeis, a selección primaria e un texto que non é base64 non fan nada nin responden. |
| `the_modes_follow_the_program` | Os modos por defecto; activar e desactivar cada un; os modos do rato, e os seus formatos, exclúense entre si; e un reinicio (RIS) déixaos como ao principio. |
| `colours_and_styles_reach_the_cells` | As cores de 16, de 256 e *truecolor*, do texto e do fondo, e todos os estilos, tamén os cinco subliñados. |
| `a_wide_character_takes_two_cells` | Un carácter ancho ocupa as súas dúas metades; se non cabe ao final da fila, vai á seguinte. |
| `a_tab_leaves_blank_cells` | O tabulador deixa celas en branco. |
| `the_cursor_follows_the_program` | Ao principio, unha barra que pestanexa; as formas do cursor (DECSCUSR), fixas e que pestanexan; o pestanexo (`?12h` e `?12l`); voltar ao cursor de meiga (`ESC [ 0 SP q`); ocultalo e mostralo; e sobre a metade dereita dun carácter ancho, debúxase desde a esquerda. |
| `the_cursor_on_a_lone_right_half_stays_in_its_cell` | Despois de borrar a metade esquerda dun carácter ancho (`ESC [ P`), a metade dereita que queda é unha cela en branco, e o cursor debúxase nela, tanto na primeira columna coma despois dun carácter estreito. Antes da revisión do código, na primeira columna era un *panic*. |
| `the_cells_of_the_cursor_are_changes_when_it_is_drawn` | As celas do cursor márcanse só se cambia e se se debuxa: ao ocultalo, só onde estaba; mentres está oculto, nada; e na última columna, unha soa cela. |
| `a_synchronized_update_waits_for_its_end` | A saída dunha actualización sincronizada non chega á pantalla ata que remata ou vence (como moito, 150 ms); `end_sync` móstraa e entrega as súas respostas. |
| `after_a_resize_everything_is_copied_again` | Despois de cambiar o tamaño, a copia ten o tamaño novo e todas as celas cambiadas, co texto no seu sitio. |
| `the_grid_is_never_smaller_than_two_columns_and_one_row` | Un terminal de 0 × 0 ou de 1 × 0 mide 2 × 1, e escribir caracteres anchos e inserir celas e liñas nel non falla. |
| `a_taller_screen_adds_blank_rows_and_leaves_the_history_alone` | Desde a subfase 1.9: un terminal de 10 × 3 con `a` a `e`, dúas no historial, pasa a ter 5 filas: `c`, `d` e `e` seguen arriba, con dúas filas baleiras abaixo, o cursor segue despois do `e`, e o historial segue con dúas filas. `alacritty_terminal` traería del `a` e `b`. |
| `a_shorter_screen_keeps_the_cursor_and_the_rows_above_it` | De 5 filas a 3, se o texto cabe, só se van as filas baleiras de abaixo: `a` e `b` seguen arriba, co cursor, e nada vai ao historial. Se non cabe, de `a`, `b` e `c` a 2 filas, a primeira vai ao historial; e alí segue cando a pantalla volta a 5 filas, con `b` e `c` arriba e as baleiras abaixo. |
| `narrower_lines_wrap_downwards_and_join_again_when_wider` | `abcdef` e `xy`, de 8 columnas a 4: `abcdef` pártese en dúas filas, que baixan cara á fila baleira, e nada vai ao historial; o cursor segue despois de `xy`. De volta a 8, as dúas filas xúntanse, a fila baleira volta abaixo, e un `z` escríbese despois de `xy`. |
| `the_cursor_keeps_its_row_when_many_rows_below_it_wrap` | En 4 × 4, con `x`, `a`, `bbbb` e `cccc` e o cursor no `a`: a 2 columnas, as dúas últimas liñas ocupan catro filas e, coma no ConPTY, a pantalla empeza na fila do cursor, e a última, que xa non cabe, vaise. O cursor segue no `a`, e un `A` escríbese enriba del. |
| `the_cursor_keeps_its_place_after_the_last_character_of_a_full_row` | Co cursor esperando despois de `abcd`, nunha fila chea de 4 columnas: a 6 columnas, un `e` vai despois do `d`, na mesma fila; a 2 columnas, `abcd` ocupa dúas filas, e o `e` vai á seguinte. |
| `on_the_alternate_screen_the_primary_one_gets_the_last_size_when_the_program_leaves` | Na pantalla alternativa, a 10 × 5 e a 4 × 2, `ALT` segue arriba, sen repartir. Se o programa volta á principal co tamaño que tiña, 10 × 3, esta non cambia, sen rastro dos tamaños do medio: `c`, `d` e `e`, o cursor despois do `e` e dúas filas no historial. E se volta despois de pasar a 10 × 5, recibe ese tamaño coma na pantalla principal: dúas filas baleiras abaixo, e o historial onde estaba. |
| `the_output_after_leaving_the_alternate_screen_reaches_the_primary_one` | Despois de pasar a 10 × 4 na pantalla alternativa, a `X` que segue a `?1049l` chega á principal, xa co tamaño novo, despois do `b`: na mesma lectura, dentro dunha actualización sincronizada (`?2026`) e nunha que remata `end_sync`, á súa hora. |
| `a_reset_on_the_alternate_screen_leaves_an_empty_primary_screen_with_history` | Un reinicio (`ESC c`) na pantalla alternativa, despois dun cambio de tamaño, volta á principal, baleira e sen historial; e esta garda historial: con seis liñas en catro filas, dúas van a el. |
| `the_view_moves_through_the_history` | Con `ten_lines`, a vista móvese dúas filas cara arriba, unha pantalla, ao principio do historial, unha pantalla cara abaixo, a unha fila por riba da pantalla e, con `scroll_to(100)`, ao principio outra vez, e volta á pantalla; en cada paso vense as filas que tocan. Mentres non se ve a fila do cursor, o cursor non se debuxa (`Hidden`); e nos extremos, `scroll` di que non se moveu. |
| `the_output_does_not_move_the_view_of_the_history` | Coa vista dúas filas arriba, en `6`, `7` e `8`, dúas liñas novas non a moven: segue mostrando as mesmas filas, agora 4 por riba da pantalla, nun historial de 9. |
| `the_rows_that_leave_a_region_below_the_top_do_not_move_the_view` | Desde a revisión do código da subfase 1.9. Con `ten_lines` e a vista dúas filas arriba, en `6`, `7` e `8`, seis saídas non meten nada no historial nin moven a vista: dez liñas novas ao final dunha rexión que empeza na segunda fila (`ESC [ 2 ; 3 r`); tres `CSI S`; caracteres que pasan á fila seguinte ao final da rexión; un tabulador despois da última columna; unha rexión que non vale (`ESC [ 1 ; 1 r`), que deixa a de antes; e, sen rexión, dous `CSI M` co cursor na segunda fila. Desde o principio do historial, a vista tampouco pasa del. E unha rexión que empeza arriba (`ESC [ 1 ; 2 r`) mete as súas filas no historial, e a vista sobe con elas. |
| `a_new_size_a_reset_or_deccolm_make_the_region_the_whole_screen_again` | Desde a revisión do código. Despois dunha rexión que empeza na segunda fila, a pantalla enteira volta ser a rexión, e dúas liñas novas ao final da pantalla meten dúas filas no historial, coa vista nas mesmas filas: despois dun cambio de tamaño; dunha rexión da pantalla alternativa, cando o programa volta á principal, que recibe outro tamaño; dun reinicio (`ESC c`); e de DECCOLM, `?3h` e `?3l`, que `alacritty_terminal` só segue para borrar a pantalla e a rexión. |
| `a_resize_keeps_the_view_inside_the_history` | Coa vista no principio do historial, despois de pasar a 10 × 8, a 2 × 2 e a 10 × 3, a vista nunca pasa do historial, e copiar o terminal non falla. E coa vista abaixo, un cambio de tamaño déixaa abaixo. |
| `a_selection_of_cells_marks_them_and_gives_their_text` | Un clic que non se move non selecciona nada, e a copia non cambia nada. Arrastrando desde o lado esquerdo da segunda cela ata o dereito da sexta, o texto é `la mu`, e só esas celas quedan marcadas e cambiadas; ata o lado esquerdo da terceira cela da fila seguinte, `la mundo`, un LF e `ad`, e só cambian as celas que entran; e cara atrás, ata o lado esquerdo da primeira, `o`. `clear_selection` di se había selección, e despois só cambia a cela que estaba seleccionada, co texto no seu sitio. |
| `a_double_click_selects_a_word_a_path_or_up_to_the_other_bracket` | Un dobre clic selecciona unha palabra (`cat`, e `ok` e `si`, dentro dunhas parénteses e dunhas comiñas); unha ruta enteira, coas súas barras invertidas e os seus `:` (`C:\x\f.rs:12:5`); nunha paréntese, ata a outra, desde as dúas; nun espazo ou nunha coma, só iso; despois do final do texto, ningún texto; e en calquera das metades dun carácter ancho, a palabra (`中文`). |
| `a_triple_click_selects_the_rows_of_a_line` | Un triplo clic nunha liña que non colle nunha fila marca as súas dúas filas, e o seu texto vai sen LF ao final; arrastrar ata a liña seguinte engádea, despois dun LF. |
| `a_wide_character_is_selected_whole` | Unha selección que empeza na metade dereita dun carácter ancho, ou que remata na esquerda, marca o carácter enteiro, e o texto lévao unha vez. |
| `the_selection_moves_with_the_text_and_goes_away_with_it` | Cando a pantalla se despraza, a selección sobe co seu texto. Cando o programa borra unha das súas filas, desaparece, tamén da outra fila, que non cambiou. Escribir enriba dela non a quita, pero borrar a pantalla si. |
| `copying_only_the_damage_gives_the_same_screen_as_copying_everything` | **A invariante da copia.** 300 casos aleatorios, de ata 79 anacos dunha lista de 70: texto, caracteres anchos e combinados, movementos do cursor, borrados, inserir e borrar caracteres e liñas, desprazamentos e rexións, a pantalla alternativa, estilos, reinicios… Cada caso lese en anacos de ata 16 bytes, por veces cambiando o tamaño, con grellas de 2 × 1 a 80 × 24. Despois de cada anaco, a pantalla copiada só co dano ten que ser igual, cela a cela e no cursor, á dun segundo terminal copiado enteiro. Esta proba atopou o oco do dano de `alacritty_terminal` cos caracteres anchos. Desde a subfase 1.8, antes de cada anaco cambia tamén a selección, igual nos dous terminais: un de cada oito empeza unha, de calquera clase; dous de cada oito móvena; e un quítaa, ás veces con celas fóra da grella. Así comproba tamén que a copia marca as celas onde cambia a selección. Desde a 1.9, outro de cada oito move a vista do historial, igual nos dous terminais, con calquera `Scroll`: de dúas filas cara abaixo a dúas cara arriba, unha pantalla ou ata un extremo. |

## `crates/meiga-core/tests/recordings.rs`

Probas de integración: usan o crate desde fóra, como os outros crates, con gravacións reais do ConPTY que leva meiga, feitas co exemplo `conpty` de `meiga-win` e a súa opción `--record` ([Código › meiga-win](meiga-win.md#cratesmeiga-winexamplesconptyrs)). As gravacións están en `tests/data/` e entran no binario das probas con `include_bytes!`, así que as probas non len ningún arquivo:

| Gravación | Que é | Tamaño |
|---|---|---|
| `cmd-echo.mrec` | `cmd /c echo ola`. | 140 bytes |
| `spinner.mrec` | O escenario `tools\perf\scenarios\spinner.ps1`, durante 3 s. | 2 243 bytes |
| `ai-stream.mrec` | O escenario `tools\perf\scenarios\ai-stream.ps1`, durante 12 s: texto con cores, un emoji, `❯` e unha táboa con liñas de caixa. | 45 838 bytes |

- As tres son dunha pseudoconsola de 120 × 30, a do exemplo, e só levan a saída destes comandos, sen datos de ningunha máquina. Están marcadas como binarias no `.gitattributes` ([Código › workspace](workspace.md#gitattributes-e-gitignore)).
- `feed` dálle uns anacos a un terminal novo, e devolve o que mostra (`Shown`): o texto e as celas de cada fila, o cursor, o título, os modos e as respostas.

| Proba | Que comproba |
|---|---|
| `cmd_writes_ola` | A primeira fila é `ola`, e o resto están baleiras; o cursor, na seguinte. O win32-input-mode está activo mentres corre o programa, e ao final o ConPTY desactívao. A única resposta é a de DA1, que o ConPTY pide ao arrancar. |
| `the_spinner_only_changes_what_it_draws_again` | O primeiro fotograma do *spinner* cambia a liña enteira (16 celas); os seguintes, co cursor oculto, só o símbolo, e o número dos segundos cando cambia: 1 ou 13 celas. Ao final, o símbolo nunha cor do cubo da paleta, `Thinking` en negra e os segundos atenuados, e o cursor visíbel outra vez, na fila seguinte: unha barra que pestanexa, porque o programa non escolleu outro cursor. |
| `the_ai_answer_keeps_its_text_and_colours` | O emoji 💡 nas súas dúas metades, `❯` en cian e en negra (`Indexed(6)`), un título na cor 141 da paleta, a táboa con liñas de caixa e o cursor ao final. |
| `how_the_output_is_split_does_not_change_what_is_shown` | Cada gravación nun só anaco, byte a byte (cada secuencia e cada carácter de varios bytes partidos) e en 20 particións aleatorias mostra exactamente o mesmo. |
| `copying_only_what_changed_gives_the_same_screen` | A mesma invariante ca en `tests.rs`, coa saída real: despois de cada bloque, a copia só do dano é igual á copia enteira. |

## `crates/meiga-core/examples/replay.rs`

Un exemplo de consola que lle dá unha gravación a un terminal de meiga tan rápido como pode, e mostra a que velocidade a leu: o que fai o fío lector coa saída do programa, sen a pseudoconsola e sen debuxar.

```text
cargo run --release -p meiga-core --example replay -- <recording> [--rounds <n>] [--screen]
```

- Cada rolda dálle todos os bloques, en orde e tal como chegaron, a un terminal novo do tamaño da gravación, e mide o tempo, en MB de saída por segundo (1 MB = 1 000 000 bytes). Hai 5 roldas, salvo que `--rounds` diga outra cousa, e ao final mostra a mediana, a mínima e a máxima. A velocidade só di algo cunha gravación grande: as pequenas len en microsegundos.
- **As actualizacións sincronizadas** que o programa non remata móstranse cando vence o seu tempo, contado cos momentos da gravación, como fai meiga desde a subfase 1.6 co temporizador dos fotogramas: `sync_end` é o momento da gravación en que vence a que está pendente, e o primeiro bloque que chega despois remátaa con `end_sync`.
- As respostas recóllense despois de cada bloque, como fai o fío lector, que llas envía ao programa.
- **`--screen`** mostra ao final o que hai na pantalla: as filas con texto, numeradas desde 1 como nas secuencias de control; o cursor; o título; os modos activos; e as respostas, cos caracteres de control á vista (␛ é ESC, ␍ é CR…), como o exemplo `conpty`. Se queda unha actualización sincronizada pendente, móstraa antes.
- Os erros de uso (sen gravación, unha opción que non existe, `--rounds` sen un número maior ca 0 ou dúas gravacións) mostran o uso e rematan co código 2; unha gravación que non se pode ler, co 1.

Por exemplo, coa gravación de `cmd /c echo ola`:

```text
crates\meiga-core\tests\data\cmd-echo.mrec: 120 × 30, 6 blocks, 54 bytes of output in 0.1 s.
Round 1: 0.0001 s, … MB/s.
…
Median: … MB/s (from … to …).
Screen of 120 × 30, the rows with text:
   1 │ola
Cursor: row 2, column 1, Beam, blinking.
Title: (none)
Modes: alternate_scroll, show_cursor
Answers: ␛[?6c
```

## `crates/meiga-core/examples/tabs.rs`

Desde a subfase 2.1, un exemplo de consola para probar o modelo das tabs a man, sen xanela e sen programas nas tabs: le ordes, unha por liña, e despois de cada unha mostra a barra en texto.

```text
cargo run -p meiga-core --example tabs
```

```text
Commands: new, close TAB, move TAB POSITION, activate TAB, next, previous, help, quit.
TAB is the name of a tab, and POSITION counts the tabs from 1.
[a]
> new
a [b]
```

- Cada tab chámase cunha letra, `a`, `b`, `c`…, na orde na que se abriu, e despois de `z`, `aa`, `ab`… (`Names::fresh`). O contido de cada tab é o seu nome: a xanela é unha `Window<String>`.
- A barra mostra a tab activa entre corchetes (`bar`).
- As ordes van en inglés, coma as outras mensaxes de meiga:
  - `new` abre unha tab ao final e actívaa, coma fará o botón +;
  - `close TAB` pecha unha tab; se é a única, a xanela pecharíase, e o exemplo remata;
  - `move TAB POSITION` move unha tab a unha posición, contada desde 1, coma na barra;
  - `activate TAB`, `next` e `previous` activan unha tab;
  - `help` mostra as ordes, e `quit` remata.
- Un nome que non existe, unha posición que non o é e unha orde que non entende mostran unha mensaxe, e a xanela queda igual.
- `run` interpreta cada orde cun `match` sobre as palabras da liña (`["move", name, position]`…) e responde que mostrar (`Outcome`).
- Se as ordes chegan por unha tubaxe, e non desde o teclado, non mostra o indicador `> ` antes de ler, senón despois, xunto con cada orde, para que a saída se lea coma unha sesión. `IsTerminal` di se a entrada é unha consola. Así o usan os pasos de comprobación da subfase.

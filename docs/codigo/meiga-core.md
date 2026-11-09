# Código › `meiga-core`

> Estado: subfase 1.7. Contén as cores e a paleta, a xeometría (puntos, tamaños e rectángulos, e o tamaño da grella), as zonas que cambiaron (*damage*), o ritmo dos fotogramas, as celas do texto, a liña de comandos dun programa, as teclas, coma eventos e coma o que le o programa, e o terminal: a fachada sobre `alacritty_terminal`, coas súas respostas e a súa paleta, o escáner previo e as gravacións.

**Responsabilidade:** toda a lóxica que non depende de Windows: o modelo de xanelas, tabs e grupos; a fachada do terminal; a codificación do teclado e do rato; a busca difusa; e a configuración. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeiga-corecargotoml) | O crate e as súas dependencias: `alacritty_terminal`, `memchr` e `unicode-width`. |
| [`src/lib.rs`](#cratesmeiga-coresrclibrs) | A raíz do crate: os módulos. |
| [`src/color.rs`](#cratesmeiga-coresrccolorrs) | `Rgb`, o tipo das cores, e `Palette`, as cores dun terminal: as 16 do esquema *Campbell* e as 256 do xterm. |
| [`src/geometry.rs`](#cratesmeiga-coresrcgeometryrs) | `Point`, `Size` e `Rect`: posicións, tamaños e rectángulos en píxeles; e `GridSize`, o tamaño da grella en celas. |
| [`src/damage.rs`](#cratesmeiga-coresrcdamagers) | `Damage`: as zonas da xanela que cambiaron desde o último fotograma. |
| [`src/pacing.rs`](#cratesmeiga-coresrcpacingrs) | `FramePacer`: cando debuxar o seguinte fotograma. |
| [`src/text.rs`](#cratesmeiga-coresrctextrs) | `CellChar` e `cells`: o texto como celas dunha grella. |
| [`src/shell.rs`](#cratesmeiga-coresrcshellrs) | `command_line`: a liña de comandos dun programa cos seus argumentos, entre comiñas segundo as regras de Windows. |
| [`src/input.rs`](#cratesmeiga-coresrcinputrs) | `KeyEvent`, unha tecla que baixa ou sobe, e `KeyEncoder`, que a converte no que le o programa: en win32-input-mode ou en VT. Os eventos de foco. |
| [`src/term/mod.rs`](#cratesmeiga-coresrctermmodrs) | `Terminal`: a fachada sobre `alacritty_terminal`, cos seus eventos, modos e respostas, a paleta, e a copia do que cambia. |
| [`src/term/screen.rs`](#cratesmeiga-coresrctermscreenrs) | `Screen`: o que mostra o terminal, en tipos de meiga, coa súa paleta, e as celas que cambiaron. |
| [`src/term/scanner.rs`](#cratesmeiga-coresrctermscannerrs) | `Scanner`: o escáner que le a saída antes do parser, polo de agora para `?9001`. |
| [`src/term/recording.rs`](#cratesmeiga-coresrctermrecordingrs) | `Recorder` e `Recording`: gravar a saída dun programa e lela. |
| [`src/term/tests.rs`](#cratesmeiga-coresrctermtestsrs) | As probas do terminal. |
| [`tests/recordings.rs`](#cratesmeiga-coretestsrecordingsrs) e `tests/data/` | Probas con gravacións reais do ConPTY. |
| [`examples/replay.rs`](#cratesmeiga-coreexamplesreplayrs) | Un exemplo de consola que lle dá unha gravación ao terminal e mide a velocidade. |

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
pub mod pacing;
pub mod shell;
pub mod term;
pub mod text;
```

- `lib.rs` é a raíz dunha biblioteca: o compilador empeza por este arquivo, e desde el decláranse os demais módulos.
- As liñas que empezan por `//!` son **documentación do crate**. `cargo doc` convérteas en documentación HTML.
- `#![forbid(unsafe_code)]` é un **atributo interno** (`#!`), que se aplica a todo o crate. Prohíbe calquera bloque `unsafe` en `meiga-core`. Co nivel `forbid`, ningún módulo pode desactivar a regra cun `#[allow]`. Así, esta parte do programa, que será a máis grande, conta con todas as garantías de seguridade de memoria de Rust.
- `pub mod color;` declara o módulo `color`, que está en `src/color.rs`. Con `pub`, os outros crates poden usalo como `meiga_core::color`. O mesmo cos demais; `shell` chegou na subfase 1.6, e `input`, na 1.7. `term`, desde a 1.5, ten submódulos, así que vai nun cartafol: o módulo está en `src/term/mod.rs`.
- As dependencias, como `alacritty_terminal`, poden ter `unsafe` dentro: `forbid` só se aplica ao código deste crate.

## `crates/meiga-core/src/color.rs`

As cores e, desde a subfase 1.6, a paleta dun terminal: a cor coa que se debuxa cada número das celas ([ADR 0013](../adr/0013-da-saida-a-pantalla.md), [Tecnoloxías › A paleta e os estilos](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)).

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
```

- **`CAMPBELL`**: as 16 cores do esquema *Campbell*, o que usan por defecto o Windows Terminal e a consola de Windows desde Windows 10: negro, vermello, verde, amarelo, azul, magenta, cian e branco, e as súas versións brillantes, na orde das secuencias de cor (`30`–`37` e `90`–`97`). Os valores, nunha táboa, en [Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas).
- **`FOREGROUND`** e **`BACKGROUND`**: o texto e o fondo de meiga, `#D4D7DD` e `#16181D`, en vez dos de *Campbell*. Antes estaban na demo de `meiga.exe`, que desaparece na subfase 1.6: agora viven xunto coa paleta, e os modos de medición (`--bench`) tamén as usan.

```rust
const CUBE_LEVELS: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub colors: [Rgb; 256],
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
}
```

**`Palette`**, desde a subfase 1.6: as cores dun terminal. As celas gardan o número da súa cor, non a cor ([`Color`](#cratesmeiga-coresrctermscreenrs)), e a paleta dá a cor de cada número ao debuxalas:

| Campo | Que é |
|---|---|
| `colors` | As 256 cores que os programas escollen polo seu número: 0–15 as que teñen nome, 16–231 un cubo de 6 × 6 × 6 e 232–255 grises, de escuro a claro. |
| `foreground`, `background` | O texto e o fondo cando o programa non escolle ningunha cor (`Color::Default`). |
| `cursor` | A cor do cursor. |

- Os campos son públicos, como os de `Rgb`: calquera combinación é unha paleta válida.
- Deriva `Clone`, `Debug`, `PartialEq` e `Eq`, pero non `Copy`, ao contrario ca `Rgb`: son 777 bytes, e así cada copia vese no código, cun `clone`.

```rust
impl Palette {
    pub const MEIGA: Palette = Palette::new(CAMPBELL, FOREGROUND, BACKGROUND, FOREGROUND);

    pub const fn new(named: [Rgb; 16], foreground: Rgb, background: Rgb, cursor: Rgb) -> Self {
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
        }
    }
}
```

**`new`** crea a paleta do xterm coas 16 cores `named`: as outras 240 son as mesmas en todos os terminais ([Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)).

- **O cubo**, da 16 á 231: `cube = index − 16`, e os seus tres díxitos en base 6 son o vermello (`cube / 36`), o verde (`cube / 6 % 6`) e o azul (`cube % 6`), así que a cor `16 + 36 r + 6 g + b` ten as compoñentes `r`, `g` e `b`. Cada díxito escolle un nivel de `CUBE_LEVELS`: 0 e, desde `0x5f`, en pasos de `0x28` (40) ata `0xff`. Por exemplo, a 67 é 16 + 36 × 1 + 6 × 2 + 3: `(0x5f, 0x87, 0xaf)`.
- **Os grises**, da 232 á 255: `8 + 10 i`, de 8 a 238. O negro e o branco xa están no cubo, na 16 e na 231. O comentario lembra por que o `as u8` non corta nada: o maior é 238.
- **Por que `const fn`.** Así `MEIGA` é unha constante, que o compilador calcula mentres compila: as 256 cores van no programa como datos, e crear un terminal ou unha pantalla só as copia. `Screen::new`, que tamén é `const`, só pode usar unha paleta constante ([`screen.rs`](#cratesmeiga-coresrctermscreenrs)). Nunha `const fn` non se pode usar `for`, que chama os métodos do trait `Iterator`: por iso o bucle é un `while` cun índice.
- **`MEIGA`** son as cores de meiga ata que haxa temas (subfase 8.3): as 16 de *Campbell*, o texto e o fondo de meiga, e o cursor da cor do texto ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **`Default`** dá `MEIGA`: así os tipos que levan unha paleta, como `Screen`, poden derivar `Default`.

**Probas.** Dúas:

| Proba | Que comproba |
|---|---|
| `the_palette_has_the_256_colours_of_xterm` | As 16 primeiras cores son as de *Campbell*; as esquinas do cubo (a 16, negra, e a 231, branca) e unha cor de dentro (a 67); o primeiro e o último gris (8 e 238); o texto, o fondo e o cursor de `MEIGA`; e que `Default` é `MEIGA`. |
| `mixing_goes_from_one_colour_to_the_other` | Con 0 e con 255, as dúas cores; con 85, do negro ao branco, o gris 85, e do branco ao negro, o 170; e con 128, case a metade, redondea ao máis próximo: o verde, de 200 a 100, dá 150 (149,8), e non 149. |

## `crates/meiga-core/src/geometry.rs`

Puntos, tamaños e rectángulos en píxeles: os tipos cos que todos os crates falan de posicións na xanela. Desde a subfase 1.4, tamén o tamaño da grella, en celas ([`GridSize`](#gridsize)).

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

Desde a subfase 1.4: o tamaño da grella do terminal, en celas, non en píxeles. É o tamaño que recibe a pseudoconsola ao creala ([Código › meiga-win](meiga-win.md#cratesmeiga-winsrcconptyrs)) e, desde a 1.5, o do núcleo do terminal. Desde a 1.6, sae do tamaño da xanela ao arrancar, dividido polo da cela, e desde a 1.9 seguirá a xanela cando cambie de tamaño.

- `u16`: ningunha grella chega a 65 535 columnas ou filas. Cada uso comproba o seu propio límite: o ConPTY, por exemplo, recibe as medidas nun `COORD`, de 16 bits con signo, así que acepta como moito 32 767, e polo menos 1.
- Deriva o mesmo ca os outros tipos do módulo, e os campos son públicos: calquera par de números é un tamaño, e quen o usa decide se lle serve.

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

O intervalo de `max_fps = "auto"`: 30 fotogramas por segundo (33,3 ms) nas sesións de Escritorio remoto e 60 (16,7 ms) en local. Os valores definitivos decídense con medicións na subfase 1.10. `Duration` é un tempo en segundos e nanosegundos, da biblioteca estándar.

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

É `Copy`, de 12 bytes: vai por valor no evento da xanela, `Event::Key` ([Código › meiga-win › `event.rs`](meiga-win.md#cratesmeiga-winsrceventrs)).

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
        // A VT terminal only sends something when a key goes down.
        if !event.down {
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
- **`high_surrogate`**, o único estado: a primeira metade dun carácter, que espera pola segunda en VT. Cambiar ao win32-input-mode descártaa.

```rust
fn push_vt(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
    // The first half of a character only waits for the second one.
    let high_surrogate = self.high_surrogate.take();
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
    let mut utf8 = [0; 4];
    out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
}
```

**`push_vt_character`** envía o carácter da tecla en UTF-8:

- **Ctrl sen carácter.** Windows dá o código de control de Ctrl cunha letra (Ctrl+C escribe `0x03`), pero nada para Ctrl+2 e un espazo para Ctrl+Espazo. Para esas, [`control_code`](#os-axudantes-das-secuencias) dá o que envía un terminal VT, e se a tecla non ten ningún, non se envía nada.
- **As dúas metades.** A primeira (`0xD800`–`0xDBFF`) gárdase; a segunda (`0xDC00`–`0xDFFF`) xúntase con ela no carácter enteiro. Unha segunda metade sen a primeira, ou unha primeira que non vai seguida da segunda, pérdese.
- **AltGr** escribe os seus caracteres con Ctrl e Alt premidos ([Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows)): un carácter imprimíbel con Ctrl e Alt é de AltGr e vai tal cal. Con Alt só, vai un ESC diante: Alt+f é `ESC f`.
- `encode_utf8` escribe o carácter nun *array* de 4 bytes, sen reservar memoria.

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

fn push_decimal(out: &mut Vec<u8>, value: u16) {
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
- **`push_decimal`** escribe as cifras de atrás cara a diante nun *array* de 5, as que ten como moito un `u16`, sen `format!`, que reservaría un texto.
- O módulo **`vk`** ten os códigos de tecla virtual que a codificación VT trata á parte, e `F5_TO_F12`, os números das teclas de función, que saltan algúns por razóns históricas. `meiga-core` non depende de `windows-sys`, así que define os seus.

**Probas.** Dez:

| Proba | Que comproba |
|---|---|
| `win32_input_mode_sends_the_whole_key_events` | Eventos que enviou o Windows Terminal co teclado español: a A ao baixar e ao subir, AltGr+2 (@), Maiús+Intro, a frecha esquerda, unha tecla morta e Maiús só; as repeticións, e un 0 que vai coma 1; e os números máis grandes. |
| `vt_sends_the_characters_in_utf8_when_keys_go_down` | a, ñ e €, en UTF-8; unha tecla morta non envía nada, e a seguinte, á; nin as teclas que soben nin Maiús só envían nada. |
| `vt_joins_the_two_halves_of_a_character` | 😀 en dous eventos dá os seus 4 bytes, tamén cos eventos de subir no medio e con Alt; unha metade soa pérdese. |
| `vt_cursor_keys_follow_the_application_mode_and_the_modifiers` | As frechas, Inicio e Fin, co modo aplicación e coas modificadoras. |
| `vt_editing_and_function_keys_send_their_sequences` | Insert, Supr, RePáx, AvPáx, F1, F4, F5, F10 e F12, con e sen modificadoras. |
| `vt_backspace_tab_enter_and_escape_send_control_codes` | Retroceso, Tab, Intro (tamén co modo *LNM*), Esc, Pausa e Ctrl+Inter, con Maiús, Ctrl e Alt. |
| `vt_ctrl_sends_control_codes` | Ctrl+C, Ctrl+Espazo, Ctrl+2, Ctrl+3, Ctrl+7, Ctrl+8 e Ctrl+Alt+A; unha tecla sen código de control non envía nada. |
| `vt_alt_sends_escape_first_but_altgr_does_not` | Alt+a é `ESC a`; AltGr+2 e Ctrl+Alt+2 envían `@`, e o `[` de AltGr, `[`, sen ESC. |
| `vt_repeats_the_keys_that_repeated` | Unha tecla repetida tres veces envía tres veces o seu, tamén unha frecha; con 0 repeticións, unha. |
| `focus_events_only_go_if_the_program_asks_for_them` | `ESC [ I` e `ESC [ O` co modo, e nada sen el. |

## `crates/meiga-core/src/term/mod.rs`

O terminal: a fachada sobre `alacritty_terminal` ([ADR 0003](../adr/0003-nucleo-vt-alacritty-terminal.md) e [ADR 0012](../adr/0012-fachada-do-terminal.md)). meiga dálle o que escribe o programa e le del o que hai que mostrar e o que hai que responder, en tipos propios. Desde a subfase 1.6 ten tamén a paleta coa que se debuxan as celas, e responde ás preguntas polas cores e polo tamaño en píxeles ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Só este módulo usa os tipos de `alacritty_terminal`: cambialo por outro núcleo, ou actualizalo a unha versión cunha API distinta, só cambia este módulo. Como encaixa co resto, en [Arquitectura › O terminal](../arquitectura.md#o-terminal-da-saída-ao-que-se-mostra) e [› Da saída á pantalla](../arquitectura.md#da-saída-á-pantalla-o-aviso-a-copia-e-o-fotograma).

```rust
pub mod recording;
mod scanner;
mod screen;

pub use screen::{Cell, CellFlags, Color, Columns, Cursor, CursorShape, Screen};
```

- Os outros arquivos do cartafol `src/term/` son os submódulos de `term`.
- `recording` é público: o exemplo `replay` e as probas usan as gravacións como `meiga_core::term::recording`. `scanner` e `screen` son privados, pero os tipos de `screen` reexpórtanse con `pub use`: fóra úsanse como `meiga_core::term::Screen`, sen saber en que arquivo están.

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

- **`SCROLLBACK_LINES`**: as liñas de historial que se gardan por riba da pantalla, as que mostrará o *scrollback* (subfase 1.9). `alacritty_terminal` non as reserva ao crear o terminal: a memoria do historial medra a medida que se enche, de 1 000 en 1 000 liñas.
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
- **`Modes`**: os modos que activou o programa e que cambian o que meiga lle envía: como se codifican as teclas (win32-input-mode, as frechas e o teclado numérico en modo aplicación), os eventos de foco, o pegado entre marcas (*bracketed paste*), que eventos do rato quere e en que formato, se a roda envía frechas, se está na pantalla alternativa, se se ve o cursor, e se o salto de liña volta tamén á primeira columna. Os números de cada modo están en [Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control). `win32_input` dío o escáner, porque `alacritty_terminal` non coñece ese modo; o resto, `alacritty_terminal`.

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

`alacritty_terminal` pide o tamaño a través do trait `Dimensions`: as filas que se ven (`screen_lines`), as columnas e as filas en total. `TermSize` envolve un `GridSize` para implementalo: implementalo directamente en `GridSize`, que é público, mostraría `alacritty_terminal` fóra da fachada. O historial non conta: `total_lines` é igual a `screen_lines`. Ata a subfase 1.5 chamábase `Size`; na 1.6 cambiou de nome porque o módulo usa agora o `Size` de [`geometry.rs`](#cratesmeiga-coresrcgeometryrs), o tamaño dunha cela en píxeles, e os dous nomes chocarían.

### `Terminal`

```rust
pub struct Terminal {
    term: Term<Listener>,
    parser: Processor,
    scanner: Scanner,
    pending: Arc<Mutex<Pending>>,
    size: GridSize,
    title: Option<String>,
    theme: Palette,
    cell: Size,
    damaged: Vec<LineDamageBounds>,
}
```

| Campo | Que é |
|---|---|
| `term` | O terminal de `alacritty_terminal`: a grella, o historial, o cursor e os modos. |
| `parser` | O parser de `vte` (`Processor`), que le os bytes e chama aos métodos de `term`. Garda o estado entre lecturas: unha secuencia partida entre dous bloques, ou un carácter UTF-8 a medias. Tamén garda a saída dunha actualización sincronizada mentres non remata. |
| `scanner` | O escáner propio ([`scanner.rs`](#cratesmeiga-coresrctermscannerrs)). |
| `pending` | A cola dos eventos, compartida co `Listener`. |
| `size` | O tamaño da grella. |
| `title` | O último título que deu o programa. |
| `theme` | Desde a subfase 1.6: as cores do tema, `Palette::MEIGA` ata que haxa temas. As que cambia o programa non van eiquí: gárdaas `alacritty_terminal` ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `cell` | Desde a 1.6: o tamaño dunha cela en píxeles, para responder a `CSI 14 t`; 0 × 0 mentres non se sabe. |
| `damaged` | As filas danadas, nun `Vec` que se reutiliza en cada copia para non reservar memoria. |

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
        ..Config::default()
    };
    Self {
        term: Term::new(config, &TermSize(size), Listener(Arc::clone(&pending))),
        parser: Processor::new(),
        …
        theme: Palette::MEIGA,
        cell: Size::new(0, 0),
        damaged: Vec::new(),
    }
}

pub fn feed(&mut self, bytes: &[u8]) -> Changes {
    self.scanner.scan(bytes);
    self.parser.advance(&mut self.term, bytes);
    self.take_changes()
}
```

- **`new`** crea un terminal de polo menos `MIN_SIZE`, coa configuración de meiga:
  - 10 000 liñas de historial;
  - desde a subfase 1.6, o cursor por defecto, `DEFAULT_CURSOR`;
  - `Osc52::OnlyCopy`: os programas poden escribir no portapapeis, pero non lelo;
  - sen o protocolo de teclado de kitty: meiga usa o win32-input-mode, que é o que pide o ConPTY ([`input.rs`](#cratesmeiga-coresrcinputrs));
  - o resto, cos valores por defecto (`..Config::default()`).

  Ten as cores de meiga (`Palette::MEIGA`) e aínda non sabe o tamaño da cela: 0 × 0 ata que llo dean con `set_cell_size`. `Processor::new()` reserva xa o búfer das actualizacións sincronizadas, de 2 MiB, que Windows conta como memoria privada aínda que non se use. Que facer con el decidirase na subfase 2.2, coa memoria por tab ([ADR 0012 › Consecuencias](../adr/0012-fachada-do-terminal.md#consecuencias)).
- **`feed`** le o que escribiu o programa: primeiro o escáner e despois o parser, cos mesmos bytes, sen copialos nin cambialos. Devolve os `Changes`.

| Método | Que fai |
|---|---|
| `size` | O tamaño da grella. |
| `set_theme(theme)` | Desde a subfase 1.6: cambia as cores do tema. As que cambiou o programa seguen coa súa. Polo de agora só o usan as probas: os temas chegan na subfase 8.3. |
| `set_cell_size(cell)` | Desde a 1.6: o tamaño dunha cela en píxeles, para responder a `CSI 14 t`. Mentres non o ten, non responde. `meiga.exe` dállo ao crear o terminal, co tamaño da cela da fonte. |
| `palette` | Desde a 1.6: as cores coas que se debuxan as celas ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `take_responses(into)` | Move as respostas ao final de `into`, para envialas ao programa. `Vec::append` deixa a cola baleira pero coa súa memoria: a seguinte resposta non reserva nada. |
| `take_clipboard` | O último texto que o programa copiou ao portapapeis; despois, ningún. Se copiou varios antes de recollelos, só queda o último, como no portapapeis. |
| `title` | O título que deu o programa, se deu algún. |
| `modes` | Os modos, traducidos de `TermMode` (uns *bitflags*) aos campos de `Modes`. `win32_input` pregúntallo ao escáner. |
| `resize(size)` | Cambia o tamaño da grella, con polo menos `MIN_SIZE`. `alacritty_terminal` reorganiza as liñas longas da pantalla principal (*reflow*), e todo queda danado. |
| `sync_deadline` | Cando hai que mostrar a actualización sincronizada (`?2026`) que empezou o programa, aínda que non a remate: `vte` gárdaa como moito 150 ms desde o último `?2026h`, ou ata encher o búfer. Mentres tanto, a saída non chega á pantalla. |
| `end_sync` | Mostra a saída da actualización sincronizada que agarda, e devolve os `Changes`. É para cando vence: o terminal non ten temporizador. Desde a subfase 1.6, remátaa a xanela: antes de cada fotograma consulta `sync_deadline`, e o temporizador dos fotogramas vence a esa hora, tamén coa xanela minimizada, que non debuxa ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)). |

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

- **`palette`** dá a paleta coa que se debuxa: a do tema, coas cores que cambiou o programa por riba. A táboa de `alacritty_terminal` admite como índice un número ou un `NamedColor`, e os nomes van despois das 256 cores.
- **Por que o tema vai aparte.** Se os cambios do programa se escribisen no tema, voltar á cor orixinal non sabería cal era, e cambiar de tema borraría o que cambiou o programa. Así, `set_theme` só cambia o tema, e as cores do programa seguen por riba.
- `for (name, color) in [(…, &mut palette.foreground), …]` percorre un *array* de pares, cada un cunha referencia mutábel a un campo distinto da mesma paleta: o compilador permíteo porque non se solapan.
- Devolve unha paleta nova en cada chamada, sen reservar memoria: son 777 bytes na pila. `copy_damage` chámaa en cada copia, e `take_changes`, se hai preguntas.
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
    let grid = self.term.grid();
    if full {
        for row in 0..self.size.rows {
            copy_row(grid, screen, row);
        }
    } else {
        for damage in &self.damaged {
            match u16::try_from(damage.line) {
                Ok(row) if row < self.size.rows => copy_row(grid, screen, row),
                _ => {}
            }
        }
    }
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

Copia na `Screen` o que cambiou desde a copia anterior ([ADR 0012](../adr/0012-fachada-do-terminal.md)) e, desde a subfase 1.6, a paleta ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)):

1. Se a `Screen` ten outro tamaño, cámbiao: todas as súas celas quedan en branco e cambiadas, e cópianse todas as filas.
2. `term.damage()` dá as filas danadas desde a última copia: `Full`, todas, ou `Partial`, a lista das filas danadas. `alacritty_terminal` dá `Full` cando cambia moito: ao borrar a pantalla, ao desprazala (cada salto de liña na última fila), ao cambiar á pantalla alternativa ou de tamaño, nun reinicio…
3. Cada fila danada cópiase **enteira** con `copy_row`, que compara cela a cela e só marca as que cambiaron. **Por que enteira:** `alacritty_terminal` dana as celas que escribe, pero non sempre outras da mesma fila que cambian con elas. Ao escribir sobre a metade dereita dun carácter ancho, por exemplo, borra tamén a esquerda, e esa non queda danada. Atopouno a proba aleatoria do terminal. Copiar e comparar unha fila custa pouco, e marcar só as celas distintas evita debuxar o que non cambiou, tamén co dano `Full`: despois de borrar a pantalla, as filas que xa estaban en branco non se marcan.
4. **O cursor.** Se cambiou (a posición, a forma ou o pestanexo), márcanse as súas celas onde estaba, antes de cambialo, e onde está, despois, con [`mark_cursor_changed`](#cratesmeiga-coresrctermscreenrs): dúas celas, por se está sobre un carácter ancho, e só unha na última columna, e só se se debuxa. A vista do terminal debuxa o cursor por riba delas. Oculto (`Hidden`), non marca nada: un *spinner* que oculta o cursor só redebuxa os seus caracteres, e unha copia sen saída nova non marca nada.
5. **A paleta.** `set_palette` compara a paleta do terminal coa da `Screen`: se cambiou unha das 256 cores, a do texto ou a do fondo, cambian todas as celas; se só cambiou a do cursor, só as do cursor. **Por que aparte:** cando o programa cambia unha cor, `alacritty_terminal` dana a pantalla enteira e as filas cópianse, pero as celas gardan o número da súa cor, que é o mesmo: comparadas, ningunha cambiou. E cando só cambia a do cursor, non dana nada. Compróbao a proba `a_change_of_the_palette_changes_every_cell`.
6. `reset_damage` esquece o dano, para a seguinte copia.

- `Ok(row) if row < self.size.rows => …` é un brazo de `match` cunha garda (`if`): só se copian as filas que existen.

### `cursor` e `take_changes`

- **`cursor`** le o cursor da grella (`grid.cursor.point`) e do estado do terminal. Desde a subfase 1.6 é público: cando o programa remata, a sesión mira se o cursor está ao principio dunha fila, para escribir nunha fila propia a liña co código de saída ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)).
  - a forma é `Hidden` se o programa o ocultou (`?25l`); se non, a que pediu (`cursor_style()`), que tamén di se pestanexa, ou `DEFAULT_CURSOR` se non pediu ningunha;
  - sobre a metade dereita dun carácter ancho, pasa á columna anterior, para debuxalo sobre o carácter enteiro, pero só se esa cela é a súa metade esquerda: coma en `copy_row`, unha metade dereita soa é unha cela en branco, e o cursor debúxase nela;
  - `point_to_viewport` converte a posición na grella, que inclúe o historial, na fila que se ve: se queda fóra (mentres se mira o historial, na subfase 1.9), o cursor non se debuxa.

  **Por que non `renderable_content()`**, que dá o cursor como o debuxa Alacritty: sobre unha metade dereita, pásao á columna anterior sen comprobar que exista. Borrar caracteres (`ESC [ P`) pode deixar unha metade dereita soa na primeira columna, co cursor enriba, e entón a resta baixa de cero: nas compilacións de depuración, que comproban os desbordamentos tamén nas dependencias, é un *panic*; en *release*, a columna dá a volta e o cursor desaparece. Atopouno a revisión do código, e compróbao a proba `the_cursor_on_a_lone_right_half_stays_in_its_cell`.
- **`take_changes`**, desde a subfase 1.6, responde primeiro as preguntas pendentes ([As preguntas](#as-preguntas-polas-cores-e-polo-tamaño)). Despois colle os `Changes` da cola e déixaa baleira: `mem::take` devolve o valor e deixa no seu sitio o valor por defecto. O título só se move se cambiou.

### Os axudantes

- **`rgb`**, desde a subfase 1.6, converte unha cor de `alacritty_terminal` nun `Rgb` de meiga. Úsana `palette` e `color`.
- **`lock`** bloquea a cola. Se un fío fallase (*panic*) mentres a tiña bloqueada, o `Mutex` quedaría «envelenado» (*poisoned*), e `lock()` daría un erro desde entón. Ningún cambio da cola pode quedar a medias, así que `unwrap_or_else(PoisonError::into_inner)` cóllea igual.
- **`at_least_min_size`**: o tamaño, con polo menos 2 columnas e 1 fila.
- **`copy_row`** copia unha fila que se ve e marca as celas que cambiaron:

  ```rust
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
      …
  }
  ```

  - `viewport_to_point` converte a fila que se ve na liña da grella, contando o historial (`display_offset`), e `&grid[point.line][..]` son as súas celas.
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
- **`color`**: as 16 cores con nome (`Named`) son as 16 primeiras da paleta, de `Indexed(0)` a `Indexed(15)`. Os outros nomes son as cores por defecto do texto e do fondo, e outras que só usa o renderizador do Alacritty (a do cursor, as atenuadas…), que ningunha secuencia lle dá a unha cela: `Default`. `Indexed` pasa igual, e `Spec` (*truecolor*), con `rgb`. `Ok(index @ 0..16)` é un patrón: o valor ten que estar entre 0 e 15, e `@` gárdao en `index`.
- **`flags`** traduce os estilos cunha táboa (`MAP`) de 11 pares, só os que debuxa meiga: os *flags* internos de `alacritty_terminal`, como os das metades dos caracteres anchos, non pasan. `fold` xúntaos con `|`.

As probas están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs).

## `crates/meiga-core/src/term/screen.rs`

O que mostra o terminal, en tipos de meiga: as celas, as súas cores e estilos, o cursor, a paleta e as celas que cambiaron. Desde a subfase 1.6, a vista do terminal de `meiga.exe` debuxa desde eiquí, sen coñecer `alacritty_terminal` ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

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
- **`Cell`**: unha cela: o carácter (`CellChar`, de [`text.rs`](#cratesmeiga-coresrctextrs)), a cor do texto, a do fondo e os estilos. A cela por defecto (`Default`) é unha cela en branco, coas cores por defecto e sen estilos.
- **`CursorShape`**: bloque, subliñado, barra vertical, o contorno dun bloque, ou oculto.
- **`Cursor`**: a fila, a columna, a forma e se pestanexa: por defecto si, salvo que o programa pida un cursor fixo. meiga só o fai pestanexar se Windows o permite ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)).
- **`Columns`**: as columnas dunha fila, de `start` (incluída) a `end` (excluída); ningunha se `end` non está despois de `start`. `union` dá as que cobren as dúas.

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
  - se cambiaron as 256 cores, a do texto ou a do fondo, cambian todas as celas: calquera pode ter unha cor que cambiou. Tamén cambian as marxes da xanela, que teñen a cor do fondo: con `all_changed`, a vista do terminal redebuxa a xanela enteira;
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

As dezaoito probas do terminal, nun arquivo aparte para que `mod.rs` sexa máis curto. É un submódulo de `term` (`#[cfg(test)] mod tests;`, ao final de `mod.rs`), así que ve tamén os seus tipos privados. Tres axudantes: `copy` copia o terminal enteiro nunha `Screen` nova; `responses` recolle as respostas; e `narrow` crea unha cela cun carácter, sen cores nin estilos.

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
| `copying_only_the_damage_gives_the_same_screen_as_copying_everything` | **A invariante da copia.** 300 casos aleatorios, de ata 79 anacos dunha lista de 70: texto, caracteres anchos e combinados, movementos do cursor, borrados, inserir e borrar caracteres e liñas, desprazamentos e rexións, a pantalla alternativa, estilos, reinicios… Cada caso lese en anacos de ata 16 bytes, por veces cambiando o tamaño, con grellas de 2 × 1 a 80 × 24. Despois de cada anaco, a pantalla copiada só co dano ten que ser igual, cela a cela e no cursor, á dun segundo terminal copiado enteiro. Esta proba atopou o oco do dano de `alacritty_terminal` cos caracteres anchos. |

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

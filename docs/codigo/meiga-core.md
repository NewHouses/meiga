# Código › `meiga-core`

> Estado: subfase 1.2. Contén as cores, a xeometría (puntos, tamaños e rectángulos), as zonas que cambiaron (*damage*) e o ritmo dos fotogramas.

**Responsabilidade:** toda a lóxica que non depende de Windows: o modelo de xanelas, tabs e grupos; a fachada do terminal; a codificación do teclado e do rato; a busca difusa; e a configuración. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeiga-corecargotoml) | O crate, sen dependencias. |
| [`src/lib.rs`](#cratesmeiga-coresrclibrs) | A raíz do crate: os módulos. |
| [`src/color.rs`](#cratesmeiga-coresrccolorrs) | `Rgb`, o tipo das cores. |
| [`src/geometry.rs`](#cratesmeiga-coresrcgeometryrs) | `Point`, `Size` e `Rect`: posicións, tamaños e rectángulos en píxeles. |
| [`src/damage.rs`](#cratesmeiga-coresrcdamagers) | `Damage`: as zonas da xanela que cambiaron desde o último fotograma. |
| [`src/pacing.rs`](#cratesmeiga-coresrcpacingrs) | `FramePacer`: cando debuxar o seguinte fotograma. |

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
pub mod damage;
pub mod geometry;
pub mod pacing;
```

- `lib.rs` é a raíz dunha biblioteca: o compilador empeza por este arquivo, e desde el decláranse os demais módulos.
- As liñas que empezan por `//!` son **documentación do crate**. `cargo doc` convérteas en documentación HTML.
- `#![forbid(unsafe_code)]` é un **atributo interno** (`#!`), que se aplica a todo o crate. Prohíbe calquera bloque `unsafe` en `meiga-core`. Co nivel `forbid`, ningún módulo pode desactivar a regra cun `#[allow]`. Así, esta parte do programa, que será a máis grande, conta con todas as garantías de seguridade de memoria de Rust.
- `pub mod color;` declara o módulo `color`, que está en `src/color.rs`. Con `pub`, os outros crates poden usalo como `meiga_core::color`. O mesmo cos outros tres.

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

Unha cor en RGB, con 8 bits (0–255) por canle: vermello (`r`), verde (`g`) e azul (`b`). Desde a subfase 1.2 úsase para todo o que se debuxa no *framebuffer*; máis adiante servirá tamén para o texto, os temas e a configuración.

- Está en `meiga-core`, e non nun crate de Windows, porque unha cor non depende de Windows. Cada uso convértea ao seu formato: `meiga-render` convértea nun píxel do *framebuffer* (`0x00RRGGBB`) e, para as brochas de GDI, nun `COLORREF` (`0x00BBGGRR`); ver [Código › meiga-render](meiga-render.md#pixel).
- `#[derive(…)]` fai que o compilador escriba automaticamente estas implementacións:
  - `Clone` e `Copy`: unha cor son 3 bytes, así que se copia ao asignala ou ao pasala a unha función, en vez de *moverse* (en Rust, por defecto, asignar un valor transfire a súa propiedade e o orixinal xa non se pode usar);
  - `Debug`: permite mostrala nas mensaxes de diagnóstico con `{:?}`;
  - `PartialEq` e `Eq`: permiten comparar dúas cores con `==`;
  - `Hash`: permite usala como clave nun `HashMap`, por exemplo nunha futura caché de glifos por cor.
- `const fn new`: unha función **constante**, que o compilador pode executar mentres compila. Por iso as cores se poden declarar como constantes: `const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);`.
- Os campos son públicos (`pub`): calquera combinación de tres bytes é unha cor válida, así que non hai ningunha regra que protexer.

## `crates/meiga-core/src/geometry.rs`

Puntos, tamaños e rectángulos en píxeles: os tipos cos que todos os crates falan de posicións na xanela.

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

**Probas.** Oito, unha por propiedade: que `right` e `bottom` non se inclúen, que os rectángulos invertidos están baleiros, `from_size` (tamén co tamaño máximo), `spanning` nas dúas ordes, a intersección e a unión (tamén con rectángulos baleiros), `contains`, e que os lados de `border` cobren o bordo sen solaparse nin saír dun rectángulo estreito.

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

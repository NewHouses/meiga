# Código › `meiga-render`

> Estado: subfase 1.2. Contén o *framebuffer*, o lenzo co que se debuxa nel e a súa presentación na xanela con GDI.

**Responsabilidade:** o renderizado por CPU. Compón no *framebuffer* só o que cambiou e presenta na xanela só os rectángulos modificados. Desde a subfase 1.3, ademais, rasterizará cada glifo unha soa vez con DirectWrite e gardarao nunha caché. Ver [Arquitectura › Fotogramas](../arquitectura.md#fotogramas-do-cambio-á-pantalla).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeiga-rendercargotoml) | As dependencias: `meiga-core` e `windows-sys`. |
| [`src/lib.rs`](#cratesmeiga-rendersrclibrs) | A raíz do crate: os módulos e o que se publica. |
| [`src/canvas.rs`](#cratesmeiga-rendersrccanvasrs) | `Canvas`, o lenzo: debuxar en píxeles en memoria, sen saír nunca deles. |
| [`src/error.rs`](#cratesmeiga-rendersrcerrorrs) | `Error`: unha chamada a GDI que fallou. |
| [`src/gdi.rs`](#cratesmeiga-rendersrcgdirs) | `Surface`, o *framebuffer* nunha *DIB section*, e a súa presentación con `BitBlt`. |

Que son as *DIB sections* e como funcionan `BitBlt` e `GdiFlush` explícase en [Tecnoloxías › DIB sections e GDI](../tecnoloxias.md#24-dib-sections-e-gdi-o-framebuffer-e-a-súa-presentación).

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
windows-sys = { workspace = true, features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
] }

[lints]
workspace = true
```

- A estrutura é a mesma que en [`meiga-core`](meiga-core.md#cratesmeiga-corecargotoml).
- `meiga-core.workspace = true`: depende de `meiga-core`, coa ruta que define o `Cargo.toml` raíz, para usar os seus tipos: as cores e os rectángulos.
- `windows-sys`, coa mesma versión ca en `meiga-win`, e só dúas *features*: os tipos básicos (`HWND`, `RECT`, `COLORREF`) e GDI. Que é `windows-sys` explícase en [Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust).

## `crates/meiga-render/src/lib.rs`

```rust
//! CPU renderer of meiga.
//!
//! Composes in a framebuffer only what changed, and presents in the window
//! only the rectangles that changed. From subphase 1.3, it will also
//! rasterize glyphs once with DirectWrite into a cache.
//!
//! The `unsafe` code of the crate, which calls GDI, is in `gdi.rs`, and every
//! block explains why it is sound in a `// SAFETY:` comment. The other
//! modules forbid it.

mod canvas;
mod error;
mod gdi;

pub use canvas::{Canvas, pixel};
pub use error::Error;
pub use gdi::{Fit, Surface};
```

- Os tres módulos son privados (`mod`, sen `pub`), e a raíz publica só os tipos que necesitan os demais crates: `meiga_render::Surface`, e non `meiga_render::gdi::Surface`.
- **Onde está o `unsafe`.** Este crate chama a GDI, e iso require `unsafe`. Todo ese código está en `gdi.rs`. Os outros dous módulos levan `#![forbid(unsafe_code)]`, así que o compilador garante que o lenzo, que é o código que máis vai medrar (o texto, o cursor, a selección…), non ten ningún bloque inseguro.

## `crates/meiga-render/src/canvas.rs`

O **lenzo** (*canvas*): unha vista sobre os píxeles do *framebuffer*, coas operacións de debuxo. É código Rust puro, sen GDI, e por iso se proba sen ningunha xanela.

### `pixel`

```rust
pub const fn pixel(color: Rgb) -> u32 {
    (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
}
```

Converte unha cor no formato dos píxeles do *framebuffer*: un `u32` co vermello no byte alto, `0x00RRGGBB`. Na memoria, como os procesadores x86 gardan primeiro o byte baixo, quedan na orde azul, verde, vermello e un byte sen usar: o formato **BGRA** de 32 bits que entende GDI. É distinto de `COLORREF`, que garda as cores ao revés (`0x00BBGGRR`, ver [`colorref`](#colorref-e-with_room)).

### `Canvas`

```rust
pub struct Canvas<'a> {
    pixels: &'a mut [u32],
    stride: usize,
    size: Size,
    clip: Rect,
}
```

- `pixels`: os píxeles, fila tras fila, como unha porción (*slice*) de `u32`. `&'a mut` é unha referencia prestada e exclusiva: mentres exista o lenzo, ninguén máis pode usar eses píxeles, e o lenzo non pode vivir máis cós píxeles (`'a`).
- `stride`: cantos píxeles hai desde o comezo dunha fila ata o comezo da seguinte. Pode ser máis có ancho, cando a memoria é maior ca o tamaño en uso (ver [`Fit::Room`](#fit-e-surface)): o píxel (*x*, *y*) está na posición `y * stride + x`.
- `size`: o tamaño en uso.
- `clip`: o **rectángulo de recorte**. Ningún debuxo sae del, e el nunca sae do lenzo.

```rust
pub fn new(pixels: &'a mut [u32], size: Size, stride: usize) -> Option<Self> {
    let width = usize::try_from(size.width).ok()?;
    let height = usize::try_from(size.height).ok()?;
    if width > stride || i32::try_from(size.width.max(size.height)).is_err() {
        return None;
    }
    let needed = match height {
        0 => 0,
        _ => stride.checked_mul(height - 1)?.checked_add(width)?,
    };
    if pixels.len() < needed {
        return None;
    }
    Some(Self { pixels, stride, size, clip: Rect::from_size(size) })
}
```

Comproba que as filas caben na memoria antes de crear o lenzo, e devolve `None` se non caben. Así, as operacións de debuxo nunca poden escribir fóra dos píxeles.

- `?` despois dun `Option` remata a función con `None` se o valor é `None`: `ok()?` converte un erro de conversión en `None`, e `checked_mul` e `checked_add` dan `None` se o resultado non cabe nun `usize`.
- O ancho non pode ser maior cá distancia entre filas, e o tamaño ten que caber nun `i32`, o tipo das coordenadas dos rectángulos.
- A última fila só necesita `width` píxeles, non `stride`.

```rust
pub fn empty() -> Canvas<'static> { … }
pub fn clipped(&mut self, rect: Rect) -> Canvas<'_> {
    Canvas {
        pixels: &mut *self.pixels,
        stride: self.stride,
        size: self.size,
        clip: self.clip.intersection(rect),
    }
}
```

- `empty` crea un lenzo sen píxeles, no que non se debuxa nada. `'static` porque non toma prestado nada.
- **`clipped`** dá un lenzo sobre os mesmos píxeles, cun recorte máis pequeno: a intersección do actual con `rect`. É o que fai que debuxar só o que cambiou sexa sinxelo: a escena debúxase enteira, sen saber que cambiou, sobre un lenzo recortado ao rectángulo que cambiou, e só se tocan os seus píxeles. `&mut *self.pixels` presta de novo os píxeles (*reborrow*): mentres viva o lenzo novo, o orixinal non se pode usar.

```rust
pub fn fill(&mut self, rect: Rect, color: Rgb) {
    let area = rect.intersection(self.clip);
    if area.is_empty() {
        return;
    }
    let value = pixel(color);
    let (left, right) = (area.left as usize, area.right as usize);
    for y in area.top as usize..area.bottom as usize {
        let row = y * self.stride;
        self.pixels[row + left..row + right].fill(value);
    }
}
```

**Encher un rectángulo** cunha cor, a única operación de debuxo que necesita a subfase 1.2.

- Primeiro recórtase ao `clip`. Como o recorte está dentro do lenzo, as coordenadas non son negativas, e `as usize` é seguro.
- Cada fila é unha porción contigua de píxeles, e `slice::fill` éncheas co mesmo valor. O compilador convérteo en instrucións que escriben varios píxeles á vez.
- Rust comproba os índices: se algo estivese mal, o programa pararía (*panic*) en vez de escribir fóra da memoria.

```rust
pub fn outline(&mut self, rect: Rect, thickness: u32, color: Rgb) {
    for side in rect.border(thickness) {
        self.fill(side, color);
    }
}

pub fn get(&self, x: u32, y: u32) -> Option<u32> { … }
```

- `outline` debuxa só o bordo dun rectángulo, por dentro, cos catro lados de [`Rect::border`](meiga-core.md#cratesmeiga-coresrcgeometryrs).
- `get` le un píxel, ou `None` fóra do lenzo. Úsano as probas.

### Probas

| Proba | Que comproba |
|---|---|
| `pixels_are_red_green_blue_from_the_high_byte` | A orde das canles: `pixel` dá `0x00RRGGBB`. |
| `rows_must_fit_in_the_memory` | `Canvas::new` acepta as filas que caben, tamén cun `stride` maior có ancho, e rexeita as que non. |
| `fill_is_cut_to_the_canvas_and_respects_the_stride` | Un rectángulo que sae polos lados enche só a parte de dentro, e os píxeles que sobran ao final de cada fila non se tocan. |
| `clipped_canvases_only_draw_inside_the_clip` | Un lenzo recortado, e outro recortado sobre el, só pintan os píxeles da intersección. |
| `outline_draws_only_the_border` | `outline` pinta os 12 píxeles do bordo dun cadrado de 4 × 4, e nada dentro nin fóra. |
| `an_empty_canvas_draws_nothing` | O lenzo baleiro non pinta nada. |

## `crates/meiga-render/src/error.rs`

```rust
#[derive(Debug)]
pub struct Error {
    function: &'static str,
    os_error: Option<io::Error>,
}

impl Error {
    pub(crate) fn last(function: &'static str) -> Self { … }

    pub(crate) fn too_big(function: &'static str) -> Self {
        Self {
            function,
            os_error: Some(io::ErrorKind::OutOfMemory.into()),
        }
    }
}
```

Unha chamada a GDI que fallou. É igual ca o [`Error` de `meiga-win`](meiga-win.md#cratesmeiga-winsrcerrorrs): garda o nome da función e o código de erro de Windows, se o hai, e móstrase como `CreateDIBSection failed: …`. Os dous crates teñen o seu propio tipo porque non dependen un do outro.

- `too_big`: o erro dunha función que nin sequera se chega a chamar, porque o que tería que reservar non cabe na memoria (por exemplo, un *framebuffer* de máis de 2 000 millóns de píxeles). `io::ErrorKind::OutOfMemory.into()` crea un erro de entrada e saída dese tipo, que se mostra como `out of memory`.

## `crates/meiga-render/src/gdi.rs`

O *framebuffer* e a súa presentación na xanela. É o único arquivo de `meiga-render` con `unsafe`. Os fragmentos son extractos, sen os comentarios `// SAFETY:`.

### `Fit` e `Surface`

```rust
pub enum Fit {
    Exact,
    Room,
}

pub struct Surface {
    dib: Dib,
    size: Size,
}
```

- `Surface` é o *framebuffer*: os píxeles nunha [*DIB section*](#dib) e o tamaño en uso, que é o da xanela.
- **`Fit`** di como se adapta a memoria a un tamaño novo:
  - `Exact`: exactamente ese tamaño, para non usar máis memoria da necesaria;
  - `Room`: se o tamaño novo cabe na memoria, mantense; se non, resérvase un cuarto máis en cada dirección. Mentres o usuario redimensiona a xanela co rato, chegan decenas de tamaños seguidos, e así non hai que reservar memoria en cada un.

```rust
pub fn resize(&mut self, size: Size, fit: Fit) -> Result<bool, Error> {
    let capacity = self.dib.capacity;
    let fits = size.width <= capacity.width && size.height <= capacity.height;
    let target = match fit {
        Fit::Exact if capacity == Dib::capacity_for(size) => None,
        Fit::Exact => Some(size),
        Fit::Room if fits => None,
        Fit::Room => Some(Size::new(
            with_room(size.width).max(capacity.width),
            with_room(size.height).max(capacity.height),
        )),
    };
    if let Some(target) = target {
        self.dib = Dib::new(target)?;
    }
    self.size = size;
    Ok(target.is_some())
}
```

- `target` é a memoria nova que hai que reservar, ou `None` se a actual vale. As ramas con `if` son **gardas**: a rama só se escolle se, ademais do patrón, se cumpre a condición.
- Con `Room`, a memoria nova nunca é máis pequena cá anterior en ningunha dirección (`max`): se a xanela medra en ancho e encolle en alto, non se perde o alto que xa había.
- Asignar `self.dib = Dib::new(target)?` destrúe a *DIB section* anterior (o seu `Drop`) despois de crear a nova. Se a nova non se pode crear, `?` devolve o erro e a anterior segue intacta.
- Devolve `true` se a memoria é nova: entón os píxeles perdéronse, e hai que debuxar todo de novo.

```rust
pub fn canvas(&mut self) -> Canvas<'_> {
    let stride = self.dib.capacity.width as usize;
    let len = stride * self.dib.capacity.height as usize;
    let pixels = unsafe { slice::from_raw_parts_mut(self.dib.bits.as_ptr(), len) };
    Canvas::new(pixels, self.size, stride).unwrap_or_else(Canvas::empty)
}
```

O lenzo sobre os píxeles en uso.

- `slice::from_raw_parts_mut` crea unha porción de Rust a partir dun punteiro e dunha lonxitude: os píxeles da *DIB section*. É `unsafe` porque o compilador non pode saber se a memoria é válida. O comentario `// SAFETY:` do código explica por que o é: os píxeles existen mentres exista `self.dib`, e `&mut self` impide que outro código Rust os use mentres vive o lenzo. Tampouco os usa GDI: as únicas funcións que os len son as copias de `copy`, que rematan antes de voltar.
- O ancho da memoria é o `stride` do lenzo; o tamaño en uso pode ser menor.
- `unwrap_or_else(Canvas::empty)`: se o lenzo non se puidese crear, que non pasa porque o tamaño en uso sempre cabe na memoria, debuxaríase nun lenzo baleiro en vez de parar.

```rust
pub fn present(&self, window: HWND, rects: &[Rect]) -> Result<(), Error> {
    if rects.is_empty() {
        return Ok(());
    }
    let target = WindowDc::get(window)?;
    self.copy(target.0, rects)
}

pub fn paint(&self, dc: HDC, rect: Rect) -> Result<(), Error> {
    self.copy(dc, &[rect])
}
```

As dúas formas de levar os píxeles á xanela:

- **`present`**: cando meiga decide debuxar un fotograma, copia os rectángulos que cambiaron. Pide o contexto de dispositivo da xanela con `GetDC` (dentro de [`WindowDc`](#windowdc-e-brush)).
- **`paint`**: cando Windows pide pintar (`WM_PAINT`), copia o rectángulo que pide no contexto que deu `BeginPaint`.

```rust
fn copy(&self, target: HDC, rects: &[Rect]) -> Result<(), Error> {
    let bounds = Rect::from_size(self.size);
    let mut result = Ok(());
    for rect in rects {
        let area = rect.intersection(bounds);
        if area.is_empty() {
            continue;
        }
        let (width, height) = (area.width() as i32, area.height() as i32);
        let copied = unsafe {
            BitBlt(target, area.left, area.top, width, height, self.dib.dc, area.left, area.top, SRCCOPY)
        };
        if copied == 0 && result.is_ok() {
            result = Err(Error::last("BitBlt"));
        }
    }
    unsafe { GdiFlush() };
    result
}
```

- Cada rectángulo recórtase ao tamaño en uso, e cópiase ao mesmo sitio da xanela con **`BitBlt`** (*bit block transfer*): desde o contexto da *DIB section* ata o de destino, píxel a píxel (`SRCCOPY`).
- Se unha copia falla, gárdase o primeiro erro e cópianse as demais igual.
- **`GdiFlush`.** GDI pode agrupar as operacións e facelas máis tarde. As copias len os píxeles, que o lenzo pode cambiar en canto `copy` volta: `GdiFlush` espera a que rematen todas. É o que fai seguro o `unsafe` de `canvas`.

```rust
pub fn mark(&self, window: HWND, rects: &[Rect], color: Rgb, thickness: u32) -> Result<(), Error> { … }
```

Debuxa directamente na xanela, e non no *framebuffer*, o bordo de cada rectángulo, cunha brocha (`FillRect` para cada lado). Son as **marcas** da demo: deixan ver que zonas presentou o último fotograma. Como non están no *framebuffer*, presentar de novo eses rectángulos bórraas.

### `Dib`

```rust
struct Dib {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: NonNull<u32>,
    capacity: Size,
}
```

A ***DIB section*** e o contexto de dispositivo en memoria no que está seleccionada. É dona dos dous, e destrúeos no seu `Drop`.

- `bits`: o punteiro aos píxeles. `NonNull` é un punteiro que nunca é nulo: o tipo xa o garante.
- `previous`: o *bitmap* que tiña o contexto antes, que hai que voltar seleccionar antes de destruír o noso.
- `capacity`: o tamaño da memoria. É polo menos 1 × 1 (`capacity_for`), porque GDI non crea *bitmaps* baleiros: unha xanela de 0 × 0 ten igual 1 píxel de memoria.

```rust
fn new(size: Size) -> Result<Self, Error> {
    let capacity = Self::capacity_for(size);
    let (Ok(width), Ok(height)) = (i32::try_from(capacity.width), i32::try_from(capacity.height))
    else {
        return Err(Error::too_big("CreateDIBSection"));
    };
    if (capacity.area()).checked_mul(4).and_then(|bytes| isize::try_from(bytes).ok()).is_none() {
        return Err(Error::too_big("CreateDIBSection"));
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..Default::default()
        },
        ..Default::default()
    };
    let dc = unsafe { CreateCompatibleDC(ptr::null_mut()) };
    …
    let bitmap = unsafe { CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, ptr::null_mut(), 0) };
    …
    let previous = unsafe { SelectObject(dc, bitmap) };
    …
}
```

1. **Os límites.** GDI recibe o ancho e o alto como `i32`, e os bytes da memoria (4 por píxel) teñen que caber nun `isize`, o tamaño máximo dunha porción en Rust. Se non caben, devólvese `too_big` sen chamar a GDI.
2. **A descrición** (`BITMAPINFO`): 32 bits por píxel, sen compresión (`BI_RGB`). **O alto, negativo**: as filas van de arriba abaixo na memoria, como na xanela. Cun alto positivo, a primeira fila da memoria sería a de abaixo, unha herdanza dos formatos antigos.
3. **`CreateCompatibleDC(null)`**: un contexto de dispositivo en memoria, compatíbel coa pantalla. É o «sitio» desde o que copia `BitBlt`.
4. **`CreateDIBSection`** reserva a memoria dos píxeles e devolve o *bitmap* e, en `bits`, a dirección dos píxeles, que o programa pode escribir directamente. Sen arquivo de memoria compartida (`null`), a memoria é do proceso.
5. **`SelectObject`** pon o *bitmap* no contexto, e devolve o que había antes. Un contexto novo ten seleccionado un *bitmap* de 1 × 1, que se garda en `previous`.

Se algún paso falla, destrúese o que xa se creara e devólvese o erro, lido antes da limpeza para que non o cambie.

```rust
impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}
```

GDI non permite borrar un *bitmap* mentres está seleccionado nun contexto: primeiro selecciónase o anterior, despois bórrase o *bitmap* e por último o contexto.

### `WindowDc` e `Brush`

```rust
struct WindowDc(HDC, HWND);

impl WindowDc {
    fn get(window: HWND) -> Result<Self, Error> { … GetDC(window) … }
}

impl Drop for WindowDc {
    fn drop(&mut self) {
        unsafe { ReleaseDC(self.1, self.0) };
    }
}

struct Brush(HGDIOBJ);
```

Dous ***guards*** (gardas): tipos pequenos que posúen un recurso de Windows e o liberan no seu `Drop`. Así non se pode esquecer liberalo, nin sequera se a función remata antes por un erro.

- `WindowDc`: o contexto de dispositivo da xanela, que `GetDC` presta e `ReleaseDC` devolve.
- `Brush`: unha brocha dunha cor sólida, para as marcas, que `CreateSolidBrush` crea e `DeleteObject` borra.

### `colorref` e `with_room`

```rust
fn with_room(value: u32) -> u32 {
    value.saturating_add(value / 4)
}

const fn colorref(color: Rgb) -> COLORREF {
    color.r as u32 | (color.g as u32) << 8 | (color.b as u32) << 16
}
```

- `with_room`: un cuarto máis. `saturating_add` queda no máximo de `u32` en vez de desbordar.
- **`colorref`.** As brochas de GDI reciben as cores nun `COLORREF`, co vermello no byte máis baixo: `0x00BBGGRR`, o contrario ca os píxeles do *framebuffer*. `as u32` amplía cada canle de 8 a 32 bits, `<<` desprázaa 8 ou 16 bits á esquerda e `|` xúntaas. Estaba en `meiga-win` na subfase 1.1, para a brocha do fondo da xanela.

### Probas

Estas probas usan GDI de verdade, pero sen ningunha xanela: só *bitmaps* en memoria.

| Proba | Que comproba |
|---|---|
| `resize_with_room_keeps_the_memory_while_it_fits` | Con `Room`, a memoria medra un cuarto (de 420 × 300 a 525 × 375) e mantense mentres o tamaño cabe; con `Exact`, axústase ao tamaño. |
| `an_empty_size_still_has_memory` | Un tamaño de 0 × 0 ten 1 × 1 de memoria e un lenzo baleiro. |
| `the_canvas_draws_in_the_memory_that_gdi_copies` | O que debuxa o lenzo é o que copia `BitBlt`: pinta un rectángulo nun *framebuffer* e cópiao noutro, que o ten nos mesmos píxeles. Só se copia a parte do tamaño en uso. |
| `colorref_stores_blue_green_red` | A orde das canles de `COLORREF`. Trocar o vermello e o azul é un erro habitual que só se notaría nunha cor rara. |

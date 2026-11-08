# Código › `meiga-render`

> Estado: subfase 1.3. Contén o *framebuffer*, o lenzo co que se debuxa nel e a súa presentación na xanela con GDI, e o texto: as fontes con DirectWrite, a caché de glifos e a mestura co fondo.

**Responsabilidade:** o renderizado por CPU. Compón no *framebuffer* só o que cambiou e presenta na xanela só os rectángulos modificados. O texto debúxase desde unha caché de glifos, que DirectWrite rasteriza unha soa vez. Ver [Arquitectura › Fotogramas](../arquitectura.md#fotogramas-do-cambio-á-pantalla) e [Arquitectura › Texto](../arquitectura.md#texto-da-cela-aos-píxeles).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeiga-rendercargotoml) | As dependencias: `meiga-core`, `windows`, `windows-core` e `windows-sys`. |
| [`src/lib.rs`](#cratesmeiga-rendersrclibrs) | A raíz do crate: os módulos e o que se publica. |
| [`src/canvas.rs`](#cratesmeiga-rendersrccanvasrs) | `Canvas`, o lenzo: debuxar en píxeles en memoria, sen saír nunca deles; e `Shades`, os tons do texto. |
| [`src/error.rs`](#cratesmeiga-rendersrcerrorrs) | `Error`: unha chamada a GDI ou a DirectWrite que fallou, ou fontes que non están instaladas. |
| [`src/gdi.rs`](#cratesmeiga-rendersrcgdirs) | `Surface`, o *framebuffer* nunha *DIB section*, e a súa presentación con `BitBlt`. |
| [`src/dwrite.rs`](#cratesmeiga-rendersrcdwriters) | `FontSystem` e `Typeface`: as fontes con DirectWrite, o tamaño da cela e a cobertura de cada glifo. |
| [`src/glyphs.rs`](#cratesmeiga-rendersrcglyphsrs) | `GlyphCache`: a caché de glifos, en *slots* do tamaño dunha cela. |
| [`src/text.rs`](#cratesmeiga-rendersrctextrs) | `Font`: unha fonte coa súa caché, que debuxa filas de celas no lenzo. |

Que son as *DIB sections* e como funcionan `BitBlt` e `GdiFlush` explícase en [Tecnoloxías › DIB sections e GDI](../tecnoloxias.md#24-dib-sections-e-gdi-o-framebuffer-e-a-súa-presentación); DirectWrite, COM e a mestura do texto, en [Tecnoloxías › DirectWrite](../tecnoloxias.md#27-directwrite-fontes-e-glifos), [COM e o crate `windows`](../tecnoloxias.md#28-com-e-o-crate-windows) e [Mesturar o texto co fondo](../tecnoloxias.md#30-mesturar-o-texto-co-fondo-gamma-e-contraste).

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
# DirectWrite is a COM API, which windows-sys does not cover.
windows = { workspace = true, features = [
    "Win32_Foundation",
    "Win32_Globalization",
    "Win32_Graphics_DirectWrite",
] }
# The #[implement] macro, which implements a COM interface for the font
# fallback, generates code that names this crate.
windows-core.workspace = true
windows-sys = { workspace = true, features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
] }

[lints]
workspace = true
```

- A estrutura é a mesma que en [`meiga-core`](meiga-core.md#cratesmeiga-corecargotoml).
- `meiga-core.workspace = true`: depende de `meiga-core`, coa ruta que define o `Cargo.toml` raíz, para usar os seus tipos: as cores, os rectángulos e as celas do texto.
- `windows`, para DirectWrite, con tres *features*: os tipos básicos (`RECT`, `E_FAIL`), a configuración rexional do usuario (`GetUserDefaultLocaleName`) e DirectWrite. Que é e por que se usa explícase en [Tecnoloxías › COM e o crate `windows`](../tecnoloxias.md#28-com-e-o-crate-windows).
- `windows-core`: o núcleo do crate `windows`, que xa chega con el. Declárase porque o código que xera a macro `#[implement]` o nomea (`::windows_core::…`), e un crate só pode nomear as súas dependencias directas.
- `windows-sys`, coa mesma versión ca en `meiga-win`, e só dúas *features*: os tipos básicos (`HWND`, `RECT`, `COLORREF`) e GDI. Que é `windows-sys` explícase en [Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust).

## `crates/meiga-render/src/lib.rs`

```rust
//! CPU renderer of meiga.
//!
//! Composes in a framebuffer only what changed, and presents in the window
//! only the rectangles that changed. Text is drawn from a cache of glyphs,
//! which DirectWrite rasterizes once.
//!
//! The `unsafe` code of the crate is in `gdi.rs`, which calls GDI, and in
//! `dwrite.rs`, which calls DirectWrite. Every block explains why it is
//! sound in a `// SAFETY:` comment. The other modules forbid it.

mod canvas;
mod dwrite;
mod error;
mod gdi;
mod glyphs;
mod text;

pub use canvas::{Blending, Canvas, Shades, pixel};
pub use dwrite::FontSystem;
pub use error::Error;
pub use gdi::{Fit, Surface};
pub use text::{BLENDING, Font};
```

- Os seis módulos son privados (`mod`, sen `pub`), e a raíz publica só os tipos que necesitan os demais crates: `meiga_render::Surface`, e non `meiga_render::gdi::Surface`. A caché de glifos e `Typeface` non se publican: úsanse a través de [`Font`](#font-e-blending).
- **Onde está o `unsafe`.** Este crate chama a GDI e a DirectWrite, e iso require `unsafe`. Todo ese código está en `gdi.rs` e en `dwrite.rs`. Os outros catro módulos levan `#![forbid(unsafe_code)]`, así que o compilador garante que o lenzo, a caché de glifos e o texto, que son o código que máis vai medrar (o cursor, a selección…), non teñen ningún bloque inseguro.

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

**Encher un rectángulo** cunha cor: o fondo das celas do texto e os rectángulos da demo.

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

### `draw_coverage`

```rust
pub fn draw_coverage(&mut self, at: Point, coverage: &[u8], width: usize, ink: Rect, shades: &Shades) {
    let rows = coverage.len().checked_div(width).unwrap_or(0);
    let (Ok(columns), Ok(rows)) = (i32::try_from(width), i32::try_from(rows)) else {
        return;
    };
    if ink.is_empty() || !Rect::new(0, 0, columns, rows).contains(ink) {
        return;
    }
    let area = ink.offset(at).intersection(self.clip);
    if area.is_empty() {
        return;
    }
    let (from, to) = ((area.left - at.x) as usize, (area.right - at.x) as usize);
    for y in area.top..area.bottom {
        let source = (y - at.y) as usize * width;
        let target = y as usize * self.stride;
        let row = &coverage[source + from..source + to];
        let pixels = &mut self.pixels[target + area.left as usize..target + area.right as usize];
        for (pixel, &c) in pixels.iter_mut().zip(row) {
            if c != 0 {
                *pixel = shades.get(c);
            }
        }
    }
}
```

Debuxa a **tinta dun glifo**: os píxeles con cobertura dunha cela cuxa esquina superior esquerda está en `at`.

- `coverage` ten filas de `width` bytes, e só se debuxa o rectángulo `ink`, a parte con tinta, que garda a caché ([`GlyphCache`](#glyphcache)). Un glifo sen tinta, como o dun espazo, non debuxa nada.
- Primeiro compróbase que `ink` está dentro da cobertura; se non o está, non se debuxa nada, en vez de ler fóra dela. `checked_div` dá `None` se o ancho é 0.
- `ink.offset(at)` move o rectángulo á posición da cela, e recórtase ao `clip`. Como a zona que queda está dentro dos dous, as restas non son negativas, e os índices están dentro da cobertura e dos píxeles.
- **Os píxeles sen cobertura non se tocan**: teñen que ser xa o fondo da táboa, que [`Font::draw`](#font-e-blending) pinta antes. Os demais toman o seu ton da táboa: un acceso a memoria por píxel, sen ningunha conta.
- O bucle interno percorre dúas porcións da mesma lonxitude con `zip`: Rust comproba os límites ao crealas, unha vez por fila, e non en cada píxel.

### `Blending` e `Shades`

```rust
pub struct Blending {
    pub gamma: f32,
    pub contrast: f32,
}

pub struct Shades {
    pixels: [u32; 256],
}
```

- **`Blending`**: como se mestura o texto co fondo, co gamma e o contraste que se explican en [Tecnoloxías › Mesturar o texto co fondo](../tecnoloxias.md#30-mesturar-o-texto-co-fondo-gamma-e-contraste). Os valores de meiga son a constante [`BLENDING`](#font-e-blending).
- **`Shades`**: os 256 tons entre o fondo e a cor do texto, un por cada cobertura, como píxeles xa listos para o *framebuffer*. `get(coverage)` dá o dunha cobertura: un acceso a un *array* de 256 cun índice de 8 bits, que nunca sae del.

```rust
pub fn new(text: Rgb, background: Rgb, blending: Blending) -> Self {
    let gamma = if blending.gamma.is_finite() && blending.gamma > 0.0 { blending.gamma } else { 1.0 };
    let contrast = if blending.contrast.is_finite() {
        let lightness =
            (0.30 * f32::from(text.r) + 0.59 * f32::from(text.g) + 0.11 * f32::from(text.b)) / 255.0;
        blending.contrast.max(0.0) * (1.0 - lightness)
    } else {
        0.0
    };
    let light = |value: u8| (f32::from(value) / 255.0).powf(gamma);
    let channels = [
        (light(text.r), light(background.r)),
        (light(text.g), light(background.g)),
        (light(text.b), light(background.b)),
    ];
    let mut pixels = [0; 256];
    for (coverage, shade) in pixels.iter_mut().enumerate() {
        let a = coverage as f32 / 255.0;
        let a = a * (contrast + 1.0) / (a * contrast + 1.0);
        let [r, g, b] = channels.map(|(text, background)| {
            ((background + (text - background) * a).powf(1.0 / gamma) * 255.0).round() as u8
        });
        *shade = pixel(Rgb::new(r, g, b));
    }
    pixels[0] = pixel(background);
    pixels[255] = pixel(text);
    Self { pixels }
}
```

1. Os valores sen sentido (un gamma negativo, 0 ou `NaN`, ou un contraste `NaN`) mestúranse como lineais, en vez de dar cores estrañas. Un contraste negativo vale 0.
2. O contraste redúcese coa luminosidade da cor do texto: enteiro para o negro e nada para o branco.
3. `light` pasa cada canle das dúas cores ao espazo do gamma, unha soa vez.
4. Para cada cobertura, de 0 a 255, refórzaa co contraste, mestura as canles e volta ao espazo dos píxeles. `channels.map` aplica a mesma conta ás tres canles. O resultado está entre 0 e 1, así que `as u8` nin satura nin dá a volta.
5. Os dous extremos son exactamente as dúas cores, sen erros de redondeo: un píxel sen cobertura é o fondo, e un cuberto enteiro, o texto.

Calcular a táboa leva 768 potencias. Faise unha vez por par de cores, e [`Font`](#font-e-blending) garda as dos últimos 64 pares.

### Probas

| Proba | Que comproba |
|---|---|
| `pixels_are_red_green_blue_from_the_high_byte` | A orde das canles: `pixel` dá `0x00RRGGBB`. |
| `rows_must_fit_in_the_memory` | `Canvas::new` acepta as filas que caben, tamén cun `stride` maior có ancho, e rexeita as que non. |
| `fill_is_cut_to_the_canvas_and_respects_the_stride` | Un rectángulo que sae polos lados enche só a parte de dentro, e os píxeles que sobran ao final de cada fila non se tocan. |
| `clipped_canvases_only_draw_inside_the_clip` | Un lenzo recortado, e outro recortado sobre el, só pintan os píxeles da intersección. |
| `outline_draws_only_the_border` | `outline` pinta os 12 píxeles do bordo dun cadrado de 4 × 4, e nada dentro nin fóra. |
| `an_empty_canvas_draws_nothing` | O lenzo baleiro non pinta nada. |
| `shades_go_from_the_background_to_the_text` | Os extremos da táboa son as dúas cores e, cunha mestura lineal, a cobertura 51 (o 20 %) dá o 20 % do camiño entre o azul e o branco: `(51, 51, 255)`. |
| `gamma_lightens_the_edges_and_contrast_darkens_those_of_dark_text` | Cun gamma de 2, o ton do medio é máis claro ca na mestura lineal, tanto no texto branco sobre negro coma no negro sobre branco. O contraste non cambia o texto branco e escurece o negro. Os valores sen sentido mestúranse como lineais. |
| `coverage_paints_only_the_pixels_with_ink_inside_the_clip` | Unha cela de 3 × 2 con tres píxeles con cobertura: só eses cambian, co seu ton. Debuxada outra vez, recortada polo `clip` e polo bordo do lenzo, só cambia a parte de dentro. |
| `coverage_outside_its_rows_draws_nothing` | Un rectángulo de tinta que sae da cobertura, ou unha cobertura de ancho 0, non debuxan nada. |

## `crates/meiga-render/src/error.rs`

```rust
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

enum Kind {
    Call {
        function: &'static str,
        cause: Option<Cause>,
    },
    NoFont(String),
}

enum Cause {
    Os(io::Error),
    Com(windows_core::Error),
}
```

Algo que o renderizador non puido facer: unha chamada a GDI ou a DirectWrite que fallou, ou fontes que non están instaladas. Desde a subfase 1.3 garda tamén a causa dos erros dos métodos COM. `Kind` e `Cause` son privados: os outros crates só ven a mensaxe (`Display`) e que é un erro (`std::error::Error`), e por iso o tipo pode cambiar por dentro sen tocalos.

| Construtor | Cando | Mensaxe |
|---|---|---|
| `last(function)` | Unha función de GDI fallou. Le o código de erro do fío (`GetLastError`), así que hai que chamalo xusto despois. Moitas funcións de GDI fallan sen deixar ningún código, e entón non se mostra: «*The operation completed successfully*» enganaría. | `BitBlt failed: …` ou `BitBlt failed` |
| `too_big(function)` | A función nin sequera se chega a chamar, porque o que tería que reservar non cabe na memoria, por exemplo un *framebuffer* de máis de 2 000 millóns de píxeles. `io::ErrorKind::OutOfMemory.into()` crea un erro dese tipo. | `CreateDIBSection failed: out of memory` |
| `com(function, error)` | Un método COM devolveu un erro. `windows_core::Error` garda o `HRESULT` e a súa descrición. | `IDWriteFactory::GetSystemFontCollection failed: … (0x8007000E)` |
| `no_font(families)` | Ningunha das familias pedidas está instalada. | `None of these fonts is installed: Cascadia Mono, Consolas.` |

- Na parte das chamadas a Windows, é igual ca o [`Error` de `meiga-win`](meiga-win.md#cratesmeiga-winsrcerrorrs). Os dous crates teñen o seu propio tipo porque non dependen un do outro.
- `call(function, cause)`, privada, constrúe os erros das chamadas para os tres primeiros construtores.

**Proba:** `messages_name_the_function_and_the_reason` comproba tres mensaxes: a dun erro de COM, co nome do método ao principio e o `HRESULT` ao final (`(0x8007000E)`); a das fontes que non están; e a dunha chamada sen causa.

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

## `crates/meiga-render/src/dwrite.rs`

As fontes con DirectWrite: a fonte das celas, o tamaño dunha cela e a cobertura de cada glifo. Xunto con `gdi.rs`, é o único arquivo do crate con `unsafe`: os métodos COM precísano, aínda que o crate `windows` comprobe os `HRESULT` e leve a conta das referencias ([Tecnoloxías › COM e o crate `windows`](../tecnoloxias.md#28-com-e-o-crate-windows)). Os fragmentos son extractos, sen os comentarios `// SAFETY:` nin algúns `map_err`.

### `FontSystem`

```rust
#[derive(Clone)]
pub struct FontSystem {
    factory: IDWriteFactory2,
    collection: IDWriteFontCollection,
    fallback: IDWriteFontFallback,
    locale: Rc<[u16]>,
}
```

DirectWrite e as fontes do sistema, que comparten todas as fontes ([`Typeface`](#typeface)). Créase unha vez, porque ler a lista das fontes instaladas leva uns milisegundos.

- `factory`: a factoría compartida de DirectWrite, a do proceso, coas súas cachés (`DWRITE_FACTORY_TYPE_SHARED`). `IDWriteFactory2`, de Windows 8.1, é a que ten o *fallback* do sistema e o antialiasing en grises en `CreateGlyphRunAnalysis`.
- `collection`: as fontes instaladas.
- `fallback`: o *fallback* do sistema, que di que fonte instalada ten un carácter que lle falta á das celas.
- `locale`: a configuración rexional do usuario, como `gl-ES`, en UTF-16 e co nulo final. O *fallback* úsaa para escoller entre fontes: os ideogramas CJK debúxanse distinto en chinés e en xaponés. `Rc<[u16]>` é un texto compartido: as copias do `FontSystem` non o copian, só suman 1 ao seu contador.
- `#[derive(Clone)]`: clonalo fai `AddRef` nos tres obxectos COM. Cada `Typeface` garda a súa copia.

```rust
pub fn new() -> Result<Self, Error> {
    let factory: IDWriteFactory2 = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }
        .map_err(|error| Error::com("DWriteCreateFactory", error))?;
    let mut collection = None;
    unsafe { factory.GetSystemFontCollection(&mut collection, false) }
        .map_err(|error| Error::com("IDWriteFactory::GetSystemFontCollection", error))?;
    let collection = collection.ok_or_else(|| {
        Error::com("IDWriteFactory::GetSystemFontCollection", windows_core::Error::from_hresult(E_FAIL))
    })?;
    let fallback = unsafe { factory.GetSystemFontFallback() }
        .map_err(|error| Error::com("IDWriteFactory2::GetSystemFontFallback", error))?;
    Ok(Self { factory, collection, fallback, locale: user_locale() })
}
```

- `DWriteCreateFactory` é xenérica no tipo da interface que devolve: `let factory: IDWriteFactory2` fai que pida esa versión.
- `GetSystemFontCollection` devolve a colección nun parámetro de saída, un `Option`. `false`: non mira se se instalaron fontes mentres corre o programa. Se a colección volvese baleira sen ningún erro, que non debería pasar, o erro é `E_FAIL`.
- Cada erro leva o nome do método, para saber cal fallou.

```rust
fn user_locale() -> Rc<[u16]> {
    let mut name = [0; LOCALE_NAME_MAX_LENGTH];
    let len = unsafe { GetUserDefaultLocaleName(&mut name) };
    match usize::try_from(len) {
        Ok(len @ 2..=LOCALE_NAME_MAX_LENGTH) => name[..len].into(),
        _ => "en-US\0".encode_utf16().collect(),
    }
}
```

- `GetUserDefaultLocaleName` escribe o nome nun búfer do tamaño máximo, 85 caracteres (`LOCALE_NAME_MAX_LENGTH`), e devolve a súa lonxitude co nulo, ou 0 se falla.
- `len @ 2..=…`: o patrón comproba que a lonxitude está no rango e dálle o nome `len`. Polo menos 2: un carácter e o nulo.
- Se falla, `en-US`.

### `Typeface`

```rust
pub struct Typeface {
    system: FontSystem,
    family: String,
    family_name: HSTRING,
    face: IDWriteFontFace,
    em: f32,
    cell: Size,
    baseline: f32,
    texture: Vec<u8>,
}
```

Unha fonte para as celas dunha grella, a un tamaño e a un DPI: o tamaño da cela, onde está a liña base e a cobertura dos seus glifos.

| Campo | Que garda |
|---|---|
| `system` | A súa copia do [`FontSystem`](#fontsystem). |
| `family` e `family_name` | A familia que se atopou, como texto de Rust e como `HSTRING` (UTF-16, para o *fallback*). |
| `face` | A cara da fonte, co peso, o ancho e o estilo normais. |
| `em` | O tamaño do em, en píxeles. |
| `cell` | O tamaño dunha cela, en píxeles. |
| `baseline` | A distancia desde o alto da cela ata a liña base, en píxeles. |
| `texture` | A textura do último glifo, que se garda para rasterizar o seguinte sen reservar memoria. |

```rust
pub fn new(system: &FontSystem, families: &[&str], points: f32, dpi: u32) -> Result<Self, Error> {
    let Some((family, face)) = families.iter().find_map(|&name| find(system, name)) else {
        return Err(Error::no_font(families));
    };
    let em = points * dpi as f32 / 72.0;
    let metrics = font_metrics(&face);
    let units = em / f32::from(metrics.designUnitsPerEm.max(1));
    let ascent = f32::from(metrics.ascent) * units;
    let descent = f32::from(metrics.descent) * units;
    let half_gap = f32::from(metrics.lineGap) * units / 2.0;
    let baseline = (ascent + half_gap).round();
    let height = (baseline + descent + half_gap).round();
    let advance = glyph_index(&face, '0')
        .and_then(|glyph| advance(&face, glyph, em))
        .unwrap_or(em / 2.0);
    let cell = Size::new(advance.round().max(1.0) as u32, height.max(1.0) as u32);
    Ok(Self { … })
}
```

1. A primeira familia da lista que estea instalada: `find_map` percorre a lista ata que `find` dá algo. Se non hai ningunha, o erro `NoFont`.
2. O em en píxeles: `puntos × DPI / 72` ([Tecnoloxías › DirectWrite](../tecnoloxias.md#27-directwrite-fontes-e-glifos)).
3. As métricas pásanse de unidades de deseño a píxeles. `max(1)` evita dividir entre 0 cunha fonte estragada.
4. A liña base redondéase primeiro, e o alto despois: así o descenso cabe enteiro debaixo dela.
5. O ancho é o avance do `0`. Se a fonte non o tivese, a metade do em.
6. A cela mide polo menos 1 × 1. Os valores son números positivos pequenos, así que as conversións a `u32` son exactas.

Cascadia Mono a 12 puntos dá celas de 9 × 19 píxeles ao 100 %, de 12 × 24 ao 125 % e de 14 × 28 ao 150 %.

```rust
pub fn rasterize(&mut self, c: char, columns: u32, coverage: &mut [u8]) -> Result<(), Error> {
    let width = self.cell.width.saturating_mul(columns);
    let size = Size::new(width, self.cell.height);
    if coverage.len() < size.area() as usize {
        return Ok(());
    }
    let (face, glyph, em) = self.face_for(c);
    let left = advance(&face, glyph, em)
        .map(|advance| ((width as f32 - advance) / 2.0).round())
        .unwrap_or(0.0);
    let (mode, grid_fit) = rendering();
    let no_advance = 0.0_f32;
    let run = DWRITE_GLYPH_RUN {
        fontFace: ManuallyDrop::new(Some(face)),
        fontEmSize: em,
        glyphCount: 1,
        glyphIndices: &glyph,
        glyphAdvances: &no_advance,
        glyphOffsets: ptr::null(),
        isSideways: BOOL(0),
        bidiLevel: 0,
    };
    let analysis = unsafe {
        self.system.factory.CreateGlyphRunAnalysis(
            &run,
            None,
            mode,
            DWRITE_MEASURING_MODE_NATURAL,
            grid_fit,
            DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
            left,
            self.baseline,
        )
    };
    drop(ManuallyDrop::into_inner(run.fontFace));
    let analysis = analysis.map_err(…)?;
    …
}
```

**Rasteriza o carácter `c`** nunha caixa de `columns` celas, 1 ou 2, e escribe a súa cobertura en `coverage`: filas do ancho da caixa, tantas coma o alto dunha cela, que teñen que chegar a cero.

1. **O glifo.** `face_for` dá a cara, o glifo e o tamaño do em: os da fonte das celas, os da fonte que escolle o *fallback* ou o glifo 0, o dos caracteres que faltan.
2. **Centrado.** Se o glifo avanza un ancho distinto do da caixa, por exemplo un ideograma dunha fonte con outras proporcións, móvese á metade da diferenza.
3. **O *glyph run*:** un só glifo, sen avance (`no_advance`) nin desprazamento. A estrutura garda a cara nun `ManuallyDrop` ([Tecnoloxías › COM](../tecnoloxias.md#28-com-e-o-crate-windows)): `ManuallyDrop::new(Some(face))` dálle a referencia, e `drop(ManuallyDrop::into_inner(…))` libéraa despois da chamada, falle ou non. Os punteiros a `glyph` e a `no_advance` valen durante a chamada, porque son variábeis locais que viven ata o final da función.
4. **A análise:** `CreateGlyphRunAnalysis`, coas opcións de [Tecnoloxías › DirectWrite](../tecnoloxias.md#27-directwrite-fontes-e-glifos), e sen transformación (`None`), porque o tamaño do em xa está en píxeles. A orixe da liña base é (`left`, `baseline`), dentro da caixa.

```rust
    let bounds = unsafe { analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1) }.map_err(…)?;
    let bounds = Rect::new(bounds.left, bounds.top, bounds.right, bounds.bottom);
    let visible = bounds.intersection(Rect::from_size(size));
    if visible.is_empty() {
        return Ok(());
    }
    self.texture.clear();
    self.texture.resize(bounds.area() as usize, 0);
    …
    unsafe { analysis.CreateAlphaTexture(DWRITE_TEXTURE_ALIASED_1x1, &texture_bounds, &mut self.texture) }
        .map_err(…)?;
    let texture_width = bounds.width() as usize;
    let (from, to) = ((visible.left - bounds.left) as usize, (visible.right - bounds.left) as usize);
    for y in visible.top..visible.bottom {
        let source = (y - bounds.top) as usize * texture_width;
        let target = y as usize * width as usize + visible.left as usize;
        coverage[target..target + (to - from)].copy_from_slice(&self.texture[source + from..source + to]);
    }
    Ok(())
```

5. **A textura.** `GetAlphaTextureBounds` di que rectángulo ocupa a cobertura, en coordenadas da caixa. Pode saír dela: o bloque `█` sobe e baixa un pouco máis ca a cela. Só se copia a parte de dentro (`visible`). Se non hai ningunha, como nun espazo, a caixa queda a cero.
6. `CreateAlphaTexture` escribe a cobertura dese rectángulo enteiro no búfer reutilizado, que ten exactamente o tamaño que pide: un byte por cada píxel do rectángulo.
7. **A copia.** Fila a fila, a parte visíbel pasa da textura á caixa. As dúas porcións teñen a mesma lonxitude, `to - from`, e `copy_from_slice` cópiaas dunha vez.

- Se a caixa non cabe en `coverage`, non fai nada: a cela queda baleira, en vez de escribir fóra.
- Os erros de DirectWrite devólvense, e quen chama decide que facer: [`Font`](#font-e-blending) deixa a cela co fondo.

```rust
fn face_for(&self, c: char) -> (IDWriteFontFace, u16, f32) {
    if let Some(glyph) = glyph_index(&self.face, c) {
        return (self.face.clone(), glyph, self.em);
    }
    self.fallback(c).unwrap_or_else(|| (self.face.clone(), 0, self.em))
}
```

A cara, o glifo e o tamaño do em con que debuxar `c`: os da fonte das celas, se o ten; se non, os do *fallback*; e se ningunha fonte o ten, o glifo 0 da fonte das celas, o dos caracteres que faltan. `clone()` fai `AddRef`.

```rust
fn fallback(&self, c: char) -> Option<(IDWriteFontFace, u16, f32)> {
    let mut units = [0; 2];
    let text = c.encode_utf16(&mut units);
    let len = text.len() as u32;
    let source: IDWriteTextAnalysisSource = OneChar { text: units, len, locale: self.system.locale.to_vec() }.into();
    let (mut mapped_len, mut font, mut scale) = (0, None, 1.0);
    unsafe {
        self.system.fallback.MapCharacters(
            &source,
            0,
            len,
            &self.system.collection,
            &self.family_name,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            &mut mapped_len,
            &mut font,
            &mut scale,
        )
    }
    .ok()?;
    let face = unsafe { font?.CreateFontFace() }.ok()?;
    let glyph = glyph_index(&face, c)?;
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    Some((face, glyph, self.em * scale))
}
```

**O *fallback***: a fonte que escolle o sistema para `c`, se algunha o ten.

- O carácter pásase a UTF-16, a codificación de Windows: unha unidade, ou dúas (un par subrogado) para os que están fóra do plano básico, como os emoji.
- `OneChar { … }.into()` crea o obxecto COM: `#[implement]` xera a conversión da estrutura de Rust á súa interface, nun bloque de memoria con conta de referencias.
- `MapCharacters` recibe o texto, a colección onde buscar, a familia preferida e o peso, o estilo e o ancho. Devolve cantos caracteres cobre a fonte escollida (`mapped_len`), a fonte (`font`, ou `None` se ningunha os ten) e unha escala (`scale`) para igualar o tamaño aparente das dúas fontes.
- `.ok()?` converte cada erro en `None`: se o *fallback* falla, debúxase o glifo dos caracteres que faltan.
- A escala só se aplica se é un número positivo.
- Cada carácter que lle falta á fonte pasa por aquí unha vez por tamaño de fonte, a primeira vez que se debuxa: despois, o seu glifo xa está na caché.

### Funcións auxiliares

| Función | Que fai |
|---|---|
| `rendering()` | As opcións da rasterización: o modo `NATURAL_SYMMETRIC` e o axuste á grella activado. |
| `find(system, name)` | A familia `name`, se está instalada, e a súa cara normal: `FindFamilyName`, `GetFontFamily`, `GetFirstMatchingFont` e `CreateFontFace`. |
| `font_metrics(face)` | As métricas da fonte (`GetMetrics`). |
| `glyph_index(face, c)` | O glifo de `c` na cara, ou `None` se a cara non o ten: o glifo 0. |
| `advance(face, glyph, em)` | Canto avanza un glifo, en píxeles. `GetDesignGlyphMetrics` dáo en unidades de deseño. |

### `OneChar`: o texto para o *fallback*

```rust
#[implement(IDWriteTextAnalysisSource)]
struct OneChar {
    text: [u16; 2],
    len: u32,
    locale: Vec<u16>,
}

impl IDWriteTextAnalysisSource_Impl for OneChar_Impl {
    fn GetTextAtPosition(&self, position: u32, text: *mut *mut u16, len: *mut u32) -> windows_core::Result<()> {
        let rest = self.len.saturating_sub(position);
        unsafe {
            *text = if rest == 0 {
                ptr::null_mut()
            } else {
                self.text.as_ptr().add(position as usize).cast_mut()
            };
            *len = rest;
        }
        Ok(())
    }
    …
}
```

O texto dun carácter, que o *fallback* le a través da interface COM `IDWriteTextAnalysisSource`. `#[implement]` xera o tipo `OneChar_Impl`, que envolve a estrutura coa conta de referencias, e o programa escribe os cinco métodos da interface:

| Método | Que responde |
|---|---|
| `GetTextAtPosition` | O texto desde `position` ata o final: un punteiro e a súa lonxitude. Ao final, un punteiro nulo e 0. |
| `GetTextBeforePosition` | O texto antes de `position`, desde o principio. |
| `GetParagraphReadingDirection` | De esquerda a dereita. |
| `GetLocaleName` | A configuración rexional, co seu nulo, para todo o texto desde `position`. |
| `GetNumberSubstitution` | Ningunha substitución: os díxitos debúxanse como son. |

- **Os punteiros.** DirectWrite recibe punteiros ao texto e á configuración rexional, que son parte do obxecto. Valen mentres o obxecto exista, e DirectWrite tenno, coa súa conta de referencias, mentres le o texto. Só os le: a ligazón de Rust declara o punteiro como `*mut` porque así traduce o `WCHAR const**` de C, e `cast_mut()` faino explícito.
- `saturating_sub`: se DirectWrite pedise unha posición despois do final, a lonxitude sería 0, en vez de desbordar.
- `GetNumberSubstitution` recibe un `OutRef`, a forma do crate `windows` de escribir unha interface nun parámetro de saída, e `write(None)` escribe un punteiro nulo.

### Probas

Usan as fontes instaladas de verdade, pero sen ningunha xanela: Cascadia Mono ou, se non está, Consolas.

| Proba | Que comproba |
|---|---|
| `the_cell_grows_with_the_dpi` | A 96, 120 e 144 DPI, a cela medra en alto, e en ancho medra ou queda igual. Sempre é máis alta ca ancha. |
| `a_missing_font_is_an_error` | Unha familia que non existe dá o erro `None of these fonts is installed: …`, e a fonte atopada é unha das pedidas. |
| `letters_have_grayscale_ink_inside_their_cell` | A `M` ten píxeles con cobertura 255 e outros intermedios (o antialiasing en grises), e tinta en polo menos a metade das columnas da cela; a `\|`, como moito en tres; e o espazo, en ningunha. |
| `descenders_go_below_the_baseline_and_stay_in_the_cell` | A 144 DPI, a `x` non baixa da liña base, e a `g` baixa máis dun píxel por debaixo dela, sen saír da cela. |
| `wide_characters_fill_both_cells_with_the_fallback_font` | A fonte das celas non ten `中`, e o carácter ten tinta nas dúas celas: o ideograma da fonte que escolle o *fallback* ou, nun Windows sen fontes CJK, o glifo dos caracteres que faltan, centrado. |
| `a_character_that_no_font_has_draws_the_missing_glyph` | U+FDD0, un «non carácter» que ningunha fonte ten, debuxa o glifo dos caracteres que faltan. |

## `crates/meiga-render/src/glyphs.rs`

A caché de glifos: a cobertura de cada glifo, rasterizada unha vez, en *slots* do tamaño dunha cela ([ADR 0010](../adr/0010-texto-con-directwrite.md)).

### `GlyphCache`

```rust
pub const CAPACITY: usize = 4096;

pub struct Glyph<'a> {
    pub coverage: &'a [u8],
    pub ink: Rect,
}

pub struct SlotId(u32);

pub struct GlyphCache {
    cell: Size,
    slot_len: usize,
    coverage: Vec<u8>,
    slots: Vec<Slot>,
    index: HashMap<u32, u32, BuildHasherDefault<KeyHasher>>,
    capacity: usize,
    hand: usize,
}

struct Slot {
    key: u32,
    ink: Rect,
    used: bool,
}
```

| Campo | Que garda |
|---|---|
| `cell` e `slot_len` | O tamaño dunha cela e os bytes dun *slot*: un por píxel. |
| `coverage` | A cobertura de todos os *slots*, un tras outro, nun só bloque de memoria. |
| `slots` | De cada *slot*: a clave do glifo que ten, o rectángulo con tinta e a marca do reloxo. |
| `index` | O *slot* de cada glifo, pola súa clave. |
| `capacity` | Cantos *slots* pode ter como moito: `CAPACITY`, salvo nas probas, e polo menos 2, as dúas metades dun carácter ancho. |
| `hand` | A agulla do reloxo: o seguinte *slot* que mira cando a caché está chea. |

- **`Glyph`** é o que se debuxa dun glifo: a cobertura do seu *slot* e o seu rectángulo con tinta.
- **`SlotId`** é o número dun *slot*. Vale ata a seguinte inserción, que pode darlle o *slot* a outro glifo: por iso [`Font`](#font-e-blending) o usa no momento.
- **A memoria** medra con cada glifo novo, ata `CAPACITY` *slots*: con celas de 14 × 28 píxeles (150 %), 1,6 MB como moito. Unha pantalla dun terminal usa uns poucos centos de glifos distintos, moitos menos ca celas.

```rust
pub fn find(&mut self, glyph: CellChar) -> Option<SlotId> {
    let slot = *self.index.get(&key(glyph))?;
    self.slots[slot as usize].used = true;
    Some(SlotId(slot))
}
```

**Buscar** un glifo é unha consulta na táboa *hash* e marcar o seu *slot* como usado. É o que se fai, en cada fotograma, en cada cela con texto que cambiou.

```rust
pub fn insert(&mut self, glyph: CellChar, rasterize: impl FnOnce(&mut [u8])) -> SlotId {
    let key = key(glyph);
    let slot = match self.index.get(&key) {
        Some(&slot) => slot as usize,
        None => self.free_slot(),
    };
    let coverage = &mut self.coverage[slot * self.slot_len..][..self.slot_len];
    coverage.fill(0);
    rasterize(coverage);
    self.slots[slot] = Slot { key, ink: ink(coverage, self.cell.width as usize), used: true };
    self.index.insert(key, slot as u32);
    SlotId(slot as u32)
}
```

**Engadir** un glifo:

1. Se xa estaba, reutiliza o seu *slot*; se non, colle un libre (`free_slot`).
2. Pon o *slot* a cero e pásallo a `rasterize`, unha *closure* que escribe a cobertura. Así, a caché non sabe nada de DirectWrite, e as probas usan *closures* que pintan un píxel. `[slot * self.slot_len..][..self.slot_len]` é a porción do *slot*: desde o seu comezo, `slot_len` bytes.
3. Calcula o rectángulo con tinta (`ink`): o máis pequeno que contén todos os píxeles con cobertura. O dun glifo sen tinta está baleiro, e non se debuxa nada.
4. Garda a clave no índice.

```rust
fn free_slot(&mut self) -> usize {
    if self.slots.len() < self.capacity {
        self.slots.push(Slot { key: 0, ink: Rect::EMPTY, used: false });
        self.coverage.resize(self.coverage.len() + self.slot_len, 0);
        return self.slots.len() - 1;
    }
    loop {
        let slot = self.hand;
        self.hand = (slot + 1) % self.slots.len();
        if !mem::take(&mut self.slots[slot].used) {
            self.index.remove(&self.slots[slot].key);
            return slot;
        }
    }
}
```

Un *slot* para un glifo novo:

- Mentres hai sitio, un *slot* novo ao final. `Vec` reserva a memoria en bloques cada vez máis grandes, así que medrar custa pouco de media.
- Coa caché chea, o **algoritmo do reloxo** (*clock*): a agulla percorre os *slots* en círculo. Se un ten a marca de usado, quítalla e segue: é a súa «segunda oportunidade». `mem::take` le a marca e deixa `false` no seu sitio. O primeiro *slot* sen marca é o elixido, e o seu glifo sae do índice.
- Remata como moito en dúas voltas, porque a primeira quita todas as marcas.
- É o algoritmo clásico de substitución de páxinas da memoria virtual: unha aproximación barata a substituír o glifo usado hai máis tempo (LRU), sen listas que actualizar cada vez que se usa un ([Wikipedia](https://en.wikipedia.org/wiki/Page_replacement_algorithm#Clock)).

```rust
fn key(glyph: CellChar) -> u32 {
    match glyph {
        CellChar::Blank => 0,
        CellChar::Narrow(c) => u32::from(c) << 2 | 1,
        CellChar::WideLeft(c) => u32::from(c) << 2 | 2,
        CellChar::WideRight(c) => u32::from(c) << 2 | 3,
    }
}
```

**A clave** dun glifo: o carácter e, nos dous bits baixos, que parte é: enteiro, a metade esquerda ou a dereita. Os caracteres chegan ata U+10FFFF, que cabe en 21 bits, así que a clave cabe en 23. Unha cela en branco nunca chega á caché.

```rust
struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    …
}
```

**O *hash* do índice**: unha rotación, un «ou» exclusivo e unha multiplicación, como o *Fx hash* do compilador de Rust. As claves son números pequenos que ninguén de fóra de meiga escolle, así que non precisan a protección contra colisións provocadas do *hash* estándar, SipHash, nin o seu custo. `BuildHasherDefault<KeyHasher>` dille ao `HashMap` que cree un `KeyHasher` novo, a cero, para cada clave.

### Probas

| Proba | Que comproba |
|---|---|
| `a_glyph_is_rasterized_once_and_found_after` | Un glifo que non está non se atopa. Ao engadilo, a *closure* recibe un *slot* a cero, e despois atópase, coa súa cobertura e o seu rectángulo con tinta. |
| `a_glyph_without_ink_has_an_empty_ink_box` | Un glifo sen cobertura ten o rectángulo con tinta baleiro. |
| `each_half_of_a_wide_character_has_its_slot` | As dúas metades dun carácter ancho teñen *slots* distintos, e o carácter enteiro non atopa ningunha. |
| `inserting_again_reuses_the_slot` | Engadir outra vez un glifo usa o mesmo *slot*. |
| `when_it_is_full_a_glyph_used_since_the_last_round_stays` | Nunha caché de 3, o cuarto glifo colle o *slot* do primeiro; e se o segundo se usa de novo, o quinto colle o do terceiro. |
| `a_reused_slot_is_cleared_first` | Un *slot* reutilizado chega a cero á *closure*, sen a cobertura do glifo anterior. |

## `crates/meiga-render/src/text.rs`

O texto en celas: os glifos dunha fonte, rasterizados a primeira vez que se debuxan, e debuxados no lenzo coas cores de cada cela.

### `Font` e `BLENDING`

```rust
pub const BLENDING: Blending = Blending { gamma: 1.5, contrast: 0.5 };
const SHADES: usize = 64;

pub struct Font {
    typeface: Typeface,
    glyphs: GlyphCache,
    wide: Vec<u8>,
    shades: Vec<(Rgb, Rgb, Box<Shades>)>,
    blending: Blending,
}
```

Unha fonte para as celas dunha grella, a un tamaño e a un DPI, coa súa caché de glifos. É o que usa o resto de meiga para debuxar texto.

- **`BLENDING`**: o gamma e o contraste do texto, ata que a subfase 8.3 os faga configurábeis. Con eles, unha liña de Cascadia Mono ao 150 % ten a mesma tinta ca no Windows Terminal, cunha diferenza menor do 0,5 % ([Tecnoloxías › Mesturar o texto co fondo](../tecnoloxias.md#30-mesturar-o-texto-co-fondo-gamma-e-contraste)).
- `wide`: un búfer para as dúas celas dun carácter ancho, que se rasterizan xuntas. Reutilízase.
- `shades`: as táboas de tons dos últimos 64 pares de cores (`SHADES`), a máis recente ao final. Unha pantalla dun terminal usa poucos pares; se hai máis, o máis antigo calcúlase de novo cando volta. Cada táboa ocupa 1 KB, nunha `Box`, e o vector só move punteiros.
- `new`, `family`, `cell` e `cached_glyphs` crean a fonte, e dan a familia atopada, o tamaño da cela e cantos glifos hai na caché.

```rust
pub fn draw(&mut self, canvas: &mut Canvas<'_>, at: Point, cells: &[CellChar], text: Rgb, background: Rgb) {
    let cell = self.cell();
    let (Ok(count), Ok(width), Ok(height)) =
        (i32::try_from(cells.len()), i32::try_from(cell.width), i32::try_from(cell.height))
    else {
        return;
    };
    let row = Rect::new(
        at.x,
        at.y,
        at.x.saturating_add(width.saturating_mul(count)),
        at.y.saturating_add(height),
    );
    let visible = row.intersection(canvas.clip());
    if visible.is_empty() || width == 0 {
        return;
    }
    canvas.fill(visible, background);
    let first = ((visible.left - at.x) / width) as usize;
    let last = ((visible.right - 1 - at.x) / width) as usize;
    let Self { typeface, glyphs, wide, shades, blending } = self;
    let shades = shades_for(shades, text, background, *blending);
    for (index, &c) in cells.iter().enumerate().take(last + 1).skip(first) {
        let Some(slot) = slot(glyphs, typeface, wide, c) else {
            continue;
        };
        let glyph = glyphs.glyph(slot);
        let x = at.x.saturating_add(width.saturating_mul(index as i32));
        canvas.draw_coverage(Point::new(x, at.y), glyph.coverage, cell.width as usize, glyph.ink, shades);
    }
}
```

**Debuxa `cells` nunha fila**, desde `at`, a esquina superior esquerda da primeira, coas cores `text` e `background`:

1. **A parte visíbel:** o rectángulo da fila, recortado polo `clip` do lenzo. Se non se ve nada, non fai nada máis: nin sequera busca os glifos.
2. **O fondo** de toda a parte visíbel, dunha vez.
3. **As celas que toca a parte visíbel**, da primeira (`first`) á última (`last`): só esas buscan o seu glifo e debuxan a súa tinta. Así, un fotograma que cambia unha cela non toca as demais da fila, e un glifo que non se ve nin se rasteriza.
4. **A táboa de tons** do par de cores, unha vez para toda a fila.
5. **Cada cela:** o seu *slot* na caché (`slot`, que rasteriza o glifo a primeira vez) e a súa tinta (`draw_coverage`). Unha cela en branco só ten o fondo.

- **Os préstamos.** `let Self { typeface, glyphs, wide, shades, blending } = self;` separa os campos en referencias distintas: a táboa de tons segue prestada mentres se modifican a caché e a fonte, e Rust permíteo porque son campos distintos.
- As contas das posicións son `saturating_*`: unha fila moi longa ou moi á dereita queda no máximo dun `i32`, en vez de desbordar. As divisións non dan números negativos, porque a parte visíbel está dentro da fila.
- Se DirectWrite non pode rasterizar un glifo, a cela queda co fondo e o debuxo continúa.

```rust
fn slot(glyphs: &mut GlyphCache, typeface: &mut Typeface, wide: &mut Vec<u8>, c: CellChar) -> Option<SlotId> {
    if c == CellChar::Blank {
        return None;
    }
    if let Some(slot) = glyphs.find(c) {
        return Some(slot);
    }
    match c {
        CellChar::Blank => None,
        CellChar::Narrow(character) => Some(glyphs.insert(c, |coverage| {
            let _ = typeface.rasterize(character, 1, coverage);
        })),
        CellChar::WideLeft(character) | CellChar::WideRight(character) => {
            let cell = glyphs.cell();
            let width = cell.width as usize;
            wide.clear();
            wide.resize(2 * cell.area() as usize, 0);
            let _ = typeface.rasterize(character, 2, wide);
            let wide = &*wide;
            let half = |right: bool| {
                move |coverage: &mut [u8]| {
                    let rows = wide.chunks_exact(2 * width);
                    for (row, out) in rows.zip(coverage.chunks_exact_mut(width)) {
                        out.copy_from_slice(&row[usize::from(right) * width..][..width]);
                    }
                }
            };
            let right = matches!(c, CellChar::WideRight(_));
            let other = if right { CellChar::WideLeft(character) } else { CellChar::WideRight(character) };
            glyphs.insert(other, half(!right));
            Some(glyphs.insert(c, half(right)))
        }
    }
}
```

**O *slot* do glifo dunha cela:**

- Unha cela en branco non ten glifo: `None`.
- Se está na caché, o seu *slot*.
- Un carácter estreito rasterízase directamente no seu *slot*. Se falla, o *slot* queda a cero e a cela vese co fondo, ata que a caché o substitúa. `let _ =` descarta o erro de forma explícita.
- **Un carácter ancho** rasterízase unha soa vez nas dúas celas, no búfer `wide`, e gárdanse as dúas metades: a *closure* que dá `half` copia a metade esquerda ou a dereita de cada fila ao *slot*. Gárdase primeiro a outra metade, a que non se pediu, porque o *slot* que vale é o da última inserción ([`SlotId`](#glyphcache)). A outra metade adoita debuxarse xusto despois, e xa estará na caché.

```rust
fn shades_for(cache: &mut Vec<(Rgb, Rgb, Box<Shades>)>, text: Rgb, background: Rgb, blending: Blending) -> &Shades {
    let index = match cache.iter().rposition(|&(t, b, _)| t == text && b == background) {
        Some(index) => index,
        None => {
            if cache.len() == SHADES {
                cache.remove(0);
            }
            cache.push((text, background, Box::new(Shades::new(text, background, blending))));
            cache.len() - 1
        }
    };
    &cache[index].2
}
```

A táboa de tons dun par de cores: búscaa desde o final, onde están as máis recentes (`rposition`), e calcúlaa se non está. Se xa hai 64, sae a máis antiga.

### Probas

Usan as fontes do sistema, como as de `dwrite.rs`.

| Proba | Que comproba |
|---|---|
| `a_row_paints_its_background_and_the_ink_of_its_glyphs` | Unha fila de tres celas (`a b`) nun lenzo de catro: a cela en branco só ten fondo, as outras dúas teñen píxeles da cor do texto, a cuarta non se toca, e a caché ten dous glifos. |
| `only_the_cells_inside_the_clip_are_drawn_and_rasterized` | Cun recorte que colle o final da terceira cela e a cuarta, só cambian os píxeles do recorte, e só se rasterizan dous glifos. |
| `a_wide_character_is_rasterized_once_for_both_cells` | Debuxando só a metade dereita de `中`, as dúas metades quedan na caché; debuxándoo enteiro despois, non se engade ningunha, e hai tinta nas dúas celas. |
| `the_shades_of_the_oldest_pair_are_dropped_first` | Con 65 pares de cores, quedan 64 e sae o primeiro; un par que xa estaba dá a súa táboa sen engadir ningunha. |

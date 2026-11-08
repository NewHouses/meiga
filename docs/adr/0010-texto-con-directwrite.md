# ADR 0010 · Texto con DirectWrite e unha caché de glifos do tamaño da cela

- **Estado:** Aceptado
- **Data:** 2026-11-08
- **Relacionados:** [ADR 0002](0002-windows-e-renderizado-cpu.md), [ADR 0003](0003-nucleo-vt-alacritty-terminal.md), [ADR 0006](0006-licenzas-e-dependencias.md)

## Contexto

- O [ADR 0002](0002-windows-e-renderizado-cpu.md) decidiu que os glifos se rasterizan unha soa vez con DirectWrite e se gardan nunha caché, e que cada fotograma compón no *framebuffer* só as celas que cambiaron. Ao implementalo, na subfase 1.3, quedan por decidir: como chamar a DirectWrite desde Rust, en que formato se garda cada glifo, como encaixa nunha cela, como se organiza a caché, que pasa cos caracteres que a fonte non ten e como se mestura o texto co fondo.
- **DirectWrite é unha API COM.** As súas funcións son métodos de interfaces (`IDWriteFactory`, `IDWriteFontFace`…) con conta de referencias, e algunhas piden que o programa implemente unha interface: o *fallback* de fontes le o texto a través de `IDWriteTextAnalysisSource`. `windows-sys`, que meiga usa para Win32 e GDI, só declara as funcións. O crate `windows`, tamén de Microsoft, envolve as interfaces COM e permite implementalas coa macro `#[implement]` ([Tecnoloxías › COM](../tecnoloxias.md#28-com-e-o-crate-windows)).
- **Rasterizar sen dispositivo gráfico.** `IDWriteFactory2::CreateGlyphRunAnalysis` calcula a cobertura dun glifo, canto tapa de cada píxel, e devólvea nunha textura en memoria. Non usa Direct2D nin Direct3D: todo é CPU, sen pasar polo rasterizador WARP.
- **Grises ou ClearType.** O antialiasing ClearType calcula tres coberturas por píxel, unha por subpíxel (vermello, verde e azul). Depende de como estean dispostos os subpíxeles do monitor, que por Escritorio remoto é o do outro equipo, deixa bordos de cor e obriga a mesturar cada canle por separado. O antialiasing en grises calcula unha cobertura por píxel. O Windows Terminal usa grises por defecto.
- **A grella.** Nun terminal, cada carácter ocupa unha cela, ou dúas se é ancho, como a maioría dos ideogramas CJK e dos emoji. Os glifos dunha fonte monoespazada caben case todos na súa cela; algúns saen un pouco, como o bloque `█`.
- **Os caracteres que a fonte non ten.** Cascadia Mono non ten ideogramas CJK nin moitos símbolos. O *fallback* de fontes do sistema (`IDWriteFontFallback::MapCharacters`) di que fonte instalada ten cada un. A primeira consulta dun proceso carga os datos do *fallback* e leva uns milisegundos; as seguintes, microsegundos.
- **Mesturar o texto co fondo.** Nun píxel cuberto á metade, a cor pode ser a media exacta das dúas (mestura lineal) ou outra máis próxima a como percibe o ollo a luz. DirectWrite e Direct2D mesturan con corrección de gamma e cun contraste extra para o texto escuro, así que o resultado depende da cor do texto, como explica o propio código do Windows Terminal. Coa mestura lineal, o texto claro sobre fondo escuro vese máis fino ca no Windows Terminal. Ler os parámetros de mestura do sistema (`CreateRenderingParams`) leva uns milisegundos.

## Decisión

1. **DirectWrite co crate `windows`**, só no módulo `dwrite.rs` de `meiga-render`, e `windows-core` para a macro `#[implement]`. `windows-sys` segue para Win32 e GDI.
2. **A fonte:** Cascadia Mono, ou Consolas se non está instalada, a 12 puntos e co peso normal. O tamaño da cela sae das métricas da fonte ao DPI da xanela:
   - o ancho é o avance do `0`, redondeado;
   - a liña base, o ascenso máis a metade do espazo entre liñas (*line gap*), redondeada;
   - o alto, a liña base máis o descenso e a outra metade do espazo, redondeado.
3. **A rasterización:** un glifo de cada vez, con `CreateGlyphRunAnalysis`: modo `NATURAL_SYMMETRIC`, medida `NATURAL`, axuste á grella de píxeles (*hinting*) e antialiasing en grises. A textura (`DWRITE_TEXTURE_ALIASED_1x1`) ten un byte de cobertura por píxel, de 0 a 255. Sen Direct2D nin ningún dispositivo.
4. **Cada glifo dentro da súa cela**, ou das dúas se o carácter é ancho: centrado en horizontal se avanza un ancho distinto, e cortado o que sae. Un carácter ancho rasterízase unha vez, nas dúas celas, e gárdase como dúas metades, unha por cela.
5. **A caché de glifos:**
   - cada glifo nun *slot* do tamaño dunha cela, todos seguidos nun mesmo bloque de memoria;
   - un índice por carácter, nunha táboa *hash* cun *hash* dunha multiplicación;
   - sitio para 4 096 *slots*, pero a memoria medra só cos glifos que se usan;
   - cando está chea, un glifo novo colle o *slot* dun que non se usou ultimamente, co algoritmo do reloxo (*clock* ou «segunda oportunidade»);
   - cada *slot* garda o rectángulo con tinta, que é o único que se debuxa.
6. **O *fallback*:** se a fonte da cela non ten o carácter, úsase a que escolle o *fallback* do sistema, coa configuración rexional do usuario. Se ningunha o ten, debúxase o glifo dos caracteres que faltan (*.notdef*) da fonte da cela. Se a rasterización falla, a cela queda co fondo.
7. **A mestura:** para cada par de cores, unha táboa cos 256 tons entre o fondo e a cor do texto. Mestúrase cun gamma de 1,5, e a cobertura do texto escuro refórzase cun contraste de 0,5 × (1 − a luminosidade da cor do texto). Con estes valores, unha liña de Cascadia Mono ao 150 % ten a mesma tinta ca no Windows Terminal, cunha diferenza menor do 0,5 %, tanto en texto claro sobre fondo escuro coma ao revés. Son constantes ata que a subfase 8.3 as faga configurábeis. Non se usa `CreateRenderingParams`.
8. **O ancho dos caracteres** (unha cela, dúas ou ningunha) é o que dá o crate `unicode-width`, o mesmo que usa `alacritty_terminal` ([ADR 0003](0003-nucleo-vt-alacritty-terminal.md)). Calcúlao `meiga-core`.

Isto concreta o [ADR 0002](0002-windows-e-renderizado-cpu.md) no punto da caché de glifos.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Chamar a COM con `windows-sys`, escribindo a man as táboas de métodos | Centos de liñas de código `unsafe` por cada interface, coa súa conta de referencias. O crate `windows` é a forma oficial e xera ese código desde os metadatos de Windows. |
| Debuxar o texto con Direct2D (`DrawGlyphRun`) | Cada fotograma pasaría por un dispositivo gráfico, que sen GPU emula WARP ([ADR 0002](0002-windows-e-renderizado-cpu.md)). A caché só necesita a cobertura de cada glifo, unha vez. |
| Texto con GDI (`ExtTextOut`, `GetGlyphOutline`) | Sen o *fallback* de DirectWrite e con menos calidade ([ADR 0002](0002-windows-e-renderizado-cpu.md)). |
| Antialiasing ClearType | Bordos de cor, depende do monitor do outro equipo por Escritorio remoto, e mesturar as tres canles por separado triplica o traballo de cada píxel. |
| Un atlas con glifos de tamaño variábel, como os das GPU | Máis complexo de encher e de baleirar. Os glifos dun terminal son case todos do tamaño dunha cela, e os *slots* fixos fan que atopar e substituír un glifo sexa inmediato. O que sae da cela pérdese: os bloques e as liñas de caixa debuxaranse por código na subfase 8.2. |
| Substituír o glifo usado hai máis tempo (LRU) cunha lista ligada | Máis memoria e actualizar punteiros cada vez que se usa un glifo. O reloxo só marca un *bit*. |
| A táboa *hash* coa función de *hash* estándar (SipHash) | Protexe contra claves escollidas por un atacante, que eiquí non existen, e é máis lenta. |
| Mestura lineal | O texto claro sobre fondo escuro vese máis fino ca no Windows Terminal. |
| Os parámetros de mestura do sistema (`CreateRenderingParams`) e a corrección de Direct2D | Uns milisegundos máis ao arrancar, e un algoritmo máis complexo. Unha táboa simple, axustada contra o Windows Terminal, dá a mesma tinta. |
| Crates de fontes e rasterizadores propios (`font-kit`, `swash`, `fontdue`, `ab_glyph`) | Máis dependencias, un aspecto distinto ao do resto de Windows e sen o *fallback* nin o *hinting* de DirectWrite. |
| Táboas de anchos propias | O ancho de cada carácter ten que coincidir co da grella do terminal, que calcula `alacritty_terminal` con `unicode-width`; se non, o cursor desaxústase. |

## Consecuencias

**Positivas:**

- O texto vese coma no Windows Terminal: a mesma rasterización e a mesma tinta.
- Cada glifo rasterízase unha vez por tamaño de fonte. Debuxar unha cela é buscala na táboa e copiar os píxeles con tinta a través da táboa de tons.
- Sen dispositivo gráfico: o mesmo comportamento con e sen GPU, en local e por Escritorio remoto.
- A memoria ten un límite: 4 096 *slots* por fonte, uns 1,6 MB con celas de 14 × 28 píxeles (150 %), e só se reserva a que se usa.
- O *fallback* debuxa calquera escritura que teña algunha fonte instalada en Windows.

**Negativas e riscos:**

- Os glifos que saen da cela córtanse: os bloques e as liñas de caixa poden deixar ocos dun píxel. Mitigación: debuxalos por código na subfase 8.2. A cursiva (1.6) pode perder algún píxel pola dereita.
- Os emoji vense dunha soa cor. Mitigación: en cor na subfase 8.1, con Direct2D sobre WIC.
- O primeiro carácter que lle falta á fonte custa uns milisegundos, unha vez por proceso.
- Co crate `windows` chegan 14 crates novos, contándoo a el, catro deles só para compilar (`proc-macro2`, `quote`, `syn` e `unicode-ident`), e máis tempo de compilación. `meiga.exe` importa agora ao arrancar `dwrite.dll`, `combase.dll` e `oleaut32.dll`.
- O gamma e o contraste son fixos ata a subfase 8.3.
- A fonte créase co DPI da xanela ao arrancar. Mitigación: os cambios de DPI, na subfase 1.9.

## Referencias

- Microsoft Learn:
  - [`IDWriteFactory2::CreateGlyphRunAnalysis`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_2/nf-dwrite_2-idwritefactory2-createglyphrunanalysis), [`IDWriteGlyphRunAnalysis::CreateAlphaTexture`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwriteglyphrunanalysis-createalphatexture) e [`DWRITE_TEXTURE_TYPE`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ne-dwrite-dwrite_texture_type);
  - [`IDWriteFontFallback::MapCharacters`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_2/nf-dwrite_2-idwritefontfallback-mapcharacters) e [`IDWriteTextAnalysisSource`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritetextanalysissource);
  - [`DWRITE_FONT_METRICS`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ns-dwrite-dwrite_font_metrics) e [`IDWriteFactory::CreateRenderingParams`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritefactory-createrenderingparams);
  - Windows Terminal, [*Text antialiasing*](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/profile-advanced#text-antialiasing): `antialiasingMode` vale `"grayscale"` por defecto.
- Windows Terminal v1.25.2733.0:
  - [`src/renderer/atlas/BackendD2D.cpp`, liña 289](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/renderer/atlas/BackendD2D.cpp#L289): sen GPU, o texto debúxase con Direct2D (`DrawGlyphRun`);
  - [`src/renderer/atlas/dwrite_helpers.cpp`, liñas 20–21](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/renderer/atlas/dwrite_helpers.cpp#L20-L21): «DirectWrite's alpha blending is gamma corrected and thus text color dependent».
- [`windows_core::implement`](https://docs.rs/windows-core/0.62.2/windows_core/attr.implement.html) e [`unicode-width` 0.2.3](https://docs.rs/unicode-width/0.2.3/unicode_width/).
- Unicode, [UAX #11: East Asian Width](https://www.unicode.org/reports/tr11/).
- Wikipedia, [*Page replacement algorithm* › *Clock*](https://en.wikipedia.org/wiki/Page_replacement_algorithm#Clock).
- [Tecnoloxías › DirectWrite](../tecnoloxias.md#27-directwrite-fontes-e-glifos) e [Código › meiga-render](../codigo/meiga-render.md#cratesmeiga-rendersrcdwriters).

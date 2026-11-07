# ADR 0002 · Só Windows, con renderizado por CPU

- **Estado:** Aceptado
- **Data:** 2026-11-06
- **Relacionados:** [ADR 0004](0004-conpty-empaquetado.md), [ADR 0005](0005-fios-e-fotogramas.md)

## Contexto

- meiga é para Windows: as shells que se usan (PowerShell 7, Windows PowerShell, cmd e as distribucións de WSL) e o seu contorno son de Windows.
- Debe funcionar ben tamén en **máquinas sen GPU** e en **sesións de Escritorio remoto (RDP)**, como as máquinas virtuais na nube. Nesas máquinas:
  - Direct3D non ten hardware no que apoiarse e Windows usa **WARP**, un rasterizador por software que emula a GPU na CPU, repartindo o traballo entre varios fíos.
  - O RDP ten que comprimir e enviar pola rede cada rexión da pantalla que cambia: canto máis se redebuxa, máis traballo para a máquina e para a rede.
- Moitos terminais modernos debuxan toda a xanela coa GPU en cada fotograma (Direct3D, Vulkan ou OpenGL, ás veces a través de capas como wgpu). Cunha GPU real é barato; sen ela, ese traballo recae enteiro na CPU.
- O propio Windows Terminal, cando detecta un adaptador por software (WARP), cambia automaticamente o seu motor de Direct3D a Direct2D (ver as referencias).
- O contido dun terminal é unha grella de celas de texto e, a miúdo, só cambian unhas poucas: o cursor, unha barra de progreso, un *spinner*.

## Decisión

1. **Só Windows.** Usamos directamente as APIs de Windows (Win32, DirectWrite, GDI e ConPTY), sen capas multiplataforma.
2. **Renderizado por CPU, só do que cambia:**
   - os glifos rasterízanse unha soa vez con DirectWrite e gárdanse nunha caché;
   - cada fotograma compón no *framebuffer*, unha *DIB section* en memoria, **só as celas que cambiaron**;
   - na xanela preséntanse **só os rectángulos modificados**, con `BitBlt` de GDI;
   - se a xanela está minimizada ou tapada, non se debuxa nada.
3. **Preparado para a GPU.** O renderizador traballa sobre unha *display list* (a lista do que hai que debuxar) detrás dunha interface `Renderer`. Un backend Direct3D 11 para as máquinas con GPU sería outra implementación desa interface (mellora futura F2).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Multiplataforma (Windows, Linux e macOS) | Obriga a capas de abstracción para as xanelas, as fontes e o PTY, que engaden custo e complexidade. O obxectivo é Windows. |
| GPU sempre (Direct3D, Vulkan, wgpu) | Excelente cunha GPU real. Sen ela, WARP fai todo o traballo na CPU, e redebuxar a xanela enteira en cada fotograma multiplica o custo. |
| Direct2D sobre WARP, como fai Windows Terminal sen GPU | Mellor ca Direct3D sobre WARP, pero segue pasando por un dispositivo gráfico emulado. Compoñer nós só as celas cambiadas dá control total sobre o custo. |
| Texto directo con GDI (`ExtTextOut`) | Sen emoji en cor nin o *fallback* de fontes de DirectWrite (CJK, símbolos), e con menos control sobre a calidade do texto. |
| Redebuxar a xanela enteira en cada fotograma | Máis simple, pero o custo dependería do tamaño da xanela e non dos cambios, e o RDP tería que enviar a xanela enteira. |

## Consecuencias

**Positivas:**

- O custo dun fotograma é proporcional ao que cambia: un cursor que pestanexa ou un *spinner* custan unhas poucas celas.
- O mesmo comportamento con e sen GPU, en local e por RDP.
- Menos traballo para o RDP: só viaxan os rectángulos que cambian.
- Sen dependencia dos controladores gráficos.

**Negativas e riscos:**

- Os cambios que afectan a toda a pantalla, como desprazar unha saída masiva ou cambiar de tab, custan máis na CPU ca nunha GPU. Mitigación: desprazar movendo o *framebuffer* en memoria e debuxar só as liñas novas (subfase 9.2), e limitar os fotogramas por segundo ([ADR 0005](0005-fios-e-fotogramas.md)).
- En resolucións moi altas, un redebuxado completo move moitos píxeles. Mitigación: o backend GPU (F2).
- Hai que implementar a caché de glifos e a composición. Mitigación: o alcance é pequeno, porque a grella é monoespazada e os glifos teñen o tamaño dunha cela.

## Referencias

- Microsoft Learn, [Windows Advanced Rasterization Platform (WARP) Guide](https://learn.microsoft.com/en-us/windows/win32/direct3darticles/directx-warp).
- Windows Terminal v1.25.2733.0, [`src/renderer/atlas/AtlasEngine.r.cpp`, liñas 192–201](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/renderer/atlas/AtlasEngine.r.cpp#L192-L201): se o adaptador é por software, o modo automático escolle Direct2D.
- [Arquitectura › Fluxo de datos](../arquitectura.md#3-fluxo-de-datos-previsto)

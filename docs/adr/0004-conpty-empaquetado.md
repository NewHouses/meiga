# ADR 0004 · ConPTY empaquetado con meiga

- **Estado:** Aceptado; concretado polos [ADR 0011](0011-fin-e-peche-das-sesions.md) e [ADR 0014](0014-o-teclado.md)
- **Data:** 2026-11-06
- **Relacionados:** [ADR 0002](0002-windows-e-renderizado-cpu.md), [ADR 0005](0005-fios-e-fotogramas.md), [ADR 0006](0006-licenzas-e-dependencias.md)

## Contexto

- Windows non ten pseudoterminais coma Unix. Desde Windows 10 1809 (build 17763) ofrece **ConPTY** (*pseudoconsole*): unha consola sen xanela que traduce a API de consola de Windows, que usan as shells, a un fluxo de texto con secuencias VT, que é o que le o terminal.
- O ConPTY que trae Windows (`CreatePseudoConsole`, en `kernel32`) depende da versión do sistema e recibe as melloras con atraso. Microsoft publica tamén o ConPTY de Windows Terminal como paquete NuGet, `Microsoft.Windows.Console.ConPTY` (licenza MIT): `conpty.dll` máis `OpenConsole.exe`, compatíbel desde Windows 10.0.17763.
- Ese ConPTY ofrece opcións que meiga necesita. Por exemplo, o flag `PSEUDOCONSOLE_GLYPH_WIDTH_WCSWIDTH` fai que ConPTY mida o ancho dos caracteres coa mesma regra (`wcswidth`) que usa `alacritty_terminal`. Se os dous lados non coinciden, o cursor desaxústase cos emoji e os caracteres CJK.
- Ao arrancar, ConPTY pídelle ao terminal os atributos do dispositivo (DA1) e activa os eventos de foco (`?1004h`) e o **win32-input-mode** (`?9001h`). Neste modo, o terminal envía cada evento de teclado completo: tecla virtual, código de exploración, carácter, se se preme ou se solta e o estado das teclas modificadoras. Así PSReadLine, AltGr e as teclas mortas funcionan igual ca nunha consola de Windows.

## Decisión

- Distribuímos `conpty.dll` e `OpenConsole.exe` xunto a `meiga.exe`, tomados do paquete NuGet cunha versión fixada (1.25.260930003) e o seu SHA-256 comprobado ao descargalo. Farao o script `tools/fetch-conpty.ps1` (subfase 1.4).
- meiga carga `conpty.dll` en tempo de execución e usa `ConptyCreatePseudoConsole`. Se non a atopa, usa `CreatePseudoConsole` de `kernel32`.
- Creamos a pseudoconsola co flag `PSEUDOCONSOLE_GLYPH_WIDTH_WCSWIDTH`.
- Respondemos a DA1 e usamos win32-input-mode cando ConPTY o activa; se non, a codificación VT clásica (subfase 1.7).
- A licenza MIT do ConPTY engádese a `THIRD-PARTY-NOTICES.txt` ([ADR 0006](0006-licenzas-e-dependencias.md)).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Só o ConPTY de Windows | O comportamento cambiaría coa versión de Windows, e faltarían as últimas correccións e opcións. Queda como reserva. |
| winpty | Anterior a ConPTY e sen mantemento activo: lía unha consola oculta de forma indirecta. |
| Ler directamente o contido dunha consola oculta | Lento e pouco fiábel. É o problema que ConPTY xa resolve. |

## Consecuencias

**Positivas:**

- O mesmo comportamento en todas as versións de Windows compatíbeis.
- Anchos de carácter coherentes entre ConPTY e a grella: o cursor non se desaxusta con emoji nin con CJK.
- Fidelidade total do teclado con win32-input-mode.

**Negativas e riscos:**

- O paquete leva dous arquivos máis.
- Hai que actualizar a versión do NuGet de cando en vez. Mitigación: versión e SHA-256 fixados nun único script; na subfase 1.4 estudarase se Dependabot pode vixiala.
- Cada shell en marcha ten o seu proceso de consola (`OpenConsole.exe`), igual que co ConPTY de Windows (`conhost.exe`).

## Referencias

- Microsoft Learn, [Creating a Pseudoconsole session](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session).
- [`Microsoft.Windows.Console.ConPTY` en NuGet](https://www.nuget.org/packages/Microsoft.Windows.Console.ConPTY): versión 1.25.260930003, compatíbel desde Windows 10.0.17763.
- Windows Terminal v1.25.2733.0, [`src/inc/conpty-static.h`, liñas 30–51](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/inc/conpty-static.h#L30-L51): os flags `PSEUDOCONSOLE_GLYPH_WIDTH_*` e as funcións `Conpty*`.
- Windows Terminal v1.25.2733.0, [`src/host/VtIo.cpp`, liñas 201–203](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/host/VtIo.cpp#L201-L203): as peticións de DA1, `?1004h` e `?9001h` ao arrancar.

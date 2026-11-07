# ADR 0007 · Integración coa shell

- **Estado:** Aceptado
- **Data:** 2026-11-06
- **Relacionados:** [ADR 0003](0003-nucleo-vt-alacritty-terminal.md)

## Contexto

- O explorador de cartafoles, duplicar unha tab no mesmo cartafol e o título por defecto da tab necesitan saber o **cartafol actual** de cada shell.
- Facer `cd` desde o explorador só é seguro se a shell está agardando unha orde, no *prompt*. Se hai un programa en execución (vim, unha compilación…), o texto inxectado iría para ese programa.
- Ler o cartafol do proceso desde fóra non é fiábel: a shell pode lanzar subprocesos, e nas shells de WSL o proceso de Windows non coñece o cartafol de Linux.
- Os terminais modernos resolven isto con secuencias OSC que emite a propia shell, normalmente desde o seu *prompt*:
  - **OSC 7**: o cartafol como URL `file://equipo/ruta`, o estándar de facto en Linux;
  - **OSC 9;9**: o cartafol como ruta de Windows, a convención de ConEmu que tamén usa Windows Terminal;
  - **OSC 133**: marcas do *prompt*: inicio do *prompt* (A), inicio da orde (B), inicio da saída (C) e fin da orde co seu código de saída (D).
- `vte` 0.15.0 non trata ningunha delas: a súa función `osc_dispatch` só recoñece os códigos 0, 2, 4, 8, 10–12, 22, 50, 52, 104 e 110–112.

## Decisión

- **Escáner previo.** En `meiga-core::term`, antes do parser, un escáner busca o carácter `ESC` con `memchr`, extrae OSC 7, 9;9 e 133 e pásalle **todos os bytes, sen cambios,** ao parser. Funciona aínda que unha secuencia chegue partida entre dúas lecturas.
- **Scripts de integración**, un por shell:
  - **PowerShell 7 e Windows PowerShell:** `meiga.ps1`, que meiga inxecta automaticamente ao lanzar a shell (`-NoExit -Command`). Envolve a función `prompt` existente despois de cargar o perfil, polo que funciona tamén con oh-my-posh e starship, e emite OSC 133 e 9;9.
  - **cmd:** a variábel `PROMPT`, con OSC 9;9.
  - **bash en WSL:** `meiga.bash`, con OSC 7 e 133. Instálase engadindo unha liña a `~/.bashrc`, só se o usuario o acepta.
- **`cd` seguro.** O explorador só inxecta `cd` se a tab está no *prompt*, segundo OSC 133. Antes baleira a liña coa tecla axeitada para cada shell, e a ruta vai sempre entre comiñas e cos escapes desa shell.
- Se unha shell non emite estas secuencias, meiga funciona igual, pero sen coñecer o seu cartafol.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Ler o cartafol do proceso desde fóra | Depende de estruturas internas de Windows, non funciona con WSL e falla cando a shell lanza subprocesos. |
| Deducir o cartafol do texto do *prompt* | Fráxil: cada persoa ten un *prompt* distinto. |
| Pedirlle ao usuario que edite os seus perfís | Funciona, pero require configuración manual. A inxección automática cobre os casos habituais. |
| Modificar `vte` para que trate estes códigos | Habería que manter unha copia propia de `vte` e actualizala con cada versión. |

## Consecuencias

**Positivas:**

- Cartafol e estado do *prompt* fiábeis nas catro shells principais.
- O escáner non modifica os bytes, así que o parser compórtase exactamente igual.

**Negativas e riscos:**

- A inxección automática ten que convivir co perfil de cada usuario. Mitigación: probala cun perfil real con oh-my-posh (subfase 4.2).
- O escáner engade traballo por cada byte. Mitigación: `memchr` usa instrucións SIMD, e hai un obxectivo medido: menos do 1 % do tempo do parser (subfase 4.1).
- Un programa podería emitir un OSC 7 ou 9;9 falso. Mitigación: o cartafol só serve para suxerir onde navegar e nunca executa nada por si só, e as rutas sempre van entre comiñas e escapadas.

## Referencias

- `vte` v0.15.0, [`src/ansi.rs`](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs), función `osc_dispatch`.
- Microsoft Learn, [Shell integration in the Windows Terminal](https://learn.microsoft.com/en-us/windows/terminal/tutorials/shell-integration): marcas OSC 133.
- Microsoft Learn, [Opening a tab/pane in the same directory](https://learn.microsoft.com/en-us/windows/terminal/tutorials/new-tab-same-directory): OSC 9;9.

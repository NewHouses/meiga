# ADR 0029 · O título da tab e os títulos gardados

- **Estado:** Aceptado; concretado polo [ADR 0032](0032-o-escaner-osc.md)
- **Data:** 2026-11-25
- **Relacionados:** [ADR 0003](0003-nucleo-vt-alacritty-terminal.md), [ADR 0004](0004-conpty-empaquetado.md), [ADR 0012](0012-fachada-do-terminal.md), [ADR 0013](0013-da-saida-a-pantalla.md), [ADR 0020](0020-a-barra-de-tabs.md), [ADR 0022](0022-os-perfis-e-a-configuracion.md), [ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md)

## Contexto

- **O título, ata a 3.1.** O programa pon o título da súa tab con OSC 0 ou OSC 2, e gárdao e vóltao dar con `CSI 22 t` e `CSI 23 t` (XTWINOPS), coma vim ao entrar e ao saír. `vte` dálle ao terminal o título sen os espazos dos extremos, tamén se queda baleiro; `alacritty_terminal` gardábao, e tamén os que gardaba o programa, ata 4096, e avisaba á fachada con `Title` ou `ResetTitle` ([ADR 0012](0012-fachada-do-terminal.md)). A fachada levaba a súa copia: un título baleiro é «sen título», e entón a tab mostra o nome do seu perfil ([ADR 0022](0022-os-perfis-e-a-configuracion.md)) e a xanela, «meiga» ([ADR 0013](0013-da-saida-a-pantalla.md), decisión 11).
- **O reinicio da pantalla, `ESC c` (RIS).** `alacritty_terminal` baleira por dentro o título e os gardados, sen avisar. A tab mantiña o último título, desde a 1.5, pero o modelo xa non o tiña: se despois o programa gardaba o título e voltábao dar, a tab quedaba co nome do perfil. E o reinicio dunha sesión na súa tab ([ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md)) deixaba a tab sen título, cun OSC 0 sen texto, pero non baleiraba os gardados: un `CSI 23 t` do programa novo traía un título do vello.
- **Os títulos longos.** `vte` non pon límite ao texto dun OSC. Un título de calquera lonxitude chegaba á tab e á xanela (`SetWindowTextW`), e cada título gardado era outra copia: cun título de 1 MB e 4096 gardados, ata 4 GB.
- **O ConPTY 1.25** pásalle ao terminal o texto do programa tal cal: procésao co seu analizador e despois escribe o orixinal (`WriteCharsVT`); despois de `ESC c`, engade só `ESC [ ? 1004 h ESC [ ? 9001 h`, sen título. Unha proba de integración compróbao co ConPTY empaquetado: OSC 0, 1, 2 e 21, `CSI 22 t`, `CSI 23 t` e `ESC c` chegan sen cambios. O título da propia consola, o de `GetConsoleTitle`, non cambia con `ESC c`; un baleiro volta ao título orixinal da consola; e un de máis de 1000 caracteres ignórase, co mesmo código ca o Windows Terminal (`AdaptDispatch::SetWindowTitle`).
- **As referencias,** polo seu código:

| | Título despois de `ESC c` | Títulos gardados | Gardados despois de `ESC c` | Títulos longos |
|---|---|---|---|---|
| Windows Terminal 1.25 | Mantense | Non ten: `CSI 22 t` e `CSI 23 t` non fan nada | — | Máis de 1000 caracteres UTF-16: ignórase, «coma en xterm» |
| xterm 412 | Mantense | 10, nun anel que sobrescribe o máis vello | Mantéñense | Máis de 1000 bytes: ignórase |
| Kitty 0.51.1 | Mantense | 10; ao pasar, vaise o máis vello | Mantéñense | — |
| Alacritty 0.17 | A xanela mantéño, pero o modelo baléirao | 4096 | Baléiranse | Sen límite |
| WezTerm, xterm.js e VS Code | Mantense | Non, ou opcional | — | — |

  No Windows Terminal, un título baleiro mostra o de partida, o `tabTitle` ou o nome do perfil, e OSC 1 (o título da icona) e OSC 21 (DECSWT) cambian o título coma OSC 0 e OSC 2; `vte` non os pasa.

## Decisión

1. **`ESC c` mantén o título** (decidido contigo), coma todas as referencias e coma o propio ConPTY, así que a tab, a xanela e a consola seguen dicindo o mesmo.
2. **O terminal leva o título e os gardados** (decidido contigo). A fachada atende ela mesma OSC 0 e OSC 2, `CSI 22 t` e `CSI 23 t` (`set_title`, `push_title` e `pop_title` do `Handler`, en `Hooked`), e `alacritty_terminal` xa non os recibe: un módulo novo, `term::title`. `Terminal::title` dá o título, e `Changes::title`, que cambiou: dar outra vez o mesmo título non é un cambio. Un título baleiro, ou só de espazos, segue sendo «sen título».
3. **`ESC c` mantén tamén os gardados** (decidido contigo), coma xterm e Kitty: son da xanela, coma o título, e non da pantalla.
4. **O reinicio da sesión na súa tab déixaa sen título e sen títulos gardados** (decidido contigo), coma un terminal novo. Concreta o [ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md): ata agora só quitaba o título.
5. **Como moito, 10 títulos gardados** (decidido contigo), coma en xterm e Kitty: gardar outro quita o máis vello. Tamén se garda «sen título», e voltar dalo volta ao nome do perfil, coma en Alacritty; e voltar dar un título cando non hai ningún gardado non fai nada.
6. **Un título de máis de 1000 caracteres ignórase** (decidido contigo): a tab queda co que tiña, coma no Windows Terminal, en xterm e no ConPTY. Cóntanse coma os conta Windows, en unidades de UTF-16, así que un emoji conta dúas. Cos 10 gardados, os títulos dunha tab ocupan como moito uns 33 KB.
7. **OSC 1 e OSC 21 seguen sen facer nada** (decidido contigo), coma en Alacritty. Se se queren coma no Windows Terminal, mellor na 4.1, co escáner de OSC: `vte` non os pasa, e habería que collelos no escáner de meiga, que entón aprende a ler un OSC partido entre dúas lecturas.

Isto concreta o [ADR 0012](0012-fachada-do-terminal.md) (o título xa non vén dos eventos de `alacritty_terminal`) e o [ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md) (o reinicio na tab baleira tamén os títulos gardados).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Que `ESC c` volte ao nome do perfil, coma nun terminal novo | Decidido contigo: mantelo. Ningunha referencia o fai, e o ConPTY mantén o seu: a tab e a consola dirían títulos distintos. |
| Voltar pór en `alacritty_terminal`, despois de cada `ESC c`, o título que mostra a tab | Decidido contigo: que a fachada os leve. Só arranxaría o reinicio da pantalla, e non o da sesión na tab nin os títulos longos; e cada `ESC c` avisaría dun título que non cambiou. |
| Deixalo coma estaba, e documentalo | Decidido contigo: arranxalo. A tab quedaba co nome do perfil sen que o programa o pedise. |
| Baleirar os gardados con `ESC c`, coma o modelo de Alacritty | Decidido contigo: mantelos, coma xterm e Kitty. Son da xanela, coma o título, que se mantén. |
| 4096 títulos gardados, coma Alacritty | Decidido contigo: 10, coma xterm e Kitty. Un programa garda un ou dous, e cada un pode ter ata 1000 caracteres: con 4096, ata uns 12 MB por tab. |
| Recortar os títulos longos aos 1000 primeiros caracteres, ou aceptalos todos, coma ata agora | Decidido contigo: ignoralos. Recortado, a tab e o ConPTY dirían títulos distintos; e sen límite, un programa podía encher a memoria con títulos. |
| OSC 1 e OSC 21 coma OSC 0 e OSC 2, coma o Windows Terminal | Decidido contigo: polo de agora, non. `vte` non os pasa, e collelos no escáner de meiga é traballo da 4.1. |

## Consecuencias

**Positivas:**

- Despois de `ESC c`, a tab e a xanela seguen co título; e gardar o título e voltar dalo trae o mesmo, tamén o que se gardou antes do reinicio.
- Un programa novo na tab empeza coma nun terminal novo, sen títulos do anterior.
- Un título non pasa de 1000 caracteres, nin os gardados de 10: cada tab garda como moito uns 33 KB de títulos, en vez de ata gigabytes.
- O terminal só avisa dun título cando cambia: o mesmo outra vez xa non é un cambio, e nunha tab oculta, un programa que o dá unha e outra vez xa non esperta a xanela; ata a 3.1, espertábaa unha vez por intervalo de fotograma.
- `alacritty_terminal` xa non garda títulos: unha versión nova que os trate doutra maneira non cambia nada en meiga.

**Negativas e riscos:**

- OSC 1 e OSC 21 non cambian o título, e no Windows Terminal si: un programa que só pon o título da icona non cambia a tab.
- `CSI 22 t` e `CSI 23 t` gardan e voltan dar o título sen mirar o seu segundo número, coma `vte`: `CSI 22 ; 1 t`, que en xterm garda só o título da icona, garda tamén o da xanela. Gardar e voltar dar sempre en parellas, coma fan os programas, dá o mesmo.
- `vte` quita os espazos dos extremos antes de que meiga o conte, e o ConPTY conta con eles: un título de case 1000 caracteres con espazos nos extremos pode cambiar en meiga e non na consola.
- `vte` garda todo o texto dun OSC mentres chega, sen límite, e despois conserva esa memoria no analizador da tab: un título enorme, aínda que meiga o ignore, ocupa esa memoria ata que se pecha a tab.
- `alacritty_terminal` 0.26.0 quita un título da súa pila de títulos cando se enche a pila do teclado de Kitty (`push_keyboard_mode`), en vez de quitalo desta; e a súa pila de títulos, desde a 3.1, está sempre baleira, así que entraría en pánico. meiga non usa o teclado de Kitty (`kitty_keyboard: false`), e esa función volta antes; se algún día o usa, hai que miralo.

## Referencias

- Windows Terminal v1.25.3131.0, as mesmas fontes ca o ConPTY empaquetado:
  - [`AdaptDispatch::SetWindowTitle`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/adapter/adaptDispatch.cpp#L2613-L2622), que ignora os títulos de máis de 1000 caracteres; [`AdaptDispatch::HardReset`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/adapter/adaptDispatch.cpp#L3029-L3122), `ESC c`, que non toca o título; e [`AdaptDispatch::WindowManipulation`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/adapter/adaptDispatch.cpp#L3469-L3513), sen `CSI 22 t` nin `CSI 23 t`;
  - [`OutputStateMachineEngine::ActionOscDispatch`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/parser/OutputStateMachineEngine.cpp#L763-L774): OSC 0, 1, 2 e 21;
  - [`WriteCharsVT`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/host/_stream.cpp#L389-L464), que lle pasa ao terminal o texto do programa e, despois de `ESC c`, `ESC [ ? 1004 h ESC [ ? 9001 h`; e [`ConhostInternalGetSet::SetWindowTitle`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/host/outputStream.cpp#L185-L189), o título da consola.
- xterm 412: [`MAX_SAVED_TITLES`](https://github.com/ThomasDickey/xterm-snapshots/blob/xterm-412/ptyx.h#L2371), [`xtermPushTitle` e `xtermPopTitle`](https://github.com/ThomasDickey/xterm-snapshots/blob/xterm-412/misc.c#L7972-L8033), [os títulos de máis de 1000 bytes](https://github.com/ThomasDickey/xterm-snapshots/blob/xterm-412/misc.c#L6141-L6145), e [`VTDestroy`](https://github.com/ThomasDickey/xterm-snapshots/blob/xterm-412/charproc.c#L12126-L12148), o único sitio que borra os gardados.
- Kitty v0.51.1, `kitty/window.py`: [os títulos gardados](https://github.com/kovidgoyal/kitty/blob/v0.51.1/kitty/window.py#L876), [`on_reset`](https://github.com/kovidgoyal/kitty/blob/v0.51.1/kitty/window.py#L1804-L1812) e [`manipulate_title_stack`](https://github.com/kovidgoyal/kitty/blob/v0.51.1/kitty/window.py#L2120-L2129).
- Alacritty v0.17.0, `alacritty_terminal` 0.26.0: [`TITLE_STACK_MAX_DEPTH`](https://github.com/alacritty/alacritty/blob/v0.17.0/alacritty_terminal/src/term/mod.rs#L41-L42), [`reset_state`](https://github.com/alacritty/alacritty/blob/v0.17.0/alacritty_terminal/src/term/mod.rs#L1833-L1858), e [`set_title`, `push_title` e `pop_title`](https://github.com/alacritty/alacritty/blob/v0.17.0/alacritty_terminal/src/term/mod.rs#L2220-L2256).
- `vte` 0.15.0: [OSC 0 e OSC 2](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs#L1344-L1363), [`CSI 22 t` e `CSI 23 t`](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs#L1739-L1744), e [o texto dun OSC](https://github.com/alacritty/vte/blob/v0.15.0/src/lib.rs#L543-L559), sen límite.
- WezTerm [20240203-110809-5046fc22](https://github.com/wezterm/wezterm/blob/20240203-110809-5046fc22/term/src/terminalstate/performer.rs#L662-L749), xterm.js [6.0.0](https://github.com/xtermjs/xterm.js/blob/6.0.0/src/common/InputHandler.ts#L3324-L3333) e VS Code [1.105.0](https://github.com/microsoft/vscode/blob/1.105.0/src/vs/workbench/contrib/terminal/common/terminalConfiguration.ts#L337-L340).
- [Tecnoloxías › Os títulos da tab](../tecnoloxias.md#65-os-títulos-da-tab).
- [Código › meiga-core › `term/title.rs`](../codigo/meiga-core.md#cratesmeiga-coresrctermtitlers).
- [Subfases › 3.1 · Títulos dinámicos](../subfases/3.1-titulos.md).

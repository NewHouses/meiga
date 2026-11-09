# ADR 0014 · O teclado

- **Estado:** Aceptado
- **Data:** 2026-11-09
- **Relacionados:** [ADR 0004](0004-conpty-empaquetado.md), [ADR 0012](0012-fachada-do-terminal.md), [ADR 0013](0013-da-saida-a-pantalla.md)

## Contexto

- Na subfase 1.7, o teclado chega ao programa. O [ADR 0004](0004-conpty-empaquetado.md) xa decide o camiño: o **win32-input-mode** cando o ConPTY o activa e, se non, a codificación VT clásica. Faltaba concretar como se converten as mensaxes de Windows en teclas, e que teclas son para Windows. Antes de escribilo, comprobouse:
  - **As mensaxes das teclas.** Windows envía unha mensaxe cando unha tecla baixa e outra cando sobe: `WM_KEYDOWN` e `WM_KEYUP` ou, con Alt e con F10, `WM_SYSKEYDOWN` e `WM_SYSKEYUP`. Levan o código de tecla virtual, que di que tecla é na distribución do teclado, e o código de exploración (*scan code*), que di onde está. O carácter chega noutra mensaxe: `TranslateMessage`, que o bucle de mensaxes chama antes de lle dar a mensaxe á xanela, publica `WM_CHAR` (ou `WM_SYSCHAR`, con Alt) co carácter que escribe a tecla na distribución do fío, ou `WM_DEADCHAR` se é unha tecla morta. Esa mensaxe vai ao final da cola: é a seguinte que recibe a xanela, salvo que xa houbese outras publicadas.
  - **Os eventos da consola.** Os programas de consola len as teclas coma eventos completos (`KEY_EVENT_RECORD`): a tecla, o código de exploración, o carácter, se baixa ou sobe, as teclas modificadoras e as de bloqueo, e as repeticións. O ConPTY pídelle ao terminal eses eventos ao arrancar (`ESC [ ? 9001 h`), cada un nunha secuencia: `ESC [ Vk ; Sc ; Uc ; Kd ; Cs ; Rc _`. Na especificación do modo, ao soltar a tecla A, o evento leva tamén o carácter, `a`, coma na consola.
  - **PSReadLine** le as teclas con `[Console]::ReadKey` de .NET, que só dá as teclas que baixan, e non as modificadoras soas. Así distingue Maiús+Intro de Intro, ou Ctrl+Espazo, que en VT son o mesmo ca Intro e ca `NUL`. A especificación do win32-input-mode naceu destes problemas. Unha tecla morta chega a .NET coma unha tecla sen carácter.
  - **O teclado español e galego.** As teclas mortas (´ e ¨) non escriben nada ata a tecla seguinte. AltGr escribe @ # € [ ] { } \ |, e Windows descríbea coma o Ctrl esquerdo e o Alt dereito premidos á vez.
  - **As teclas publicadas por outro programa.** As ferramentas de automatización, e a comprobación desta subfase, publican as mensaxes das teclas na cola da xanela. Entón, o carácter que publica `TranslateMessage` para unha tecla queda despois das mensaxes que xa estaban, e a seguinte mensaxe pode ser outra tecla.
  - **As teclas do sistema.** Con Alt, `DefWindowProcW` dálles significado: Alt+F4 pecha a xanela, Alt+Espazo abre o seu menú, Alt só e F10 activan a barra de menús, que meiga non ten, e Alt cunha letra busca no menú unha tecla de acceso e, se non a hai, fai soar un aviso. As teclas Windows son para os atallos de Windows. Cando un método de entrada (IME) colle unha tecla, a xanela recibe `VK_PROCESSKEY` no seu lugar.
  - **O IME.** Ao crear a xanela, Windows carga a pila de entrada de texto, coa que funcionan os IME (por exemplo, para chinés ou xaponés), o panel de emoji (Win + .) e o ditado (Win + H). Medido nunha copia de meiga que a desactivaba con `ImmDisableIME`: con ela, o proceso carga 11 módulos máis, usa algo máis de memoria e arranca uns milisegundos máis tarde; con foco e en repouso, non esperta nin unha vez. Sen ela, o panel de emoji ábrese pero non escribe nada; o ditado funciona igual.
  - Ata a subfase 1.6, a xanela só usaba unha tecla, a P, para as marcas dos fotogramas.

## Decisión

1. **Eventos completos.** meiga converte as mensaxes das teclas en eventos de tecla coma os da consola de Windows, `KeyEvent` en `meiga-core`: a tecla, o código de exploración, o carácter (unha unidade UTF-16), se baixa ou sobe, as teclas modificadoras e de bloqueo, e as repeticións. Faino o teclado da xanela, en `meiga-win`.
2. **O evento dunha tecla que baixa espera o seu carácter.** Se despois da mensaxe dunha tecla que baixa hai un carácter na cola (`PeekMessageW` sen sacalo), o evento espera por el e sae con el; se non, sae de inmediato. Unha tecla morta baixa sen carácter, e o seu vai coa tecla seguinte (´ e a escriben á), coma na consola. Se non se xuntan (´ e x escriben ´x), o segundo carácter chega sen ningunha tecla que o espere, e o seu evento leva a tecla do seu código de exploración. Das teclas que soben, só espera Alt, que ao soltalo escribe o carácter do código que se tecleou con el no teclado numérico: calquera outro carácter que estea na cola cando sobe unha tecla é dunha tecla anterior.
3. **As teclas publicadas.** Se chega a mensaxe doutra tecla mentres un evento espera, o evento colle da cola o primeiro carácter (`PeekMessageW` con `PM_REMOVE`) e sae antes.
4. **Ao soltar unha tecla, o evento di que escribe**, coma na consola e no Windows Terminal: `ToUnicodeEx` co estado das teclas, sen tocar o estado dunha tecla morta e sen contar o teclado numérico con Alt.
5. **Alt+F4 e Alt+Espazo son para Windows** (`DefWindowProcW`): pechan a xanela e abren o seu menú, coma no Windows Terminal. Alt só, F10 e as outras combinacións con Alt van ao programa (decidido contigo).
6. **Quedan fóra** as teclas Windows, as que colle un IME (`VK_PROCESSKEY`) e os códigos 0 e desde `0xFF`.
7. **O win32-input-mode,** se o programa o activou, normalmente o ConPTY ao arrancar: cada evento, ao baixar e ao subir, cos seis números sempre e as repeticións polo menos a 1, porque o ConPTY garda un 0 tal cal, e un programa podería perder a tecla.
8. **O VT de reserva,** antes de que o ConPTY active o modo ou se o desactiva: o que envía o xterm ao baixar a tecla. O carácter en UTF-8; as frechas, Inicio e Fin con `ESC [` ou, no modo aplicación (`?1`), con `ESC O`; Insert, Supr, RePáx, AvPáx e F5–F12 con `~`; F1–F4 con `ESC O`; e `;m` coas modificadoras. Retroceso envía DEL, e con Ctrl, BS; Maiús+Tab, `ESC [ Z`; Intro, CR, ou CR LF co modo *LNM*, e con Ctrl, LF. Alt pon un ESC diante. Ctrl cunha tecla sen carácter envía o seu código de control (Ctrl+Espazo, `NUL`), pero AltGr sen carácter non envía nada: é unha tecla morta, coma o ~ de AltGr+4 no teclado español, ou unha tecla sen carácter con AltGr. Os caracteres de AltGr van tal cal, e as dúas metades dun carácter de fóra do plano básico, xuntas. Unha tecla que trae un carácter que non é seu envía o carácter, non a súa secuencia: o ´ que trae Intro cando ´ e Intro non se xuntan, ou o 7 do teclado numérico que chega só coa tecla Inicio (decisión 2). E Alt, cando sobe co Bloq Num activado, envía o carácter do código que se tecleou con el no teclado numérico, coma o Windows Terminal.
9. **Os eventos de foco:** `ESC [ I` e `ESC [ O`, se o programa os pediu (`?1004`).
10. **As teclas van á entrada da sesión** (`Input::write`) desde o fío da interface, cun búfer que se reutiliza. Non se espera: o fío escritor envíaas ([ADR 0011](0011-fin-e-peche-das-sesions.md)).
11. **As marcas, cunha opción:** `--marks` en vez da tecla P, que agora vai ao programa (decidido contigo).
12. **O IME queda,** como fai Windows por defecto (decidido contigo, despois de medilo): funcionan o panel de emoji, o ditado e os IME. As mensaxes do IME (`WM_IME_…`) van a `DefWindowProcW`, que converte o texto escollido en `WM_CHAR`: caracteres sen tecla, coma na decisión 2.

Isto concreta o [ADR 0004](0004-conpty-empaquetado.md) no que deixaba para esta subfase: como se envían as teclas en win32-input-mode e en VT.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Enviar só caracteres e secuencias VT, coma un terminal de Unix | O programa non distinguiría Maiús+Intro de Intro, nin recibiría Ctrl+Espazo, Ctrl+Alt+? nin as teclas modificadoras soas. O ConPTY pide o win32-input-mode ao arrancar. |
| O protocolo de teclado de kitty | O ConPTY non o fala: pide o win32-input-mode. |
| Converter as teclas sen `TranslateMessage`, con `ToUnicodeEx` en cada tecla que baixa | `ToUnicodeEx` cambia o estado das teclas mortas, e habería que levalo a man, e tamén o teclado numérico con Alt. `TranslateMessage` xa o fai, coma para calquera programa de Windows. |
| Supor que a mensaxe seguinte sempre é o carácter | Coas teclas publicadas por outro programa, a seguinte pode ser outra tecla: o evento perdería o seu carácter ou levaría o doutra. Unha copia de meiga sen a decisión 3 falla en 18 das 38 teclas da comprobación co teclado español. |
| Esperar o carácter cun temporizador | Un temporizador por tecla, e a mesma confusión coas teclas publicadas. |
| Que esperen tamén as teclas que soben | Coas teclas publicadas, unha tecla pode subir antes de que cheguen os caracteres das anteriores, e o seu evento levaría un deles: con ´ e x, o x. .NET ignora as teclas que soben, e o x perderíase. Atopouno a CI na primeira versión. |
| Eventos sen carácter ao soltar a tecla | A consola e o Windows Terminal envíano, e os programas poden esperalo. |
| En VT, con AltGr sen carácter, o código de control da tecla sen modificadoras, coma o Windows Terminal | Co ~ de AltGr+4 no teclado español, unha tecla morta, enviaría `ESC` e `0x1C` antes do ~. Para saber se unha tecla é morta habería que chamar a `ToUnicodeEx` ao codificala, e `meiga-core` non usa APIs de Windows. AltGr serve para escribir caracteres, e unha tecla que con AltGr non escribe nada non envía nada. Atopouno a revisión do código. |
| Pasarlle a Windows todas as teclas con Alt | Alt cunha letra faría soar o aviso, e Alt e F10 entrarían no modo do menú. PSReadLine e moitos programas usan Alt cunha letra. |
| Que meiga ou o programa traten Alt+F4 | Alt+F4 pecha calquera xanela de Windows, tamén a do Windows Terminal. |
| Manter a tecla P para as marcas | Roubaríalle a P ao programa. |
| Desactivar o IME con `ImmDisableIME` | Arrancaría uns milisegundos antes, con 11 módulos menos, pero o panel de emoji non escribiría nada, e os IME de chinés, xaponés ou coreano non funcionarían. En repouso, o IME non custa nada. |
| *Raw input* (`WM_INPUT`) | Dá os códigos de exploración sen distribución: os caracteres, as teclas mortas e AltGr terían que facerse a man. |

## Consecuencias

**Positivas:**

- PSReadLine recibe os mesmos eventos ca na consola de Windows: Maiús+Intro, Ctrl+Espazo, Alt cunha letra, as teclas mortas e AltGr do teclado español e galego. `check-keyboard.ps1` compróbao na CI, con 38 teclas no teclado español e 28 no estadounidense.
- Escribir non engade ningún fío nin temporizador: o teclado vive nas mensaxes da xanela, e en repouso segue sen espertar.
- Alt+F4 e Alt+Espazo funcionan coma en calquera xanela, e o panel de emoji e o ditado tamén.
- Cada tecla escríbese nun búfer que se reutiliza, sen reservar memoria.

**Negativas e riscos:**

- Cada tecla con carácter son dúas mensaxes, e en win32-input-mode cada pulsación son polo menos dous eventos, ao baixar e ao subir, de ata uns 30 bytes cada un: máis bytes ca en VT, pero ao ritmo ao que se escribe.
- Se outro programa publica caracteres sen tecla mentres un evento espera, o evento podería levar un deles.
- O IME custa uns milisegundos no arranque e algo de memoria. A súa xanela de composición non vai onde está o cursor, porque meiga non lle di onde está: queda para as melloras futuras (F5).
- Os atallos de meiga (fase 2) collerán algunhas teclas antes do programa, coma Ctrl+Maiús+T: decídese na subfase 2.6, sen roubarlle ao programa as que xa usa.

## Referencias

- Microsoft Learn: [*Keyboard Input Overview*](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input), [`WM_KEYDOWN`](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-keydown), [`WM_SYSKEYDOWN`](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-syskeydown), [`WM_CHAR`](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-char), [`WM_DEADCHAR`](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-deadchar), [`TranslateMessage`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage), [`ToUnicodeEx`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex), [`KEY_EVENT_RECORD`](https://learn.microsoft.com/en-us/windows/console/key-event-record-str) e [`ImmDisableIME`](https://learn.microsoft.com/en-us/windows/win32/api/imm/nf-imm-immdisableime).
- Windows Terminal v1.25.2733.0: a especificación do win32-input-mode, [`doc/specs/#4999 - Improved keyboard handling in Conpty.md`](https://github.com/microsoft/terminal/blob/v1.25.2733.0/doc/specs/%234999%20-%20Improved%20keyboard%20handling%20in%20Conpty.md).
- [*XTerm Control Sequences*](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html): *PC-Style Function Keys*.
- [Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows), [› As teclas no terminal](../tecnoloxias.md#42-as-teclas-no-terminal-win32-input-mode-e-vt) e [› O IME e o panel de emoji](../tecnoloxias.md#43-o-ime-e-o-panel-de-emoji).
- [Código › meiga-core › `input.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcinputrs) e [Código › meiga-win › `keyboard.rs`](../codigo/meiga-win.md#cratesmeiga-winsrckeyboardrs).
